# Release Checklist

Use this checklist when preparing a Linux AppImage release of Clanker Yap.

## 0.1.0 target

Release target:
- Linux x86_64 AppImage

Tested:
- Wayland
- X11

Not included in 0.1.0:
- macOS release
- Windows release

## Pre-release

- [ ] Confirm version in `src-tauri/tauri.conf.json`
- [ ] Confirm version in `src-tauri/Cargo.toml`
- [ ] Confirm README matches the current release target
- [ ] Confirm `docs/build.md` matches the current release target
- [ ] Confirm release notes draft exists

## Quality gate

```sh
cd src-tauri
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

- [ ] Tests pass
- [ ] Clippy passes with zero warnings
- [ ] Formatting check passes

## Build

```sh
npm run tauri:build
```

- [ ] AppImage builds successfully
- [ ] Artifact exists at `src-tauri/target/release/bundle/appimage/`

## Smoke tests

### General
- [ ] App launches from AppImage
- [ ] Main window renders correctly
- [ ] Settings can be opened and saved
- [ ] Model path is valid or in-app download works

### Dictation workflow
- [ ] Global hotkey registers
- [ ] Hold-to-record works
- [ ] Release triggers transcription
- [ ] Overlay appears during recording
- [ ] Overlay hides after completion
- [ ] Successful transcription pastes into a text editor
- [ ] Successful transcription pastes into a terminal using terminal mode
- [ ] Very short push-to-talk does not leave the app stuck

### Persistence
- [ ] Settings persist across restart
- [ ] History persists across restart
- [ ] Cumulative word count persists across restart

### Session coverage
- [ ] Wayland smoke test completed
- [ ] X11 smoke test completed

## Release artifacts

- [ ] Upload `.AppImage`
- [ ] Generate SHA256 checksum
- [ ] Include release notes
- [ ] Note Linux AppImage-only support for 0.1.0
- [ ] Note smoke testing on Wayland and X11

## Post-release

- [ ] Tag release in git
- [ ] Publish GitHub release
- [ ] Verify download/install instructions once from the published release page
