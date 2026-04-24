# Recording Overlay Pill Progress

Tracks which phases of the **Recording Overlay Pill** plan have been completed.
Updated after each phase commit. Read by agent prompts to determine current state.

## Current Phase

**Phase 0** — Not started. Overlay window infrastructure + GTK Layer Shell + capabilities.

## Phase Status

| Phase | Description | Status | Commit |
|-------|-------------|--------|--------|
| 0 | Overlay window infra + GTK Layer Shell + capabilities | 🔲 | — |
| 1 | FFT-based audio frequency band extraction | 🔲 | — |
| 2 | Overlay frontend (HTML/CSS/JS pill) | 🔲 | — |
| 3 | Wire overlay + level emission to orchestrator | 🔲 | — |
| 4 | Polish, animations, and edge cases | 🔲 | — |

## Status Legend

| Symbol | Meaning |
|--------|---------|
| 🔲 | Not started |
| 🔧 | In progress (agent working) |
| ✅ | Complete — committed and verified |
| ❌ | Blocked — see notes |

## Notes

- Plan document: `plans/recording-overlay-pill/PLAN.md` (v3.0, Draft)
- Gap report: `plans/recording-overlay-pill/GAP-REPORT.md` (17 gaps; 9 from v1.0, 8 from v3.0 grounding)
- All phases must pass `cargo check` and the app must launch without panic
- Each phase gets one squashed commit on `main`
- Read the full AGENT-PROMPT.md for detailed phase instructions
- Target platform: **Linux** (X11 + Wayland)
- **Threading rule:** All overlay window ops go through `run_on_main_thread()` — pipeline() runs on spawn_blocking (GAP-2)
- **Emission order:** Emit Tauri events before show/hide overlay calls (GAP-14)
- **Crate structure:** All new modules go in `voice_transcribe_lib` (lib.rs), called from main.rs setup closure (GAP-16)

## Blocking Issues

- None currently

## Phase Details

### Phase 0
**Scope:** Create `infrastructure/overlay.rs` with thread-safe overlay window creation/positioning. Initialize GTK Layer Shell at creation time on Linux. Add `"overlay"` to capabilities with required permissions. Create placeholder `src/overlay.html`. Window: transparent, no shadow, no decorations, always-on-top, click-through, hidden by default.
**Context:** Follow Keyless for thread-safe show/hide (`run_on_main_thread`). Follow Handy for GTK Layer Shell at creation. See GAP-1, GAP-2, GAP-4, GAP-5, GAP-6.

### Phase 1
**Scope:** Create `infrastructure/audio/eq.rs` with `EqState` (realfft, Hann window, 1024pt FFT, 7 log-spaced bands, attack/decay smoothing). Add `eq_rx` field to `RecorderHandle` — channel created in `spawn_for_device()`, `EqState` created per-recording in `Start` arm. This is infrastructure only — no Tauri event emission.
**Context:** Follow Keyless `eq_pipeline.rs` exactly. EqState is created inside the Start arm of recorder_thread (GAP-11). Channel created alongside existing channels in spawn_for_device (GAP-10). realfft already compiled transitively via `rubato` v0.15.0 (direct dep, not feature-gated) (GAP-17). Note: the Start arm has three `build_input_stream` calls (F32, I16, fallback) — EqState feeding must happen in all three.

### Phase 2
**Scope:** Build `src/overlay.html` with full pill UI: 7 EQ bars, frosted glass, scale animations, state transitions. Listen for existing Tauri events. `html, body { background: transparent; }`. No `<script src="vendor/tauri.js">` — use `window.__TAURI__` directly (GAP-13).
**Context:** Follow Keyless Pill.tsx/Pill.css for animations. Use app design tokens from style.css.

### Phase 3
**Scope:** Add `level_cancel: Arc<AtomicBool>` to AppState (GAP-12). Call `show_overlay()`/`hide_overlay()` from orchestrator. Emit events BEFORE show/hide calls (GAP-14). Spawn level emission task in `on_press()` that reads `eq_rx` and emits `mic-level` at ~30fps. Stop task via cancel flag in `on_release()`. `hide_overlay()` must use `run_on_main_thread()` for pipeline's blocking thread.
**Context:** See GAP-2 (threading), GAP-9 (responsibility split), GAP-12 (cancellation), GAP-14 (emit order). `on_press`/`on_release` receive `&AppHandle` (GAP-15).

### Phase 4
**Scope:** Edge cases: rapid PTT toggle, shutdown cleanup, monitor reconnect. Visual polish. Test on X11 + Wayland. Comment on single-instance callback. Verify click-through and no focus stealing.
**Context:** See GAP-8 (single-instance comment).

## Completed Phases

| Phase | Commit | Summary |
|-------|--------|---------|
| — | — | No phases completed yet |
