# Clanker Yap Documentation

Complete documentation for the Clanker Yap voice transcription application.

## Overview

Clanker Yap is a local voice-to-text desktop application that enables real-time transcription using the Whisper machine learning model. Your voice recordings never leave your machine, ensuring complete privacy.

## Documentation Index

### Getting Started
- **[Installation](./getting-started.md)** - System requirements and installation steps
- **[Quick Start](./quick-start.md)** - Get up and running in minutes

### Configuration
- **[Configuration Guide](./configuration.md)** - All configuration options explained
- **[Whisper Models](./whisper-models.md)** - Model selection and download guide

### Development
- **[Architecture](./architecture.md)** - Project structure and design decisions
- **[Development Guide](./development.md)** - Setting up for local development
- **[Build & Release](./build.md)** - Building releases for distribution

### Reference
- **[Command Reference](./commands.md)** - CLI and Tauri commands
- **[API Reference](./api.md)** - Internal Rust API documentation

### Help
- **[Troubleshooting](./troubleshooting.md)** - Common issues and solutions
- **[FAQ](./faq.md)** - Frequently asked questions

## Key Features

| Feature | Description |
|---------|-------------|
| **Local Processing** | All transcription happens on your machine via whisper-rs |
| **Push-to-Talk** | Global hotkey activates recording (default: `Ctrl+Shift+V`) |
| **Smart Paste** | Automatically pastes text with terminal-friendly modes |
| **History** | SQLite-backed storage for recent transcriptions |
| **Single Instance** | Ensures only one app instance runs at a time |

## Pipeline Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                        Push-to-Talk Pipeline                     │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│   [1] HOLD HOTKEY ──► [2] RECORD AUDIO ──► [3] RELEASE HOTKEY   │
│                            │                                      │
│                            ▼                                      │
│                    [4] RESAMPLE TO 16kHz                         │
│                            │                                      │
│                            ▼                                      │
│              [5] TRANSCRIBE VIA WHISPER.CPP                      │
│                            │                                      │
│                            ▼                                      │
│          [6] COPY TO CLIPBOARD + PASTE INJECTION                 │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

## Tech Stack

| Layer | Technology |
|-------|------------|
| **Frontend** | Vanilla HTML, CSS, JavaScript |
| **Backend** | Rust with Tauri v2 |
| **ML Engine** | whisper-rs (whisper.cpp bindings) |
| **Database** | SQLite (via rusqlite) |
| **Audio** | cpal + hound |
| **Clipboard** | tauri-plugin-clipboard-manager |

## Project Layout

```
clanker-yap/
├── src/                    # Frontend assets
│   ├── index.html          # Main UI
│   ├── main.js             # Frontend logic
│   └── style.css           # Styling
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── domain/         # Core types and errors
│   │   ├── application/    # Use cases and state
│   │   ├── infrastructure/ # External integrations
│   │   │   ├── audio/      # Recording and resampling
│   │   │   ├── whisper/    # ML transcription
│   │   │   ├── paste/      # Clipboard injection
│   │   │   └── persistence/# SQLite storage
│   │   └── presentation/   # Tauri commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                   # This documentation
└── README.md
```

## License

See [LICENSE](../LICENSE) for licensing details.

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for contribution guidelines.