# Changelog

All notable changes to Clanker Yap should be documented in this file.

The format is inspired by [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and follows the release rules in [RELEASING.md](./RELEASING.md).

## [Unreleased]

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
