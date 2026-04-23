# Quick Start Guide

Get Clanker Yap running in under 5 minutes.

## TL;DR

```sh
# 1. Install dependencies (Linux)
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 libappindicator-gtk3 librsvg nodejs npm

# 2. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable

# 3. Clone and setup
git clone https://github.com/your-username/clanker-yap.git
cd clanker-yap
npm install

# 4. Download model
mkdir -p ~/.local/share/voice-transcribe
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true" \
  -o ~/.local/share/voice-transcribe/ggml-base.en.bin

# 5. Run
npm run tauri dev
```

## What Just Happened?

Here's what each step does:

| Step | What It Does |
|------|--------------|
| `npm install` | Installs Tauri CLI and frontend dependencies |
| Model download | Fetches the Whisper GGML model (~148 MB) |
| `npm run tauri dev` | Compiles Rust, launches the app window |

## The Recording Flow

```
[HOLD HOTKEY] ──► Recording starts ──► [RELEASE HOTKEY]
                                              │
                                              ▼
                                    Audio is resampled to 16kHz
                                              │
                                              ▼
                                    Whisper runs locally
                                              │
                                              ▼
                              Text copied to clipboard
                                              │
                                              ▼
                           Ctrl+V simulates paste
```

## Testing It Works

1. Open a text editor (gedit, vscode, terminal, etc.)
2. Hold `Ctrl+Shift+V`
3. Say "Hello world"
4. Release the hotkey
5. You should see "hello world" appear in your text editor

## Status Panel

The app shows current state in the top panel:

```
┌─────────────────────────────────────────┐
│ ● Ready     System Default   ░░░░░░░░░ │
└─────────────────────────────────────────┘
```

| Visual | Meaning |
|--------|---------|
| Green dot | Ready to record |
| Pulsing bars | Actively recording |
| Spinning bars | Processing transcription |
| Checkmark | Done, text pasted |

## Need Help?

- Something broken? → [Troubleshooting](./troubleshooting.md)
- Want to customize? → [Configuration](./configuration.md)
- Want to contribute? → [CONTRIBUTING.md](../CONTRIBUTING.md)