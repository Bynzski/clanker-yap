# Plan Review Checklist — v2.2

This is the gap-report review record, not the generic template. Each item is ticked with evidence of where in PLAN.md / AGENT-PROMPT.md it's addressed.

## Plan Document (PLAN.md)

### Header
- [x] Author filled in (`jay`)
- [x] Date is current (`2026-04-22`)
- [x] Status is `Approved`
- [x] Version bumped to `2.2`

### Purpose
- [x] Clear problem statement — local, low-resource push-to-talk dictation
- [x] Explains why — privacy, low resource, minimal UI
- [x] No implementation details in purpose section

### Scope
- [x] In-scope items listed with priorities (P0/P1)
- [x] Out-of-scope explicit (Ollama, LLM post-processing, tray, autostart, multi-language, etc.)
- [x] Single-instance enforcement promoted to P0

### What Already Exists
- [x] Documents the empty-folder, no-git starting state
- [x] Prereq handles `git init`

### Dependencies
- [x] Host tooling (cargo, node, cmake, pkg-config) listed
- [x] Linux system libs listed with example `pacman`/`apt` commands
- [x] Wayland paste fallback documented (`wtype`/`ydotool`)
- [x] Runtime-only dep (GGML model) explicitly Blocked until user downloads

### Cross-Cutting Concerns
- [x] `AppError` enum is complete, with `serde::Serialize` impl for Tauri
- [x] `AppState` documents `!Send` handling for `cpal::Stream` and `WhisperState`
- [x] Logging (`tracing`) initialised once
- [x] Shutdown via `RunEvent::ExitRequested`
- [x] `dirs::data_dir()` chosen as the single source of truth for paths
- [x] Constants use `WHISPER_SAMPLE_RATE` (not ambiguous "DEFAULT_SAMPLE_RATE")
- [x] **Push-to-talk vs toggle RESOLVED** — push-to-talk
- [x] Single-instance plugin added
- [x] Concurrency rules explicit (hotkey → non-blocking; transcription → `spawn_blocking`; cpal → dedicated thread)

### Implementation Plan
- [x] Phase order is logical (dependencies respected)
- [x] Dependency graph included; 0/1/2/5 parallelisable after Prereq
- [x] No phase over-reaches

### Phase Details
- [x] Prereq includes `git init`, host-deps verification, all pinned deps (so later phases don't touch Cargo.toml)
- [x] Phase 0 settings schema is single JSON row (not ambiguous key-value)
- [x] Phase 0 defines first-run bootstrap (`load_or_init`)
- [x] Phase 0 prunes history at write time + indexes `created_at`
- [x] Phase 1 uses the real `whisper-rs 0.14` API (`WhisperContext` + per-call `WhisperState`, `FullParams`, `Greedy { best_of: 1 }`)
- [x] Phase 1 wraps transcription in `spawn_blocking`
- [x] Phase 2 owns `cpal::Stream` on a dedicated thread (works around `!Send`)
- [x] Phase 2 handles I16/U16/F32 sample formats
- [x] Phase 2 resamples to 16 kHz via `rubato`
- [x] Phase 2 down-mixes stereo to mono
- [x] Phase 3 is explicitly push-to-talk and handles live hotkey re-registration
- [x] Phase 3 debounces rapid presses / ignores release without press
- [x] Phase 4 UI is read-write (update_settings wired)
- [x] Phase 4 vendors `@tauri-apps/api` locally (no CDN at runtime)
- [x] Phase 5 uses `enigo` (no `osascript`, `xdotool`, `cmd echo v | clip`)
- [x] Phase 5 documents macOS Accessibility + Wayland libei fallback

### File Structure
- [x] Layout covers every module touched in every phase
- [x] `entitlements.plist` included for macOS
- [x] No mythical `build.rs` custom content

### Testing Strategy
- [x] Unit: schema bootstrap idempotency, history pruning, resampler length, error serde
- [x] Integration (manual): end-to-end dictation, settings persistence, missing-model banner
- [x] Smoke: build + launch + clean shutdown

### Rollout Gates
- [x] Per-phase verification criterion listed

### Change Log
- [x] v2.2 entry summarises the gap-report fixes

## Agent Prompt (AGENT-PROMPT.md)

- [x] Outer fence is `~~~` so inner ` ``` ` blocks render
- [x] Global rules forbid new Cargo deps after Prereq
- [x] Every phase section mirrors PLAN exactly
- [x] Hard Stops cover missing host tooling, link errors, scope creep
- [x] Commit format + completion report template included
- [x] Dependency graph in trailing section

## README.md

- [x] Version 2.2
- [x] Push-to-talk pipeline in the diagram
- [x] Host-deps section with Arch + Ubuntu examples
- [x] Model path table per-OS
- [x] Feature list reflects single-instance + enigo-based paste

## PROGRESS.md

- [x] Notes point at v2.2
- [x] Blocking issues updated (host tools, git init, cmake, Wayland)
- [x] Per-phase notes align with PLAN v2.2 scope

## General Quality

- [x] No "TODO" / "TBD" placeholders
- [x] No fabricated APIs (whisper-rs, cpal, enigo signatures match crate docs)
- [x] All path references use the same `dirs::data_dir()` scheme
- [x] Version pins are concrete (`whisper-rs 0.14`, `rubato 0.15`, etc.)

## Known Deferred Items (out of scope, acknowledged)

- Autostart on login (future)
- System tray / background-only mode (future)
- In-app model downloader (future)
- Code signing / notarisation / installers (future)
- Multi-language models (English only in v1)
- Full transcription-history management UI (future)
- CI / `cargo fmt --check` / `cargo clippy` gates (recommend adding post-Prereq; not a blocker)

## Sign-Off

- [x] Author has reviewed the v2.2 pass
- [x] Gap report items 1–28 from the 2026-04-22 review are resolved or explicitly deferred above
- [x] File paths in PLAN verified against planned directory structure
- [x] Status: Approved — ready for Phase Prereq execution
