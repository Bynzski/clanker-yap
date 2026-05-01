# AGENTS.md — Clanker Yap

## What This App Is

**Clanker Yap** is a local, privacy-first voice-to-text dictation app built with **Tauri v2** (Rust backend, vanilla JS frontend). It captures audio from a microphone, transcribes it locally using **whisper.cpp** (via `whisper-rs`), and injects the resulting text into whatever application has focus — all without sending audio to any cloud service.

The user holds a global push-to-talk hotkey (default `CmdOrCtrl+Shift+V`), speaks, releases, and the app runs: **record → resample to 16 kHz → transcribe → clipboard paste → save history**.

---

## Repository Structure

```
clanker-yap/
├── src/                          # Frontend (vanilla JS, no framework, no bundler)
│   ├── index.html                # Single-page UI shell
│   ├── overlay.html              # Floating always-on-top recording overlay pill
│   ├── main.js                   # All frontend logic (invokes, event listeners, DOM)
│   ├── style.css                 # Full CSS (dark theme, monospace)
│   └── vendor/tauri.js           # Tauri v2 API shim (no npm required)
│
├── src-tauri/                    # Rust backend (Tauri v2)
│   ├── Cargo.toml                # Rust dependencies
│   ├── tauri.conf.json           # Tauri window/bundle config (main + overlay windows)
│   └── src/
│       ├── lib.rs                # Crate root — logging init, module declarations
│       ├── main.rs               # Tauri app bootstrap, shortcut registration, plugins
│       │
│       ├── domain/               # Pure types — no I/O, no external deps
│       │   ├── constants.rs      # App-wide constants (sample rates, limits, defaults)
│       │   ├── error.rs          # AppError enum, Result<T> alias
│       │   ├── settings.rs       # Settings struct, AudioInputSelection enum
│       │   └── transcription.rs  # Transcription entity with validation
│       │
│       ├── application/          # Use cases and shared state
│       │   ├── state.rs          # AppState (DB, settings, whisper, recorder, recording state)
│       │   ├── orchestrator.rs   # Hotkey press/release → full pipeline + overlay coordination
│       │   └── use_cases/
│       │       ├── transcribe.rs     # Load-or-get whisper engine, run transcription
│       │       ├── transcription.rs  # Save + prune transcription history
│       │       ├── settings.rs       # Get/update persisted settings
│       │       ├── paste.rs          # Clipboard + keystroke injection
│       │       └── model_download.rs # Download default GGML model via curl/wget
│       │
│       ├── infrastructure/       # External world integrations
│       │   ├── audio/
│       │   │   ├── device.rs     # Audio device enumeration (cpal)
│       │   │   ├── recorder.rs   # Dedicated thread recorder with crossbeam channels
│       │   │   ├── resample.rs   # Linear interpolation resampling to 16 kHz
│       │   │   └── eq.rs         # FFT-based frequency band extraction (EqState)
│       │   ├── whisper/
│       │   │   ├── engine.rs     # WhisperContext wrapper, lazy-load + cache
│       │   │   └── downloader.rs # curl/wget model download
│       │   ├── persistence/
│       │   │   ├── paths.rs      # Platform data directory resolution
│       │   │   ├── db.rs         # SQLite connection wrapper (rusqlite, bundled)
│       │   │   ├── settings_repo.rs    # Single-row JSON settings persistence
│       │   │   └── transcription_repo.rs # Transcription CRUD + prune
│       │   ├── paste/
│       │   │   └── service.rs    # Enigo keystroke injection (standard + terminal modes)
│       │   └── overlay.rs        # Floating always-on-top overlay window (thread-safe)
│       │
│       └── presentation/         # Tauri command handlers + DTOs
│           ├── dto.rs            # Request/response types for the frontend API
│           └── commands/
│               ├── settings_cmds.rs      # get/update/download settings
│               ├── transcription_cmds.rs # history + status
│               ├── audio_cmds.rs         # list audio inputs
│               └── window_cmds.rs        # drag, minimize, close, resize
│
└── plans/                        # Planning documents (ignore in normal work)
```

---

## Architecture: Slim Frontend, Skinny Routes, Heavy Backend

This is the cardinal rule. Violate it and the codebase rots.

### Frontend (`src/`) — Display Only

- **No business logic.** The frontend invokes Tauri commands and renders the response. That's it.
- **No data transformation.** No computing, filtering, or deriving values that the backend could provide.
- **No state management framework.** Plain JS variables (`settings`, `transcriptions`). The backend is the source of truth.
- **No bundler, no npm runtime.** Vanilla HTML + CSS + JS loaded directly by the Tauri webview. `vendor/tauri.js` is a shim for `window.__TAURI__`.
- The frontend listens to Tauri events (`recording-started`, `transcription-complete`, etc.) and updates the DOM. It never computes transcription results, audio formats, or device capabilities.

### Presentation Layer (`presentation/`) — Skinny Routes

- Tauri command handlers are **thin adapters**: deserialize the request, call one use case, serialize the response.
- **No business logic in commands.** Validation, orchestration, and side effects live in `application/` and `domain/`.
- Command handlers may access `AppState` but must not contain branching logic beyond "call use case, return result or error."

### Backend (`domain/` + `application/` + `infrastructure/`) — Heavy

- **Domain** (`domain/`): Pure types, validation, constants, error definitions. Zero I/O. Zero external dependencies beyond `serde`, `chrono`, `uuid`.
- **Application** (`application/`): Use cases that coordinate between domain and infrastructure. `AppState` holds all shared resources. The `orchestrator` wires the full record→transcribe→paste→save pipeline.
- **Infrastructure** (`infrastructure/`): All I/O — SQLite, whisper.cpp, cpal audio, enigo paste, HTTP downloads. Implements traits/contracts expected by the application layer.

---

## Commands

### Build & Run

```bash
# Development (hot reload frontend, compile Rust in debug)
npm run tauri:dev

# Production build — Linux host builds Linux targets (appimage, deb)
# Windows targets (nsis, msi) are built on a Windows machine (see below)
npm run tauri:build
```

### Tauri Build Notes for CachyOS / Arch Linux

When building Tauri AppImages on Arch-based systems, always use:

```bash
NO_STRIP=1 npx tauri build
```

`NO_STRIP=1` is required because `linuxdeploy`'s bundled `strip` is too old for Arch's newer ELF format (`.relr.dyn` sections). Without it, the build fails with "unknown type \[0x13\] section `.relr.dyn'" errors on nearly every shared library.

A warning about `__TAURI_BUNDLE_TYPE` not being found may appear when the `tauri-cli` and Rust `tauri` crate versions are slightly mismatched. This is harmless unless the app uses Tauri's auto-updater. Keep CLI and crate versions aligned before enabling updater-related release workflows.

> **Agents:** Do not run plain `npx tauri build` on Arch/CachyOS. Always use `NO_STRIP=1 npx tauri build` or `npm run tauri:build`.

### Rust Backend (from `src-tauri/`)

```bash
# Compile check (fast, no codegen)
cargo check

# Run all tests
cargo test

# Run a specific test
cargo test domain::transcription::tests

# Run tests with output
cargo test -- --nocapture

# Clippy (lint — MUST pass with zero warnings)
cargo clippy -- -D warnings

# Format check (MUST pass)
cargo fmt --check

# Auto-format
cargo fmt

# Build in release mode
cargo build --release
```

### Frontend

No build step. Edit `src/*.html`, `src/*.css`, `src/*.js` directly. Tauri dev server serves them with `frontendDist: "../src"`.

---

## Testing Requirements

### Mandatory Gate: Every Run Ends Green

**Every agent run must finish with all of these passing. No exceptions.**

```bash
cd src-tauri

# 1. All tests must pass
cargo test

# 2. Clippy must pass with zero warnings (denied)
cargo clippy -- -D warnings

# 3. Formatting must be clean
cargo fmt --check
```

If any of these fail, the run is incomplete. Fix the issue and re-run until all three pass.

### Existing Tests

Tests live alongside the code they test in `#[cfg(test)] mod tests` blocks:

| Location | What it tests |
|---|---|
| `domain/error.rs` | AppError serializes to display string |
| `domain/transcription.rs` | Text length validation, duration range validation |
| `infrastructure/audio/resample.rs` | 48→16 kHz resampling, passthrough at same rate |

### When Adding Code

- **New domain types** (entities, value objects) → add validation tests in the same file.
- **New use cases** → test the use case function with mock inputs where feasible.
- **New infrastructure** → at minimum, test the pure/data-transformation parts (resampling, parsing, serialization).
- **Bug fixes** → add a regression test that reproduces the bug and proves the fix.
- Do not add integration tests that require hardware (microphone, whisper model) unless explicitly asked. Test the logic around the hardware instead.

---

## Key Domain Rules

- **Transcription text** must not exceed 10,000 characters.
- **Recording duration** must be between 150ms and 60,000ms.
- **History** is pruned to the 10 most recent entries after each save.
- **Whisper** requires 16 kHz mono f32 samples. The recorder thread handles downmix and resampling.
- **Settings** are stored as a single JSON row in SQLite (`app_settings` table, always `id = 1`).
- **Hotkey** changes are validated by attempting to re-register the global shortcut. On failure, the previous hotkey is restored (rollback).
- **Audio device** selection supports system default or exact name match. If the named device disappears, it falls back to system default with a warning.

---

## Dependencies (Rust)

| Crate | Purpose |
|---|---|
| `tauri` 2 | App framework, IPC, window management |
| `tauri-plugin-global-shortcut` 2 | Push-to-talk hotkey |
| `tauri-plugin-clipboard-manager` 2 | Clipboard write before paste |
| `tauri-plugin-single-instance` 2 | Prevent duplicate app launches |
| `whisper-rs` 0.16 | Rust bindings for whisper.cpp |
| `cpal` 0.15 | Cross-platform audio capture |
| `rubato` 0.15 | Sample rate conversion (resampling) |
| `realfft` 3 | Pure-Rust FFT for frequency band extraction (EQ visualization) |
| `rusqlite` 0.32 (bundled) | SQLite persistence |
| `enigo` 0.2 | Cross-platform keystroke injection |
| `parking_lot` 0.12 | Efficient mutex for shared state |
| `crossbeam-channel` 0.5 | Audio recorder thread communication |
| `serde` / `serde_json` | Serialization |
| `uuid` 1 | Transcription IDs (v4) |
| `chrono` 0.4 | Timestamps |
| `thiserror` 1 | Error derives |
| `tracing` / `tracing-subscriber` | Structured logging |
| `gtk-layer-shell` 0.8 | GTK Layer Shell for Wayland overlay positioning (Linux only) |

---

## Windows Build Workflow

The codebase supports Windows (`nsis`, `msi`) and Linux (`appimage`, `deb`) bundle targets. Artifact generation is **Linux-host only for Linux targets, Windows-host only for Windows targets**.

### Operating Model

| Machine | Role |
|---|---|
| **Linux (this machine)** | Development, prep, Rust validation (`cargo test`, `cargo clippy`, `cargo fmt`). Builds `appimage` and `deb` locally. |
| **GitHub Actions** | Validates the Rust backend compiles on `windows-latest` (no artifact produced). |
| **Windows machine** | Builds `nsis` and `msi` installers from a tagged commit. |

**Windows installers are not built on this Linux machine.**

### CI — Windows Validation (GitHub Actions)

A workflow on `windows-latest` runs the standard Rust validation gate:
```bash
cd src-tauri
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```
This confirms the backend compiles cleanly on Windows but does not produce an installer artifact.

### Building Windows Installers (Windows host)

From a Windows machine, from repo root:

```powershell
npm ci
npx tauri build
```

Expected outputs:
- `src-tauri/target/release/bundle/nsis/` — NSIS installer
- `src-tauri/target/release/bundle/msi/` — MSI installer

Smoke-test the installer before publishing.

---

## Conventions

- **Error handling:** Every fallible function returns `Result<T>` (= `std::result::Result<T, AppError>`). No unwraps in library code. Only `main.rs` may `.expect()`.
- **State access:** `AppState` uses `parking_lot::Mutex`. Lock, do work, drop the guard — never hold across an await or I/O boundary.
- **Blocking work:** Whisper transcription and audio collection run on `tauri::async_runtime::spawn_blocking`. Never block the Tauri main thread.
- **Module organization:** Follow the domain → application → infrastructure → presentation layering. Dependencies point inward (presentation → application → domain ← infrastructure).
- **Logging:** Use `tracing::{info, warn, error, debug}` with structured fields. Filter via `RUST_LOG` env var.
- **Frontend events:** Backend emits events to the frontend (`app.emit(...)`). Frontend listens with `listen()`. Never poll.
