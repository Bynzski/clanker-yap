# Configuration Guide

Detailed explanation of all Clanker Yap configuration options.

## Configuration Storage

Settings are stored in a SQLite database at:
- **Linux**: `~/.local/share/voice-transcribe/voice-transcribe.db`
- **macOS**: `~/Library/Application Support/dev.jay.voice-transcribe/voice-transcribe.db`
- **Windows**: `%APPDATA%\dev.jay.voice-transcribe\voice-transcribe.db`

## Settings Schema

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `hotkey` | String | `CmdOrCtrl+Shift+V` | Global push-to-talk hotkey |
| `model_path` | String | `~/.local/share/voice-transcribe/ggml-base.en.bin` | Path to Whisper model |
| `model_name` | String | `ggml-base.en.bin` | Display name for the model |
| `paste_mode` | String | `auto` | Clipboard paste behavior (`auto`, `standard`, `terminal`) |
| `auto_paste` | Boolean | `true` | Whether to automatically simulate Ctrl+V after copying to clipboard |
| `audio_input` | JSON | `null` | Selected microphone device |
| `schema_version` | Integer | `1` | Settings format version |

## Hotkey Configuration

### Format

Hotkeys use Tauri accelerator format: `<Modifier>+<Key>`

**Valid modifiers:**
- `CmdOrCtrl` - Control on Windows/Linux, Command on macOS
- `Ctrl` - Control key
- `Alt` - Alt key
- `Shift` - Shift key
- `Super` - Windows key / Command key

**Requirements:**
- At least one modifier key is required
- Must include one non-modifier key

### Examples

| Hotkey | Description |
|--------|-------------|
| `CmdOrCtrl+Shift+V` | Default (Ctrl+Shift on Windows/Linux, Cmd+Shift on macOS) |
| `Alt+R` | Alt+R |
| `CmdOrCtrl+Alt+Space` | All modifiers plus space |

### Setting via UI

1. Click the **Hotkey** toolbar button
2. Click "Record shortcut"
3. Press your desired key combination
4. Click "Save"

## Model Configuration

### Supported Formats

Clanker Yap uses GGML-format Whisper models. These are `.bin` files containing quantized weights.

### Recommended Models

| Model | Size | Speed | Accuracy | Best For |
|-------|------|-------|----------|----------|
| `ggml-base.en.bin` | 148 MB | Fastest | Good | English-only, fast transcription |
| `ggml-small.en.bin` | 466 MB | Fast | Better | English-only, higher accuracy |
| `ggml-medium.en.bin` | 1.5 GB | Medium | Very Good | English-only, detailed capture |
| `ggml-base.bin` | 148 MB | Fastest | Good | Multilingual, base quality |
| `ggml-small.bin` | 466 MB | Fast | Better | Multilingual, balanced |

### Setting Custom Path

1. Click the **Model** toolbar button
2. Click "Edit"
3. Enter full path (e.g., `/home/user/models/whisper-base.bin`)
4. Click "Save"

### Downloading Models

**Via UI:**
1. Click "Download base.en" in the Model settings
2. Wait for download to complete
3. Model is saved to default location

**Manually:**
```sh
curl -L "MODEL_URL" -o ~/.local/share/voice-transcribe/MODEL_NAME.bin
```

Model URLs from [ggml-org/whisper.cpp](https://huggingface.co/ggml-org/whisper.cpp/tree/main) on HuggingFace.

## Audio Input Configuration

### Selection Types

```json
{
  "type": "system_default"  // Use system default device
}
```

```json
{
  "type": "by_name",
  "value": "USB Microphone: Audio (hw:1,0)"
}
```

### Setting via UI

1. Click the **Mic** toolbar button
2. Select device from dropdown
3. Click "Save"

### Finding Device Names

List available devices via the microphone dropdown in the UI, or check:
- **Linux**: `arecord -l` or `pactl list sources`
- **macOS**: System Settings > Sound > Input
- **Windows**: Sound settings

## Paste Mode Configuration

Controls how transcribed text is inserted into applications.

### Auto-paste

The `auto_paste` setting controls whether Clanker Yap automatically simulates
a Ctrl+V (or Cmd+V) keystroke after copying the transcription to the clipboard.

- **Enabled (default):** Text is copied to the clipboard and automatically pasted.
- **Disabled:** Text is only copied to the clipboard — press Ctrl+V manually.

On KDE/Wayland, the keyboard simulation uses a persistent input controller that
is initialised once per session. You may see a single "Remote Control" permission
prompt the first time auto-paste runs. Subsequent pastes reuse the same session
without additional prompts.

### Paste Shortcut Style

### Modes

| Mode | Behavior | Use Case |
|------|----------|----------|
| `auto` | Standard paste, fallback to keyboard simulation | Default, works in most apps |
| `standard` | Direct clipboard paste via Tauri | Most applications |
| `terminal` | Type character-by-character | Terminals, Vim, applications that block paste shortcuts |

### Comparison

| Aspect | Standard | Terminal |
|--------|----------|----------|
| Speed | Fast | Slow |
| Unicode | Supported | Limited |
| Special chars | Handled | May have issues |
| Clipboard | Modified | Not modified |

### Setting via UI

1. Click the **Paste** toolbar button
2. Select mode (Auto/Standard/Terminal)
3. Click "Save"

## Configuration Files

### tauri.conf.json

Main Tauri configuration file located at `src-tauri/tauri.conf.json`:

```json
{
  "productName": "Clanker Yap",
  "version": "0.1.0",
  "identifier": "dev.jay.voice-transcribe",
  "app": {
    "windows": [{
      "width": 480,
      "height": 196,
      "resizable": false
    }]
  }
}
```

**Key options:**
- `productName` - Window title and app name
- `version` - App version (used in builds)
- `identifier` - Unique app identifier (required for stores)
- `width`/`height` - Window dimensions
- `resizable` - Whether window can be resized

### Customizing Window Size

Edit `src-tauri/tauri.conf.json`:

```json
{
  "app": {
    "windows": [{
      "label": "main",
      "width": 600,
      "height": 400
    }]
  }
}
```

## Environment Variables

No environment variables are required. All configuration is stored in the SQLite database.

## Resetting to Defaults

To reset all settings:

1. Delete the database file:
   ```sh
   rm ~/.local/share/voice-transcribe/voice-transcribe.db
   ```
2. Restart the application

This will recreate the database with default values.