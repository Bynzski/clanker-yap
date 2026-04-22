# Voice Transcription

Local voice-to-text desktop app built with Tauri v2, Rust, and a vanilla HTML/JS/CSS frontend.

Pipeline:

```text
Hold hotkey -> record -> release hotkey -> transcribe locally -> paste text
```

The app is currently set up for local Linux development on this machine. Packaging is limited to `.deb` bundles.

## Features

- Local transcription via `whisper-rs`
- Push-to-talk global hotkey
- Clipboard + simulated paste injection
- SQLite-backed settings and transcription history
- Single-instance app behavior

## Prerequisites

Linux packages:

```sh
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 libappindicator-gtk3 librsvg nodejs npm
```

Required tools:

- Rust stable toolchain
- Node.js and npm
- `cmake`
- system WebKit/AppIndicator libraries for Tauri on Linux

If Rust is not installed:

```sh
rustup default stable
```

Install project dependencies:

```sh
npm install
```

## Model File

The app expects a Whisper GGML model file at:

```text
~/.local/share/voice-transcribe/ggml-base.en.bin
```

Download one from:

- <https://huggingface.co/ggerganov/whisper.cpp/tree/main>

`ggml-base.en.bin` is the default expected model.

## Run

Development:

```sh
npm run tauri dev
```

The first launch may require microphone permission depending on platform/session configuration.

## Verify

Rust checks:

```sh
cargo check --manifest-path src-tauri/Cargo.toml
cargo test --manifest-path src-tauri/Cargo.toml
```

## Build

Debug app bundle for this machine:

```sh
npm run tauri build -- --debug
```

Current Linux bundle target:

- `.deb`

Expected output:

```text
src-tauri/target/debug/bundle/deb/Voice Transcription_0.1.0_amd64.deb
```

## Project Layout

```text
src/                    frontend
src-tauri/src/domain/   core types and errors
src-tauri/src/application/
src-tauri/src/infrastructure/
src-tauri/src/presentation/
plans/voice-transcribe/ plan and implementation notes
```

## Notes

- The repo still contains the original planning docs under `plans/voice-transcribe/`.
- Tauri bundle targets are intentionally narrowed to Linux `.deb` for faster local verification.
