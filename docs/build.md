# Build & Release Guide

This guide covers building and releasing Clanker Yap for **Linux** and **Windows**.

## Release targets

Clanker Yap ships as:

| Platform | Artifact | Built on |
|---|---|---|
| **Linux x86_64** | AppImage, `.deb` | Linux host |
| **Windows x64** | NSIS installer, MSI | Windows host |

Current status:
- **Linux (Wayland):** smoke tested
- **Linux (X11):** smoke tested
- **Windows:** smoke tested
- **macOS:** not yet supported as a release target

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

### Production build (Linux)

```sh
npm run tauri:build
```

> `tauri:build` already includes `NO_STRIP=1`, which is required on Arch/CachyOS for AppImage builds.

### Production build (Windows)

From a Windows machine, from repo root:

```powershell
npm ci
npx tauri build
```

> Windows installers are built on a Windows machine only. The Linux machine does not produce Windows artifacts.

## Output

Successful release builds produce:

**Linux:**
```text
src-tauri/target/release/bundle/appimage/Clanker Yap_X.Y.Z_amd64.AppImage
src-tauri/target/release/bundle/deb/Clanker Yap_X.Y.Z_amd64.deb
```

**Windows:**
```text
src-tauri/target/release/bundle/nsis/Clanker Yap_X.Y.Z_x64-setup.exe
src-tauri/target/release/bundle/msi/Clanker Yap_X.Y.Z_x64_en-US.msi
```

## Run the AppImage

```sh
chmod +x "src-tauri/target/release/bundle/appimage/Clanker Yap_X.Y.Z_amd64.AppImage"
./src-tauri/target/release/bundle/appimage/Clanker\ Yap_X.Y.Z_amd64.AppImage
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
- Windows

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

For each release, publish:

**Linux:**
- `.AppImage` file
- `.deb` file (optional)
- SHA256 checksum
- release notes

**Windows (built on Windows host):**
- NSIS `.exe` installer
- MSI `.msi` installer (optional)
- SHA256 checksum

Include notes on:
- Which platforms were smoke tested
- Known limitations

## Versioning

For each release, update the version in:

- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `package.json` (optional but recommended to keep aligned)

## Notes

- Clanker Yap supports both Linux and Windows desktop usage.
- Linux packaging is AppImage + deb.
- Windows packaging is NSIS + MSI.
- macOS support may be added later.
