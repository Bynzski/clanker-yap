# Build & Release Guide

This guide covers the current **0.1.0 Linux AppImage** release target for Clanker Yap.

## Release target

Clanker Yap currently ships as a:

- **Linux x86_64 AppImage**

Current status:
- **Wayland:** smoke tested
- **X11:** smoke tested
- **macOS:** not yet supported as a release target
- **Windows:** not yet supported as a release target

## Prerequisites

Install:

- Rust stable
- Node.js and npm
- Tauri CLI dependencies from `npm install`

### Linux packages

#### Arch / CachyOS

```sh
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 libappindicator-gtk3 librsvg nodejs npm
```

#### Debian / Ubuntu

```sh
sudo apt install build-essential cmake libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev nodejs npm
```

#### Fedora

```sh
sudo dnf install gcc-c++ cmake webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel nodejs npm
```

## Build commands

### Development

```sh
npm run tauri:dev
```

### Production AppImage

```sh
npm run tauri:build
```

> `tauri:build` already includes `NO_STRIP=1`, which is required on Arch/CachyOS for AppImage builds.

## Output

Successful release builds produce:

```text
src-tauri/target/release/bundle/appimage/Clanker Yap_0.1.0_amd64.AppImage
```

## Run the AppImage

```sh
chmod +x "src-tauri/target/release/bundle/appimage/Clanker Yap_0.1.0_amd64.AppImage"
./src-tauri/target/release/bundle/appimage/Clanker\ Yap_0.1.0_amd64.AppImage
```

## Release verification

Before publishing, verify:

1. `cd src-tauri && cargo test`
2. `cd src-tauri && cargo clippy -- -D warnings`
3. `cd src-tauri && cargo fmt --check`
4. `npm run tauri:build`
5. Launch the AppImage manually
6. Test the push-to-talk workflow in a real app
7. Confirm the model can be downloaded or loaded from disk
8. Confirm settings, history, and cumulative word count persist across restart

## Recommended smoke test matrix

### Session types

- Wayland
- X11

### User flows

- App launches from the AppImage
- Global hotkey registers successfully
- Hold-to-record / release-to-transcribe works
- Text pastes into a text editor
- Text pastes into a terminal using terminal paste mode
- Overlay appears and hides correctly
- Microphone selection works
- Model download/setup works
- Restart preserves settings and history

## Release artifacts

For a 0.1.0 release, publish:

- `Clanker Yap_0.1.0_amd64.AppImage`
- SHA256 checksum
- release notes
- a short note that Linux AppImage is the only supported release target for 0.1.0
- a short note that Wayland and X11 were smoke tested

## Versioning

For each release, update the version in:

- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `package.json` (optional but recommended to keep aligned)

## Notes

- Clanker Yap is currently optimized for Linux desktop usage.
- Packaging is AppImage-first for 0.1.0.
- Future macOS and Windows support can be added later, but they are not part of this release.
