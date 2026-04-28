# Frequently Asked Questions

Common questions about Clanker Yap.

## General

### What is Clanker Yap?

Clanker Yap is a local voice-to-text transcription application. It converts your spoken words into text that is automatically pasted into any application. All processing happens on your machine using the Whisper machine learning model.

### How does it work?

1. **Hold** a configurable hotkey to start recording
2. **Speak** into your microphone — a floating overlay pill appears showing FFT audio bars
3. **Release** the hotkey to stop recording — overlay shows processing state
4. **Audio** is transcribed locally using Whisper
5. **Text** is automatically pasted into your active application
6. **Overlay** disappears after transcription completes

### Is my audio sent to the cloud?

**No.** All processing happens locally on your machine. Your voice recordings never leave your device.

### What languages are supported?

- **English models** (`.en` suffix) - Optimized for English only
- **Multilingual models** - Support 99+ languages

Choose a model based on your language needs.

### What's the default hotkey?

The default hotkey is `CmdOrCtrl+Shift+V`:
- **Linux/Windows:** `Ctrl+Shift+V`
- **macOS:** `Cmd+Shift+V`

### Can I use a different hotkey?

Yes. Open the app, click the **Hotkey** toolbar button, and follow the instructions to set a new shortcut.

## Technical

### What models are supported?

GGML-format Whisper models from [ggml-org/whisper.cpp](https://huggingface.co/ggml-org/whisper.cpp).

| Model | Size | Best For |
|-------|------|----------|
| base.en | 148 MB | Fast, English-only |
| small.en | 466 MB | Balanced accuracy/speed |
| medium.en | 1.5 GB | High accuracy |
| large | 2.9 GB | Maximum accuracy |

### Do I need a GPU?

**No.** Clanker Yap runs entirely on CPU. No GPU is required.

### What audio formats are supported?

- Input: Any format your microphone supports
- Internal: 16kHz 16-bit mono (required by Whisper)
- The app automatically resamples audio to the correct format.

### What does the recording overlay show?

A floating pill appears at the bottom of the screen while recording:

- **7 FFT frequency bars** — react to your voice in real time (low bars = bass, high bars = treble)
- **Recording state** — green/cyan gradient bars with a pulsing dot
- **Processing state** — amber pulsing bars when transcription is running

The overlay is always-on-top, click-through, and transparent.

### How much memory does it use?

Approximate memory usage:
- **Model (base):** ~150 MB
- **Audio buffer:** ~1 MB per recording
- **App overhead:** ~50 MB

### Can I use a different microphone?

Yes. Go to **Mic** in the toolbar to select a specific audio device, or use the system default.

## Usage

### How do I use it?

1. Ensure a Whisper model is downloaded (see [Getting Started](./getting-started.md))
2. Start the app
3. Hold the hotkey and speak
4. Release when done
5. Text is automatically pasted

### Can I use it in a terminal?

Yes. Set the paste mode to "Terminal" in settings for better terminal compatibility.

### How do I review past transcriptions?

Click **History** in the toolbar to see recent transcriptions.

### Can I copy text without pasting?

Yes. The transcribed text is also copied to your clipboard. You can paste manually with Ctrl+V.

### What if the app isn't responding?

1. Wait for current transcription to complete
2. If still unresponsive, close and restart the app
3. Check logs with `RUST_LOG=trace npm run tauri:dev`

## Configuration

### Where is settings stored?

Settings are stored in a SQLite database:
- **Linux:** `~/.local/share/voice-transcribe/voice-transcribe.db`
- **macOS:** `~/Library/Application Support/dev.jay.voice-transcribe/voice-transcribe.db`
- **Windows:** `%APPDATA%\dev.jay.voice-transcribe\voice-transcribe.db`

### Can I use a custom model path?

Yes. Click **Model** in the toolbar and enter a custom path.

### What's the difference between paste modes?

| Mode | How it works |
|------|-------------|
| Auto | Standard paste, fallback to keyboard |
| Standard | Direct clipboard paste |
| Terminal | Type character-by-character |

### How do I reset settings?

Delete the database file and restart the app:
```sh
rm ~/.local/share/voice-transcribe/voice-transcribe.db
```

## Troubleshooting

### Hotkey doesn't work

1. Ensure no other app is using the same shortcut
2. Try a different hotkey
3. Check app permissions
4. See [Troubleshooting](./troubleshooting.md)

### Microphone not detected

1. Check microphone is connected
2. Test microphone in system settings
3. Try selecting specific device in app
4. See [Troubleshooting](./troubleshooting.md)

### Model won't load

1. Verify model file exists at configured path
2. Download model if missing
3. Check file permissions
4. See [Troubleshooting](./troubleshooting.md)

### Text not pasting

1. Try "Terminal" paste mode
2. Check target app accepts paste
3. Manually paste with Ctrl+V
4. See [Troubleshooting](./troubleshooting.md)

## Development

### Is this open source?

Yes. Clanker Yap is open source. See [LICENSE](../LICENSE).

### Can I contribute?

Yes! See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

### What technologies are used?

| Layer | Technology |
|-------|------------|
| Frontend | Vanilla HTML/CSS/JS |
| Backend | Rust + Tauri v2 |
| ML | whisper-rs (whisper.cpp) |
| Database | SQLite (rusqlite) |
| Audio | cpal + hound |

### How is the project structured?

See [Architecture](./architecture.md) for detailed project structure.

## Privacy

### Does the app collect data?

**No.** Clanker Yap does not:
- Send audio to any server
- Collect usage statistics
- Track your transcriptions (except locally)
- Report errors externally

### Is my transcription history private?

Transcriptions are stored locally in the SQLite database. No data is sent externally.

## Support

### Where can I get help?

1. Read the [docs](./README.md)
2. Check [Troubleshooting](./troubleshooting.md)
3. Search existing [issues](https://github.com/your-username/clanker-yap/issues)
4. Create a new issue with details

### How do I report a bug?

1. Check existing issues
2. Run with `RUST_LOG=trace` to capture logs
3. Create issue with:
   - Platform and OS version
   - Steps to reproduce
   - Expected vs actual behavior
   - Log output

### How do I request a feature?

Create a feature request issue with:
- Description of the feature
- Use case / why you need it
- Any implementation suggestions

## Future

### Will there be a macOS/Windows build?

Possibly later, but not for the 0.1.0 release target.

Right now Clanker Yap ships as a Linux AppImage and has been smoke tested on both Wayland and X11. macOS and Windows builds are not yet part of the supported release workflow.

### Can I use a larger model?

Yes. Download any GGML Whisper model and set the path in settings.

### Will GPU support be added?

Currently CPU-only. GPU acceleration could be added but is not planned for v0.1.0.

### Can I use WhisperX or other backends?

No. Clanker Yap uses whisper-rs specifically. Other backends would require significant changes.