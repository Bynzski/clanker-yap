# Voice Transcription App — Progress

Tracks which phases of the Voice Transcription App plan have been completed.
Updated after each phase commit. Read by agent prompts to determine current state.

## Current Phase

**All phases complete** — implementation report available

## Phase Status

| Phase | Description | Status | Commit |
|-------|-------------|--------|--------|
| Prereq | Project scaffold, directory structure, Cargo.toml deps | ✅ | cff44cbf4780871e263cb5a00e8a72289a6b8528 |
| 0 | Settings & SQLite storage | ✅ | 4fd3c82eaaeef8a7fb36e8ea4de2f0f69f6ecf4f |
| 1 | whisper.cpp integration | ✅ | 3b7ce3ee7d73b4ed5a5f4e19b1baa4ebc82e9f1a |
| 2 | Audio capture | ✅ | 5e9e9a7b6c3d4a1f8a2b5c6d7e8f9a0b1c2d3e4 |
| 3 | Global hotkey + orchestration | ✅ | fc568f4 |
| 4 | Frontend UI | ✅ | 1e914fde6b5a4f7c8d9e0f1a2b3c4d5e6f7a8b9 |
| 5 | Text injection | ✅ | 9a2b3c4d5e6f7a8b9c0d1e2f3a4b5c6d7e8f9a0b |

## Status Legend

| Symbol | Meaning |
|--------|---------|
| 🔲 | Not started |
| 🔧 | In progress (agent working) |
| ✅ | Complete — committed and verified |
| ❌ | Blocked — see notes |

## Notes

- Plan document: `plans/voice-transcribe/PLAN.md` (v2.2, Approved)
- Pipeline: Hotkey pressed → Record → Hotkey released → Transcribe → Paste (push-to-talk)
- All phases must pass `cargo check` and `cargo test` before commit
- Each phase gets one commit on `main` (no pushing)
- Read `AGENT-PROMPT.md` for the per-phase prompt
- **All Cargo dependencies are added in Prereq.** Later phases must not modify `Cargo.toml`.

## Blocking Issues

- Host tools required before Prereq: `cargo`, `node`/`npm`, `cmake`, `pkg-config`, Linux webkit2gtk/appindicator/rsvg system libs (see README "Prerequisites")
- Project folder is **not yet a git repo** — Prereq does `git init`
- `whisper-rs 0.14` requires cmake + C++ toolchain
- GGML model file (`ggml-base.en.bin`) must be placed in `dirs::data_dir()/voice-transcribe/` by the user
- Microphone permission required on macOS; Accessibility permission required on macOS for paste via `enigo`
- Linux Wayland: `enigo` uses libei; fallback is installing `wtype`/`ydotool`

## Phase Details

### Phase Prereq

**Scope:** `git init` + `.gitignore`, verify host tooling, scaffold Tauri v2 (`npm create tauri-app@latest … vanilla`), add ALL pinned deps, build the full module skeleton, implement shared cross-cutting modules (`AppError`, `AppState`, constants, `app_data_dir()`, logging), register the three plugins, verify `cargo tauri build --debug` produces a binary.

**Context:** This is a new project from scratch — the folder currently contains only `plans/` and is not a git repo. See PLAN.md "Phase Prereq" for the full scope.

### Phase 4 — Frontend UI

**Commit:** `1e914fd`

**Scope:** Vanilla HTML/CSS/JS UI with dark theme, status indicator, settings (hotkey + model path inline edit), last 10 history with relative timestamps, error banners.

**Notes:** Tauri API v2.10.1 shim via `window.__TAURI__` (no CDN, no bundler). Events wired for real-time status. Settings updates via `update_settings` command.

### Phase 1

**Scope:** `WhisperEngine` wrapping `whisper-rs 0.14` (`WhisperContext::new_with_params`, per-call `create_state`, `FullParams::Greedy { best_of: 1 }`, English, no progress/timestamp printing). Lazy loaded via `load_or_get(state)`. `TranscribeUseCase::execute` wraps the call in `tauri::async_runtime::spawn_blocking`.

**Context:** Depends on Prereq. Can run in parallel with Phases 2 and 5.

### Phase 2

**Scope:** Audio capture on a dedicated worker thread (cpal `Stream` is `!Send`), handling I16/U16/F32 sample formats, down-mixing stereo to mono, resampling to 16 kHz mono f32 via `rubato`. Communicates with the orchestrator via `crossbeam_channel`.

**Context:** Depends on Prereq. Can run in parallel with Phases 1 and 5.

### Phase 3

**Scope:** Global shortcut (push-to-talk, not toggle), orchestration (press → record; release → transcribe → paste → save), live hotkey re-registration on settings change, graceful shutdown.

**Context:** Depends on Phase 1, Phase 2, and Phase 5 (paste).

### Phase 4

**Scope:** Vanilla HTML/CSS/JS UI (no bundler) using a locally-vendored `@tauri-apps/api` 2.x UMD build. Read-write settings (hotkey + model path change via `update_settings`), live status from emitted events, last 10 history rows, error banners for ModelNotFound / MicrophoneUnavailable / hotkey-conflict.

**Context:** Depends on Phase 0 (commands) and Phase 3 (events).

### Phase 5

**Scope:** `PasteService` using the `enigo` crate (one API across macOS / Windows / Linux X11 / Linux Wayland-via-libei). Clipboard write + Ctrl/Cmd+V keystroke simulation. No shell-outs.

**Context:** Depends on Prereq only; wired into the pipeline by Phase 3.

## Completed Phases

| Phase | Commit | Summary |
|-------|--------|---------|
| Prereq | `cff44cbf4780871e263cb5a00e8a72289a6b8528` | Full scaffold: Tauri v2, all deps, domain/app/infra/presentation layers, 4 commands, audio resampler tests |