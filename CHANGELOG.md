# Changelog

All notable changes to Clanker Yap should be documented in this file.

The format is inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and follows the release rules in [RELEASING.md](./RELEASING.md).

## [Unreleased]

## [0.1.3] - 2026-05-01

### Added
- Windows CI workflow (`windows-latest`) validates Rust backend on every push/PR to `main`
- `nsis` and `msi` bundle targets in Tauri config for Windows installer builds
- Recording overlay pill now supported on Windows — shown during push-to-talk recording and processing, positioned at bottom-center of the primary monitor
- Terminal paste mode (`Ctrl+Shift+V` / `Shift+Insert`) now active on Windows, matching existing Linux behavior
- App icons tracked in git (required by `tauri-build` for Windows Resource file generation)

### Changed
- Overlay creation and all overlay calls gated behind `#[cfg(target_os = "linux")]` so the app compiles cleanly on Windows without GTK
- `spawn_level_emission_task` moved from orchestrator into overlay module (Linux-only code lives with Linux-only infrastructure)
- Linux-only dependency comments clarified in `Cargo.toml` with explicit Windows and macOS sections
- Closing the main app window now exits the entire Tauri process (including backend services), instead of only closing the visible window while hidden windows could keep the process alive
- Tauri dependency versions pinned exactly (`2.10.1`) rather than with caret ranges
- AGENTS.md updated with Windows Build Workflow section documenting the operating model

### Fixed
- Frontend rendering/runtime break caused by stray duplicate JavaScript lines after `getPasteModeDescription(...)` in `src/main.js`
- `dtolnay/rust-action` → `dtolnay/rust-toolchain` in CI workflow (incorrect action name)
- Unused-variable clippy errors on Windows for `window` (overlay) and `app` (shutdown) suppressed

## [0.1.2] - 2026-04-30

### Changed
- Paste controller now reuses a single persistent `Enigo` instance instead of creating a new one on every transcription, eliminating repeated KDE/Wayland "Remote Control — Control input devices" permission prompts
- `Enigo` is lazily initialised on first paste and held for the app lifetime — at most one permission prompt per session
- `auto_paste` setting now defaults to `true` on all platforms including Wayland
- If keyboard simulation init fails (e.g. permission denied), the controller gracefully falls back to clipboard-only without retrying on every subsequent paste
- Toggling `auto_paste` back on resets the controller, giving it a fresh initialisation attempt
- Paste controller is cleaned up on app shutdown

### Added
- `auto_paste` toggle in the Paste settings UI (allows users to opt out of automatic keyboard simulation)
- `PasteController` struct in `AppState` for persistent input-device session management
- `PasteOutcome` enum (`CopiedOnly` / `CopiedAndPasted`) to communicate paste status through the pipeline
- `clipboard_only` field in the `transcription-complete` event so the frontend can show appropriate status

### Fixed
- Repeated KDE/Wayland "Remote Control" permission prompts no longer appear after each transcription

## [0.1.1] - 2026-04-29

### Added
- Release process documentation in `RELEASING.md`
- Git hook workflow for commit-message validation and pre-push verification
- `docs/release-checklist.md` for Linux AppImage release prep

### Changed
- README and release-facing docs now reflect the Linux AppImage release target
- Documentation now explicitly notes Wayland and X11 smoke-test coverage
- Recorder lifecycle refactored to keep one long-lived CPAL input stream per worker instead of rebuilding per push-to-talk cycle
- Start/Stop now toggles recording state and buffer handling without dropping the stream (stream drops only on shutdown/worker exit)
- Architecture and troubleshooting docs updated to describe the new recorder stream lifecycle and Linux permission-prompt behavior

## [0.1.0] - 2026-04-27

### Added
- Local push-to-talk voice transcription desktop app built with Tauri v2 and Rust
- Local Whisper transcription via `whisper-rs` / `whisper.cpp`
- Global hotkey workflow for hold-to-record and release-to-transcribe
- Floating recording overlay with live mic level visualization
- Clipboard paste injection with terminal-friendly paste mode
- SQLite-backed settings and transcription history
- Built-in model download flow for the default Whisper model
- Single-instance app behavior
- Cumulative word count persisted across restarts

### Fixed
- Very short push-to-talk captures now resolve gracefully instead of surfacing an unrecoverable error
- Persisted cumulative word count now hydrates correctly on startup

### Notes
- Initial release target is Linux x86_64 AppImage
- Smoke tested on both Wayland and X11
- macOS and Windows are not yet supported release targets
