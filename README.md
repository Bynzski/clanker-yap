# Clanker Yap

Local voice-to-text desktop application built with Tauri v2, Rust, and a vanilla HTML/JS/CSS frontend.

## Documentation

Full documentation is available in the [`docs/`](docs/) folder:

| Document | Description |
|----------|-------------|
| [docs/README.md](docs/README.md) | Documentation index |
| [docs/getting-started.md](docs/getting-started.md) | Installation and setup |
| [docs/quick-start.md](docs/quick-start.md) | Get running in 5 minutes |
| [docs/configuration.md](docs/configuration.md) | All configuration options |
| [docs/whisper-models.md](docs/whisper-models.md) | Model selection guide |
| [docs/architecture.md](docs/architecture.md) | Project structure and design |
| [docs/development.md](docs/development.md) | Local development guide |
| [docs/build.md](docs/build.md) | Building releases |
| [docs/troubleshooting.md](docs/troubleshooting.md) | Common issues and solutions |
| [docs/faq.md](docs/faq.md) | Frequently asked questions |
| [docs/commands.md](docs/commands.md) | Tauri command reference |
| [docs/api.md](docs/api.md) | Internal Rust API docs |

## Pipeline

```
Hold hotkey -> record -> release hotkey -> transcribe locally -> paste text
```

The app is currently configured for local Linux development and `.deb` packaging.

## Features

- Local transcription via `whisper-rs`
- Push-to-talk global hotkey
- Clipboard + simulated paste injection, including terminal-friendly paste mode
- SQLite-backed settings and transcription history
- Single-instance app behavior
- Built-in download flow for the default `ggml-base.en.bin` model

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

```
~/.local/share/voice-transcribe/ggml-base.en.bin
```

Download one from:

- [ggml-org/whisper.cpp](https://huggingface.co/ggml-org/whisper.cpp/tree/main)

`ggml-base.en.bin` is the default expected model.

The repository does not include:

- Whisper model files
- local SQLite databases
- build artifacts
- `node_modules`

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

```
src-tauri/target/debug/bundle/deb/Clanker Yap_0.1.0_amd64.deb
```

## Project Layout

```
src/                    frontend
src-tauri/src/          Rust backend
├── domain/             core types and errors
├── application/        use cases and state
├── infrastructure/     external integrations
│   ├── audio/         recording and resampling
│   ├── whisper/       ML transcription
│   ├── paste/        clipboard injection
│   └── persistence/   SQLite storage
└── presentation/      Tauri commands
```

## License

See [LICENSE](./LICENSE)

## Contributing

See [CONTRIBUTING.md](./CONTRIBUTING.md)

## Code of Conduct

See [CODE_OF_CONDUCT.md](./CODE_OF_CONDUCT.md)

## Security

See [SECURITY.md](./SECURITY.md)