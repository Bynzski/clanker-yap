# Getting Started with Clanker Yap

This guide walks you through installing and running Clanker Yap on your system.

## System Requirements

### Linux

**Required packages:**
```sh
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 libappindicator-gtk3 librsvg nodejs npm
```

For **Debian/Ubuntu**-based distributions:
```sh
sudo apt install build-essential cmake libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev nodejs npm
```

For **Fedora/RHEL**:
```sh
sudo dnf install gcc-c++ cmake webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel nodejs npm
```

**Required tools:**
- Rust stable toolchain
- Node.js (v16+) and npm

### macOS

Install Rust via [rustup](https://rustup.rs/), then:
```sh
brew install cmake webkit2gtk@4.1
```

### Windows

Install the following via [scoop](https://scoop.sh/) or manually:
- Rust
- Visual Studio Build Tools (for WebView2)
- Node.js and npm

## Installation

### 1. Install Rust

```sh
# If Rust is not installed
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Set stable toolchain as default
rustup default stable
```

### 2. Clone and Setup

```sh
git clone https://github.com/your-username/clanker-yap.git
cd clanker-yap
npm install
```

### 3. Download Whisper Model

The application needs a GGML-format Whisper model for transcription.

**Download the default model:**
```sh
mkdir -p ~/.local/share/voice-transcribe
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true" \
  -o ~/.local/share/voice-transcribe/ggml-base.en.bin
```

**Model locations checked in order:**
1. Custom path set in settings
2. `~/.local/share/voice-transcribe/ggml-base.en.bin`

### 4. Run the Application

**Development mode:**
```sh
npm run tauri dev
```

**Production build:**
```sh
npm run tauri build
```

The built application will be at:
```
src-tauri/target/release/bundle/deb/Clanker Yap_0.1.0_amd64.deb
```

## First Launch

On first launch, the app will:

1. Initialize the SQLite database (~/.local/share/voice-transcribe/voice-transcribe.db)
2. Set default hotkey to `Ctrl+Shift+V`
3. Configure default model path
4. Request microphone permission if needed

## Usage

### Basic Operation

1. **Hold** the configured hotkey (default: `Ctrl+Shift+V`)
2. **Speak** into your microphone
3. **Release** the hotkey
4. The transcribed text is **automatically pasted** into your active application

### Status Indicators

| Status | Meaning |
|--------|---------|
| Ready | App is idle, ready to record |
| Recording | Hotkey held, actively recording — overlay pill visible |
| Processing | Transcribing audio — overlay pill shows processing state |
| Done | Text copied and pasted |

### Settings Access

Click the toolbar icons to configure:
- **Hotkey** - Change push-to-talk shortcut
- **Model** - Select Whisper model file
- **Mic** - Choose audio input device
- **Paste** - Configure paste behavior (Auto/Standard/Terminal)
- **History** - View recent transcriptions

### Recording Overlay

When you hold the hotkey, a floating recording indicator pill appears at the bottom of your screen:

- **Recording state:** Shows 7 real-time FFT frequency bars that react to your voice
- **Processing state:** Shows an amber pulsing indicator while transcribing

The overlay is:
- Always on top of other windows
- Click-through (doesn't block mouse events)
- Transparent (doesn't obscure your work)
- Hidden automatically after transcription completes

## Next Steps

- Read the [Configuration Guide](./configuration.md) for detailed settings
- See [Architecture](./architecture.md) to understand the codebase
- Check [Troubleshooting](./troubleshooting.md) if you encounter issues

## Troubleshooting

**Microphone not detected?**
- Ensure your microphone is connected and working
- Check system audio settings
- Try selecting a specific device in the Mic settings

**Model download fails?**
- Check your internet connection
- Try an alternative mirror or download manually

**Hotkey doesn't work?**
- Ensure no other application is using the same shortcut
- Check that the app has input device permissions

See [Troubleshooting](./troubleshooting.md) for more solutions.