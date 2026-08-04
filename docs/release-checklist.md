# AppImage Release Checklist

## Pre-release

- [ ] Version matches in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`
- [ ] `CHANGELOG.md` contains the dated release section
- [ ] Release-facing documentation is current
- [ ] Release notes draft exists

## Install and build

```sh
npm ci
npm run release:verify
```

- [ ] All Rust tests pass
- [ ] Clippy passes with zero warnings
- [ ] Formatting is clean
- [ ] The AppImage exists in `src-tauri/target/release/bundle/appimage/`
- [ ] No other package format was produced by this build

## Smoke tests

- [ ] AppImage launches
- [ ] Main window renders correctly
- [ ] Model path or in-app model download works
- [ ] Global hotkey registers
- [ ] Hold-to-record and release-to-transcribe work
- [ ] Very short push-to-talk does not leave the app stuck
- [ ] Blank audio is not pasted or saved
- [ ] Overlay appears and hides correctly
- [ ] Editor paste works
- [ ] Terminal paste works
- [ ] Settings, history, and cumulative word count survive restart
- [ ] Wayland smoke test completed
- [ ] X11 smoke test completed when practical

## Publish

- [ ] Generate `SHA256SUMS.txt`
- [ ] Commit release preparation
- [ ] Tag and push the release
- [ ] Upload the AppImage and checksum if creating a GitHub release
- [ ] Verify the published download once
