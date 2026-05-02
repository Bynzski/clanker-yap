# Release Checklist

Use this checklist when preparing a release of Clanker Yap.

## Release targets

| Platform | Artifact | Built on |
|---|---|---|
| Linux x86_64 | AppImage, `.deb` | Linux host |
| Windows x64 | NSIS, MSI | Windows host |

Tested:
- Wayland
- X11
- Windows

Not included:
- macOS release

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

## Build (Linux)

```sh
npm run tauri:build
```

- [ ] AppImage builds successfully
- [ ] `.deb` builds successfully
- [ ] Artifacts exist at `src-tauri/target/release/bundle/`

## Build (Windows — on Windows host)

```powershell
npm ci
npx tauri build
```

- [ ] NSIS installer builds successfully
- [ ] MSI builds successfully
- [ ] Artifacts exist at `src-tauri/target/release/bundle/nsis/` and `.../msi/`

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
- [ ] Windows smoke test completed

## Release artifacts

**Linux:**
- [ ] Upload `.AppImage`
- [ ] Upload `.deb` (optional)
- [ ] Generate SHA256 checksum

**Windows:**
- [ ] Upload NSIS `.exe` installer
- [ ] Upload MSI `.msi` installer (optional)
- [ ] Generate SHA256 checksum

**Common:**
- [ ] Include release notes
- [ ] Note which platforms were smoke tested
- [ ] Note known limitations

## Post-release

- [ ] Tag release in git
- [ ] Publish GitHub release
- [ ] Upload Windows artifacts from Windows host
- [ ] Verify download/install instructions once from the published release page
