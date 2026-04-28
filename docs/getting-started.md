# Getting Started with Clanker Yap

This guide covers the current supported path for Clanker Yap: **running on Linux**.

## Platform status

Current release target:

- **Linux x86_64 AppImage**

Tested environments for the 0.1.0 release target:

- **Wayland:** smoke tested
- **X11:** smoke tested

Not yet released:

- macOS
- Windows

## System requirements

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

Also required:

- Rust stable toolchain
- Node.js and npm

## Installation

### 1. Install Rust

```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

### 2. Clone and install dependencies

```sh
git clone https://github.com/Bynzski/clanker-yap.git
cd clanker-yap
npm install
```

### 3. Get a Whisper model

Clanker Yap expects a local GGML Whisper model.

Default path:

```text
~/.local/share/voice-transcribe/ggml-base.en.bin
```

You can either:

- download the default model from inside the app, or
- place one there manually from [whisper.cpp on Hugging Face](https://huggingface.co/ggml-org/whisper.cpp/tree/main)

## Running the app

### Development

```sh
npm run tauri:dev
```

### Production AppImage build

```sh
npm run tauri:build
```

Output:

```text
src-tauri/target/release/bundle/appimage/Clanker Yap_0.1.0_amd64.AppImage
```

Run it with:

```sh
chmod +x "src-tauri/target/release/bundle/appimage/Clanker Yap_0.1.0_amd64.AppImage"
./src-tauri/target/release/bundle/appimage/Clanker\ Yap_0.1.0_amd64.AppImage
```

## First launch

On first launch, the app will:

1. initialize the SQLite database
2. set the default hotkey to `Ctrl+Shift+V`
3. use the default model path unless you change it
4. request microphone permission if your environment requires it

## Basic usage

1. Open any app where you want text inserted
2. Hold the configured hotkey
3. Speak into your microphone
4. Release the hotkey
5. Clanker Yap transcribes locally and pastes the result

## What you can configure

- **Hotkey** — push-to-talk shortcut
- **Model** — local Whisper model path
- **Mic** — audio input device
- **Paste mode** — Auto, Standard, or Terminal
- **History** — recent transcriptions

## Need more help?

- [Quick Start](./quick-start.md)
- [Configuration](./configuration.md)
- [Whisper Models](./whisper-models.md)
- [Troubleshooting](./troubleshooting.md)
- [Build & Release](./build.md)
