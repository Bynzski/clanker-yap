# Agent Directive: Execute Recording Overlay Pill Plan

Self-contained instructions for an agent to execute one phase of the recording overlay pill implementation.

---

## How It Works

1. **Read progress** — Load `plans/recording-overlay-pill/PROGRESS.md`. Find the first phase with status:
   - 🔲 (not started) → this is the next phase
   - 🔧 (in progress) → resume this phase

2. **Read plan** — Load `plans/recording-overlay-pill/PLAN.md`. Study the target phase section:
   - Scope: what to do
   - Out of scope: what NOT to do
   - Files: what to create/modify
   - Context: what patterns to follow

3. **Read gap report** — Load `plans/recording-overlay-pill/GAP-REPORT.md`. Understand the issues that were found and why the plan is structured the way it is. Do not reintroduce solved gaps.

4. **Execute** — Implement the phase per plan scope.

5. **Validate** — Run `cargo check` from the project root. Ensure no errors.

6. **Commit** — Commit with format:
   ```
   feat(overlay): phase <N> — <short description>

   <bullet list of changes>

   Phase <N> of plans/recording-overlay-pill/PLAN.md
   ```

7. **Update progress** — Mark phase as ✅ with commit hash in `PROGRESS.md`.

---

## Critical Rules

### Rule 1: Thread Safety (GAP-2)
`pipeline()` runs on `spawn_blocking`. `on_press()`/`on_release()` run on the main thread. **ALL** `WebviewWindow` method calls in `overlay.rs` must be wrapped in `app.run_on_main_thread()`. There are no exceptions.

### Rule 2: Capabilities (GAP-1)
The overlay window must be listed in `src-tauri/capabilities/default.json` with the required permissions. Without this, window operations will silently fail.

### Rule 3: Transparency (GAP-4)
Both `<html>` AND `<body>` must have `background: transparent`. The window builder must include `.shadow(false)`. Verify the overlay appears transparent, not opaque black.

### Rule 4: FFT, Not RMS (GAP-3)
Phase 1 uses FFT (`realfft` crate) to produce 7 frequency bands. RMS is not sufficient for multi-bar visualization. Follow Keyless's `EqState` pattern.

### Rule 5: Responsibility Split (GAP-9)
- **Phase 1 (infrastructure):** `EqState` + `eq_rx` channel on `RecorderHandle`. No `AppHandle`, no Tauri events.
- **Phase 3 (application):** Orchestrator spawns task that reads channel and emits events.

### Rule 6: Recorder Channel Design (GAP-10)
The `eq_rx` channel is created in `spawn_for_device()` alongside existing `cmd_tx`/`result_tx` channels. `eq_rx` is stored on `RecorderHandle` as a public field. `eq_tx` is passed to `recorder_thread()` as a parameter.

### Rule 7: EqState Per-Recording (GAP-11)
`EqState::new(sample_rate)` is created inside the `Start` arm of `recorder_thread`, NOT as a persistent field. It lives inside the cpal callback closure and is dropped when the stream is dropped on `Stop`.

### Rule 8: Cancellation via AtomicBool (GAP-12)
Phase 3 adds `level_cancel: Arc<AtomicBool>` to `AppState`. The level emission task checks this flag. Set `true` in `on_release()`, reset `false` in `on_press()` before spawning.

### Rule 9: Emit Before Show/Hide (GAP-14)
Always emit the Tauri event BEFORE calling show/hide overlay. Hidden windows receive events, so the overlay renders the correct state before becoming visible. Order: `app.emit("recording-started")` → `show_overlay(&app)`.

### Rule 10: Overlay Frontend API Access (GAP-13)
The overlay HTML uses `window.__TAURI__` directly (injected by `withGlobalTauri: true` in tauri.conf.json). No `<script src="vendor/tauri.js">` tag is needed. Use `const { listen } = window.__TAURI__.event;`.

### Rule 11: &AppHandle References (GAP-15)
`on_press` and `on_release` receive `&AppHandle` and `&AppState` references. Overlay functions should accept `&AppHandle` and `.clone()` for closures. `AppHandle` is `Clone + Send + Sync`.

---

## Key Reference Files

These external repos contain the patterns to follow. Read them before implementing.

| What | Where |
|------|-------|
| Thread-safe overlay show/hide | https://github.com/hate/keyless/blob/main/keyless-desktop/src-tauri/src/overlay.rs |
| FFT EQ pipeline (`realfft`) | https://github.com/hate/keyless/blob/main/keyless-audio/src/eq_pipeline.rs |
| EQ pill component (CSS) | https://github.com/hate/keyless/blob/main/keyless-desktop/src/overlay/pill/Pill.css |
| GTK Layer Shell at creation | https://github.com/cjpais/Handy/blob/main/src-tauri/src/overlay.rs |
| Capabilities with overlay window | https://github.com/cjpais/Handy/blob/main/src-tauri/capabilities/default.json |

---

## Codebase Context

The crate is a lib+binary split:
- `src-tauri/src/lib.rs` — exports `voice_transcribe_lib` with all modules
- `src-tauri/src/main.rs` — thin binary, creates app and calls setup

New modules (`overlay.rs`, `eq.rs`) go in the lib crate under `infrastructure/`. The `create_overlay()` function is `voice_transcribe_lib::infrastructure::overlay::create_overlay()`. It's called from `main.rs`'s `.setup()` closure which has access to `app.handle()`.

The existing `RecorderHandle` (`src-tauri/src/infrastructure/audio/recorder.rs`) uses `crossbeam_channel` for all communication. The cpal stream is created fresh in the `Start` arm and dropped in the `Stop` arm. The `EqState` follows this same lifecycle pattern.

`realfft` is already compiled as a direct dependency of `rubato` v0.15.0 (confirmed in Cargo.lock: `realfft` v3.5.0 is a direct dep of `rubato`). Adding it as a direct dependency in `Cargo.toml` adds zero compile time.

---

## General Rules

1. Read ALL context files (PLAN.md, PROGRESS.md, GAP-REPORT.md) before writing code.
2. Execute ONLY the scope defined in the current phase. Do NOT:
   - Modify files outside phase scope
   - Refactor unrelated code
   - Add features not in scope
3. Follow existing codebase patterns — do not invent new patterns.
4. The project is Linux-targeted. Use `#[cfg(target_os = "linux")]` for platform-specific code.
5. Run `cargo check` frequently. Fix errors immediately.
6. If blocked, report immediately. Do NOT work around blockers silently.
7. The overlay frontend is vanilla HTML/CSS/JS (no framework) — match the existing `src/main.js` and `src/style.css` patterns.
8. Use the app's existing design tokens from `src/style.css` (colors, fonts, spacing).
9. `withGlobalTauri: true` is set in `tauri.conf.json` — ALL windows get `window.__TAURI__` injected automatically.

---

## Completion

When the phase is complete and `cargo check` passes:

```
## Phase <N> Complete

### Files Changed
- <file>: <what>

### New Files
- <file>: <purpose>

### Validation
- cargo check: PASS/FAIL
- build: PASS/FAIL
- app launch: PASS/FAIL

### Commit
<hash>

### Ready for Next Phase
YES/NO
```

Then update `plans/recording-overlay-pill/PROGRESS.md`:
- Set completed phase status to ✅ with commit hash
- Set next phase status to 🔲

---

## Invocation

```
Execute the next phase of the recording-overlay-pill plan.
PLAN_PATH: plans/recording-overlay-pill/
```
