# AppImage Build & Release Guide

Clanker Yap has one release target: a Linux x86_64 AppImage.

## Prerequisites

Install Rust stable, Node.js, npm, and the Linux packages required by Tauri.

### Arch / CachyOS

```sh
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 libappindicator-gtk3 librsvg nodejs npm
```

### Debian / Ubuntu build host

```sh
sudo apt install build-essential cmake libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev nodejs npm
```

### Fedora

```sh
sudo dnf install gcc-c++ cmake webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel nodejs npm
```

## Commands

Development:

```sh
npm run tauri:dev
```

AppImage build:

```sh
npm run tauri:build
```

Full verification and AppImage build:

```sh
npm run release:verify
```

The build script passes both `NO_STRIP=1` and `--bundles appimage`. `NO_STRIP=1` avoids the older `linuxdeploy` strip utility failing on Arch/CachyOS ELF files.

## Output

```text
src-tauri/target/release/bundle/appimage/Clanker Yap_X.Y.Z_amd64.AppImage
```

No Debian, Windows, or macOS packages are produced.

## Release verification

Before publishing:

1. Run `npm ci`.
2. Run `npm run release:verify`.
3. Launch the generated AppImage.
4. Test push-to-talk, transcription, editor paste, terminal paste, overlay behavior, model setup, settings, history, and word-count persistence.
5. Generate and verify a SHA-256 checksum.

See [`../RELEASING.md`](../RELEASING.md) and [`release-checklist.md`](./release-checklist.md).
