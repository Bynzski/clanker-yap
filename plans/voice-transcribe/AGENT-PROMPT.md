# Agent Prompt — Voice Transcription App (v2.2)

Reusable prompt for executing **one** phase of `plans/voice-transcribe/PLAN.md`. Each invocation runs in a fresh agent session.

---

## How to Use

1. Read `plans/voice-transcribe/PROGRESS.md` to confirm the next phase.
2. Copy the block below (the outer fence is `~~~` so the inner code blocks render cleanly).
3. Replace `{{PHASE}}` with one of: `Prereq`, `0`, `1`, `2`, `3`, `4`, `5`.
4. Paste into a fresh agent session.
5. After the agent commits, update `PROGRESS.md` with the commit hash.

---

## Prompt

~~~
You are implementing Phase {{PHASE}} of the Voice Transcription App plan (v2.2).

## Context — read these first

1. `plans/voice-transcribe/PLAN.md` — the approved plan. Pay attention to:
   - "Cross-Cutting Concerns" (AppError, AppState, constants, logging, push-to-talk decision, single-instance, concurrency rules)
   - the specific Phase {{PHASE}} section
   - the "Dependency Summary" table (all deps are added in Prereq; later phases must not touch `Cargo.toml`)
   - the "File Structure" section
2. `plans/voice-transcribe/PROGRESS.md` — confirm {{PHASE}} is the next unstarted phase.

## Global Rules (apply to every phase)

1. **Do exactly what the phase scope describes.** Do not add features outside scope.
2. **No new `Cargo.toml` dependencies** in any phase except Prereq. If you think you need one, STOP and report.
3. Every module must be registered in its parent `mod.rs`.
4. All errors use `crate::domain::error::{AppError, Result}`.
5. All CPU-bound work (transcription) runs inside `tauri::async_runtime::spawn_blocking`.
6. Use `parking_lot::Mutex`, never `std::sync::Mutex`.
7. Use `tracing::{info, warn, error, debug}` — no `println!`.
8. Platform-conditional code uses `#[cfg(target_os = "...")]`.
9. Run after each meaningful change: `cargo check --manifest-path src-tauri/Cargo.toml`.
10. Run before committing: `cargo test --manifest-path src-tauri/Cargo.toml` — must pass (pre-existing failures must be reported, not masked).
11. No `--no-verify`, no `git add -A` (prefer explicit paths), no amending prior commits.

## Phase Definitions

### Phase Prereq — Project Scaffold

**CRITICAL:** this phase must leave the repo in a buildable state. Every subsequent phase assumes it.

Steps:

1. `git init` in the project root. Create `.gitignore` with `target/`, `node_modules/`, `dist/`, `.DS_Store`, `*.log`, `src-tauri/gen/`.
2. Verify host tooling (`cargo --version`, `node --version`, `npm --version`, `cmake --version`, `pkg-config --version`). If anything is missing, STOP and report — do not attempt `sudo`.
3. Scaffold Tauri v2 in-place:
   ```sh
   npm create tauri-app@latest -- --template vanilla --manager npm \
     --identifier dev.jay.voice-transcribe --yes .
   ```
4. Edit `src-tauri/Cargo.toml` to add all dependencies from the PLAN "Dependency Summary" table. Pin versions exactly as listed.
5. Create the full directory layout from PLAN "Phase Details → Phase Prereq → Scope". Every `mod.rs` must exist and re-export its children (even if some sub-files are empty placeholders for later phases).
6. Implement in full NOW:
   - `domain/error.rs` — the complete `AppError` enum (with `serde::Serialize` impl)
   - `domain/constants.rs` — the complete constants block
   - `application/state.rs` — `AppState` + `RecordingState`
   - `infrastructure/persistence/paths.rs` — `app_data_dir()`
   - `lib.rs` — `init_logging()` + module re-exports
   - `main.rs` — Tauri builder registering `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`, `tauri-plugin-single-instance`; calling `init_logging()`; placeholder `AppState` (no DB yet — Phase 0 replaces); `RunEvent::ExitRequested` hook
7. Replace `src-tauri/tauri.conf.json` with the block shown in PLAN Prereq scope (productName, identifier, window 480×560, no devUrl, frontendDist `../src`).
8. Write `src-tauri/capabilities/default.json` with exactly the four permissions from PLAN (no `shell:allow-execute`).
9. Add `src-tauri/entitlements.plist` (audio-input entitlement) and the `macOS` block in `tauri.conf.json`.
10. Replace `src/index.html` with a minimal dark-themed "Voice Transcription — loading…" shell. `src/main.js` and `src/style.css` can be stubs.
11. Verify: `cd src-tauri && cargo check` passes; `cargo tauri build --debug` produces a binary.

Out of scope: any business logic, persistence, whisper, audio, paste, UI beyond the loading shell.

### Phase 0 — Settings & Persistence

**CRITICAL:** settings are stored as a single JSON row, not key-value. Read PLAN "Phase 0" schema carefully.

Scope:
- `domain/settings.rs` — `Settings { hotkey, model_path, model_name, schema_version }` with `Default` that resolves `model_path` via `app_data_dir().join(DEFAULT_MODEL_FILE)`.
- `domain/transcription.rs` — `Transcription::new` validates `MAX_TRANSCRIPTION_LENGTH` and duration range.
- `infrastructure/persistence/db.rs` — `Db::open()` ensures `app_data_dir()` exists, opens `voice-transcribe.db`, runs the three `CREATE TABLE IF NOT EXISTS` + one `CREATE INDEX IF NOT EXISTS` from PLAN.
- `settings_repo.rs` — `load_or_init(db)` inserts defaults if row 1 missing; `save(db, settings)`.
- `transcription_repo.rs` — `save`, `recent(limit)` ordered DESC, `prune_to(keep)` called after every save.
- Use cases in `application/use_cases/` for get/update settings and save/get history.
- DTOs per PLAN "Phase 0" — `SettingsResponse`, `UpdateSettingsRequest`, `UpdateSettingsResponse { success, message, requires_restart }`, `TranscriptionItem`, `TranscriptionHistoryResponse`.
- Tauri commands: `get_settings`, `update_settings`, `get_transcription_history`. Register all three in `main.rs::invoke_handler`.
- Replace Prereq's placeholder `AppState` with one built from real `Db` + loaded `Settings`.

Out of scope: whisper (Phase 1), audio (Phase 2), hotkey (Phase 3), UI (Phase 4), paste (Phase 5).

### Phase 1 — whisper.cpp Integration

Scope:
- `infrastructure/whisper/engine.rs` — `WhisperEngine::load(path)` returns `AppError::ModelNotFound` if file missing; otherwise builds `WhisperContext`.
- `WhisperEngine::transcribe(samples: &[f32]) -> Result<String>` using the real whisper-rs 0.14 API (`create_state`, `FullParams::new(SamplingStrategy::Greedy { best_of: 1 })`, language "en", translate false, no progress/timestamp printing, `num_cpus::get_physical()` threads). Iterate segments via `full_n_segments` / `full_get_segment_text`.
- `infrastructure/whisper/mod.rs::load_or_get(state)` — lazy-loads + caches in `state.whisper`. Returns `Arc<WhisperEngine>`.
- `application/use_cases/transcribe.rs` — `TranscribeUseCase::execute` wraps the engine call in `tauri::async_runtime::spawn_blocking`.
- Duration validation: reject if `samples.len() * 1000 / 16_000 < MIN_RECORDING_DURATION_MS`.
- `tracing::info!(samples, elapsed_ms, "transcribed")`.

Audio contract: samples are **already** 16 kHz mono f32 in [-1, 1] — Phase 2 guarantees this. Do NOT resample here.

Out of scope: audio capture, hotkey, paste.

### Phase 2 — Audio Capture (+ resampling)

**CRITICAL:** `cpal::Stream` is `!Send`. Do NOT store it in `AppState`. Own it inside a dedicated `std::thread` and communicate via channels.

Scope:
- `infrastructure/audio/recorder.rs` — `RecorderHandle { cmd_tx, result_rx, _join }`:
  - `spawn()` starts a worker thread; picks `cpal::default_host().default_input_device()`; returns `AppError::MicrophoneUnavailable` if `None`.
  - Worker handles `RecorderCmd::{Start, Stop, Shutdown}`.
  - On `Start`: build stream matching device's native sample format + rate; callback converts to f32, down-mixes to mono, appends to buffer.
  - Support all three sample formats: `I16`, `U16`, `F32`.
  - On `Stop`: drop stream, resample buffer to 16 kHz via `infrastructure/audio/resample.rs`, send result.
  - Auto-stop if buffer exceeds `MAX_RECORDING_DURATION_MS`.
- `infrastructure/audio/resample.rs` — thin wrapper around `rubato::SincFixedIn<f32>` for `input_rate → WHISPER_SAMPLE_RATE`. Short-circuit if already 16 kHz.
- Error variants: `MicrophoneUnavailable`, `Audio("Permission denied")`, `Audio("Recording too short")`, `Audio("Stream build failed: ...")`.
- Unit test: synthesise a 48 kHz sine buffer, run through resampler, assert length ≈ input_len * 16_000 / 48_000 within tolerance.

Out of scope: whisper calls, hotkey wiring, UI.

### Phase 3 — Global Hotkey & Orchestration

**CRITICAL:** push-to-talk. Pressed = start, Released = stop + transcribe + paste + save. Not toggle.

Scope:
- Register global shortcut via `tauri-plugin-global-shortcut` using `AppState.settings.hotkey`. Handler distinguishes `ShortcutState::Pressed` vs `Released`.
- `application/orchestrator.rs`:
  - `on_press(app, state)`: if `RecordingState::Idle`, ensure recorder spawned, `recorder.start()`, transition to `Recording { started_at }`, emit `recording-started`.
  - `on_release(app, state)`: if `Recording`, transition to `Processing`, emit `recording-stopped { duration_ms }`. Inside `spawn_blocking`: `stop_and_collect` → duration check → `load_or_get` engine → `transcribe` → `paste::inject` (log + emit error on failure, continue) → save via `SaveTranscriptionUseCase`. Emit `transcription-complete` on success, `transcription-error` on any failure. Transition to `Idle`.
  - Ignore repeated presses while not `Idle`; log at `debug`.
- Live hotkey change (called by `update_settings` when hotkey differs): unregister old, register new; roll back on failure. Set `requires_restart = false` only if rollback succeeds cleanly.
- `RunEvent::ExitRequested`: send `RecorderCmd::Shutdown`; drop whisper engine.
- Hotkey conflict on initial registration → emit `hotkey-conflict { hotkey }`.
- Add `get_status` command (returns `{ state, last_error? }`) and register it.

Frontend events emitted: `recording-started`, `recording-stopped`, `transcription-complete`, `transcription-error`, `hotkey-conflict`.

Out of scope: UI rendering, paste internals.

### Phase 4 — Frontend UI

Scope:
- `src/index.html` — full dark-theme layout from PLAN (status indicator, hotkey + [Change…], model name + path + [Change…], recent-history list, error banner).
- `src/vendor/tauri.min.js` — download the @tauri-apps/api 2.x UMD build and reference locally (no CDN dependency at runtime). Document the version in a comment at the top of `main.js`.
- `src/style.css` — dark theme, system font stack, responsive to the 480×560 window.
- `src/main.js`:
  - On load: `invoke('get_settings')`, `invoke('get_transcription_history')`, `invoke('get_status')`.
  - `listen(...)` for all five events; update status indicator + history.
  - [Change…] buttons open inline inputs; submit → `invoke('update_settings', { request: {...} })`. If `requires_restart`, show "Restart required" banner.
  - Error banner paths: `ModelNotFound` (with link to https://huggingface.co/ggerganov/whisper.cpp/tree/main), `MicrophoneUnavailable`, `hotkey-conflict`, generic `transcription-error`.
  - Truncate history text to 80 chars; relative timestamps computed client-side.

Out of scope: multi-page routing, frameworks, history management beyond last 10, model download.

### Phase 5 — Text Injection

**CRITICAL:** use `enigo` uniformly — do NOT shell out to `osascript`, `xdotool`, `cmd`, or `powershell`.

Scope:
- `infrastructure/paste/service.rs::inject(app, text)`:
  1. `app.clipboard().write_text(text.to_string())?`
  2. Build `Enigo::new(&Settings::default())?`.
  3. Press `Key::Meta` on macOS (`cfg!(target_os = "macos")`), else `Key::Control`.
  4. Click `Key::Unicode('v')`.
  5. Release the modifier.
- `application/use_cases/paste.rs::PasteUseCase` wraps the service.
- On `AppError::PasteFailed`: log at `warn`, return error. The orchestrator (Phase 3) still saves the transcription so the text remains in the clipboard for manual paste.
- Document per-OS permission requirements in `plans/voice-transcribe/README.md` (macOS Accessibility, Linux Wayland libei fallback).

Out of scope: direct input-field targeting, OCR, accessibility-API hooking.

## Commit

When phase work is done and `cargo check` + `cargo test` both pass:

1. Stage only the files you modified or created (never `git add -A`).
2. Commit with exactly this format:

   ```
   <type>(<scope>): phase {{PHASE}} — <short description>

   - <bullet of what was done>
   - <...>

   Phase {{PHASE}} of plans/voice-transcribe/PLAN.md (v2.2)
   ```

   Types: `feat`, `fix`, `refactor`, `docs`, `chore`. Scope is usually `app` or a module name.

3. Do NOT push.

## Completion Report

After committing, output:

```
## Phase {{PHASE}} Complete (v2.2)

### Changes
- <file>: <what changed>

### New Files
- <file>: <purpose>

### Verification
- cargo check: PASS/FAIL
- cargo test:  PASS/FAIL
- cargo tauri build --debug: PASS/FAIL/NOT RUN (note why)

### Commands Registered (if any)
- <list>

### Issues Found
- <issue or "None">

### Ready for Next Phase
- YES/NO (+ blocker if NO)
```

Then update `plans/voice-transcribe/PROGRESS.md`:
- Set the completed phase to ✅ with the commit hash.
- Leave the next phase as 🔲 (the next agent sets it to 🔧 on start).

## Absolute Hard Stops

STOP and report if any of these occur:

- `cargo check` fails and the fix is outside the phase scope.
- `cargo test` fails due to something you did (pre-existing failures must be called out before you touch them).
- You need to add a `Cargo.toml` dependency that isn't already in the Dependency Summary table.
- You need to edit a file outside this phase's "Scope" or "Files".
- The plan says a module/function exists that doesn't actually exist in the repo.
- Host tooling is missing during Prereq (no `sudo` — just report).
- `whisper-rs` or `enigo` linker errors that can't be resolved by reading the crate's README.

Report blockers clearly. Do not push through them.
~~~

---

## Template Variables

| Variable | Replace With |
|----------|--------------|
| `{{PHASE}}` | `Prereq`, `0`, `1`, `2`, `3`, `4`, or `5` |

## Phase Dependency Graph

```
Prereq
  ├── Phase 0 ──────────────────┐
  ├── Phase 1 ────────────┐     │
  ├── Phase 2 ────────────┤     │
  └── Phase 5 ────────────┤     │
                          │     │
              Phase 3 ◄───┘     │
              Phase 4 ◄─────────┘
```

Phases 0, 1, 2, 5 can run in parallel after Prereq. Phase 3 needs 1+2+5. Phase 4 needs 0 (for commands) and ideally 3 (for events).

## Blocking External Issues

- `whisper-rs` requires cmake + C++ toolchain on the build host.
- GGML model file (`ggml-base.en.bin`) must be downloaded by the user post-install.
- macOS builds need Accessibility permission for paste (surfaced by UI on first failure).
- Linux Wayland: `enigo` uses libei; fallback = install `wtype`/`ydotool`.
