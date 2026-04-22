# Voice Transcription App — README

A lightweight, single-purpose desktop app for local voice-to-text dictation using whisper.cpp.

## Quick Links

- [Plan Document](./PLAN.md) — Full implementation plan (v2.2)
- [Progress](./PROGRESS.md) — Phase tracking
- [Agent Prompt](./AGENT-PROMPT.md) — Per-phase execution prompt
- [Checklist](./CHECKLIST.md) — Plan review record

## Overview

```
Hotkey Pressed (down) → Record → Hotkey Released (up) → Transcribe (whisper.cpp) → Paste → Save
```

Features:
- **100% local** — no network calls for transcription
- **Low resource** — `base.en` model default, fine on old hardware
- **Push-to-talk** — global hotkey (default: `Cmd/Ctrl+Shift+V`, held while speaking)
- **Auto-paste** — text injected into focused input via `enigo` (works on macOS, Windows, Linux X11 + Wayland)
- **History** — last 10 transcriptions stored in SQLite
- **Single-instance** — duplicate launches focus the existing window

## Prerequisites

### Host build tools (run once before Phase Prereq)

Arch / CachyOS:
```sh
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 libappindicator-gtk3 librsvg nodejs npm
rustup default stable
npm install -g @tauri-apps/cli@latest
```

Ubuntu / Debian:
```sh
sudo apt install -y build-essential cmake pkg-config libssl-dev libasound2-dev \
  libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev libxdo-dev
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
npm install -g @tauri-apps/cli@latest
```

### Runtime

- Microphone access.
- **Linux Wayland paste fallback:** if `enigo`'s libei path is denied by your compositor, install `wtype` (or `ydotool`) — `enigo` picks these up automatically.
- **macOS:** Accessibility permission (System Settings → Privacy → Accessibility) for paste keystroke simulation. Microphone permission prompted on first launch.

### Model file

Download a GGML model from <https://huggingface.co/ggerganov/whisper.cpp/tree/main> and place it in the platform data dir:

| OS | Path |
|----|------|
| Linux | `~/.local/share/voice-transcribe/ggml-base.en.bin` |
| macOS | `~/Library/Application Support/voice-transcribe/ggml-base.en.bin` |
| Windows | `%APPDATA%\voice-transcribe\ggml-base.en.bin` |

| Model | Size | Notes |
|-------|------|-------|
| `ggml-base.en.bin` | ~148 MB | **Default** — fast, good for old hardware |
| `ggml-small.en.bin` | ~488 MB | Optional upgrade — better accuracy |

The app surfaces a "Model not found" banner with this link if the file is missing at startup.

## Phases

| Phase | Description | Status |
|-------|-------------|--------|
| Prereq | `git init`, host-deps check, Tauri scaffold, shared modules (`AppError`, `AppState`, constants, logging, paths) | 🔲 |
| 0 | Settings + SQLite + first-run bootstrap | 🔲 |
| 1 | whisper.cpp integration (lazy-loaded, `spawn_blocking`) | 🔲 |
| 2 | Audio capture — cpal worker thread + rubato resampling to 16 kHz mono f32 | 🔲 |
| 3 | Global hotkey (push-to-talk) + orchestration + live hotkey re-register | 🔲 |
| 4 | Frontend UI — status, settings (read + update), history | 🔲 |
| 5 | Text injection via `enigo` (cross-platform, no shell-outs) | 🔲 |

Phases 0/1/2/5 can run in parallel after Prereq. See [PLAN.md](./PLAN.md) "Implementation Plan → Phase Order" for dependencies.

## Running the App

```sh
# Development
cd src-tauri && cargo tauri dev

# Production build
cd src-tauri && cargo tauri build
```

## Architecture

```
src/                      # Frontend (vanilla HTML/JS/CSS, no bundler)
src-tauri/src/
├── domain/               # error.rs (AppError), constants.rs, settings, transcription
├── application/          # state.rs (AppState), orchestrator.rs, use_cases/
├── infrastructure/       # persistence/ (SQLite), whisper/, audio/ (cpal+rubato), paste/ (enigo)
└── presentation/         # Tauri commands + DTOs
```

## Status

- **Plan Version:** 2.2
- **Status:** Approved (ready for execution)
- **Created:** 2026-04-22
- **Last updated:** 2026-04-22 — gap-report pass: push-to-talk resolved, real whisper-rs API, cpal resampling, enigo replaces per-OS shell-outs, single-instance plugin, settings JSON schema, first-run bootstrap, unified `dirs::data_dir()` paths, pinned dep versions, host-deps checklist
