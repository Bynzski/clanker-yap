# Whisper Models Guide

Everything you need to know about choosing and configuring Whisper models.

## Overview

Clanker Yap uses [whisper.cpp](https://github.com/ggerganov/whisper.cpp) models via the whisper-rs Rust bindings. These are GGML-format models optimized for local inference.

## Model Files

### Format

GGML (Gate Generalized Model Language) is a tensor format optimized for:
- Fast inference on CPU
- Quantization support (INT4, INT8)
- No GPU required

### File Extension

`.bin` files contain:
- Model architecture metadata
- Quantized weights
- Vocabulary mapping

### Storage Location

Default location: `~/.local/share/voice-transcribe/`

## Available Models

### English-Only Models

| Model | File | Size | VRAM | Speed | WER* |
|-------|------|------|------|-------|------|
| Tiny | `ggml-tiny.en.bin` | 75 MB | ~1 GB | Fastest | ~8% |
| Base | `ggml-base.en.bin` | 148 MB | ~1 GB | Fast | ~5% |
| Small | `ggml-small.en.bin` | 466 MB | ~2 GB | Medium | ~4% |
| Medium | `ggml-medium.en.bin` | 1.5 GB | ~3 GB | Slow | ~3% |
| Large | `ggml-large.bin` | 2.9 GB | ~4 GB | Slowest | ~2.5% |

*Word Error Rate on clean speech

### Multilingual Models

| Model | File | Size | Languages |
|-------|------|------|-----------|
| Tiny | `ggml-tiny.bin` | 75 MB | 99+ |
| Base | `ggml-base.bin` | 148 MB | 99+ |
| Small | `ggml-small.bin` | 466 MB | 99+ |
| Medium | `ggml-medium.bin` | 1.5 GB | 99+ |
| Large-v2 | `ggml-large-v2.bin` | 2.9 GB | 99+ |
| Large-v3 | `ggml-large-v3.bin` | 2.9 GB | 99+ |

## Choosing a Model

### Decision Factors

```
┌─────────────────────────────────────────────────────────────┐
│                    Model Selection Flow                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│   English only? ──YES──► Use ".en" models (smaller)         │
│         │                                                        │
│         NO                                                       │
│         │                                                        │
│         ▼                                                        │
│   Need speed? ──YES──► Base or Small                          │
│         │                                                        │
│         NO                                                       │
│         │                                                        │
│         ▼                                                        │
│   Need accuracy? ──YES──► Medium or Large                     │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Recommendations by Use Case

| Use Case | Recommended | Rationale |
|----------|-------------|-----------|
| Daily dictation | `ggml-base.en` | Fast, good accuracy, 148 MB |
| Developer (code) | `ggml-small.en` | Better accuracy for technical terms |
| Transcription | `ggml-medium.en` | High accuracy, reasonable speed |
| Non-English | `ggml-base` or `ggml-small` | Multilingual support |
| Research | `ggml-large` | Maximum accuracy |

## Downloading Models

### Via Application UI

1. Open Clanker Yap
2. Click the **Model** toolbar button
3. Click "Download base.en"
4. Wait for completion (~148 MB)

### Via Command Line

**Using curl:**
```sh
# Create directory
mkdir -p ~/.local/share/voice-transcribe

# Download base English model
curl -L "https://huggingface.co/ggml-org/whisper.cpp/resolve/main/ggml-base.en.bin?download=true" \
  -o ~/.local/share/voice-transcribe/ggml-base.en.bin

# Download small English model
curl -L "https://huggingface.co/ggml-org/whisper.cpp/resolve/main/ggml-small.en.bin?download=true" \
  -o ~/.local/share/voice-transcribe/ggml-small.en.bin
```

**Using wget:**
```sh
wget -O ~/.local/share/voice-transcribe/ggml-base.en.bin \
  "https://huggingface.co/ggml-org/whisper.cpp/resolve/main/ggml-base.en.bin?download=true"
```

### HuggingFace Mirrors

Official source: https://huggingface.co/ggml-org/whisper.cpp/tree/main

Alternative mirrors may be available. Search for "ggml whisper" repositories on GitHub for community mirrors.

## Model Validation

### Verify Download

Check file size and hash:

```sh
# Check file size (should be ~148 MB for base.en)
ls -lh ~/.local/share/voice-transcribe/ggml-base.en.bin

# Verify SHA256 (optional)
sha256sum ~/.local/share/voice-transcribe/ggml-base.en.bin
```

### Test Model in App

1. Open Clanker Yap
2. Ensure model path is set correctly in Model settings
3. Hold hotkey and speak
4. Verify transcription appears

### Error: "Model not found"

This means the model file doesn't exist at the configured path.

**Solutions:**
1. Verify file exists: `ls -la ~/.local/share/voice-transcribe/`
2. Check Settings model_path matches actual file location
3. Re-download if corrupted

## Custom Model Paths

### Setting Custom Path

1. Click **Model** in toolbar
2. Click "Edit"
3. Enter full path to your model file
4. Click "Save"

### Example Custom Paths

| Path | Description |
|------|-------------|
| `~/models/whisper-base.bin` | User models directory |
| `/opt/whisper/ggml-small.en.bin` | System-wide installation |
| `/mnt/data/whisper/model.bin` | External drive |

## Model Performance Tips

### Improving Transcription Quality

1. **Speak clearly** - Whisper handles some noise but benefits from clear audio
2. **Use headphones** - Prevents feedback loops
3. **Close background apps** - Reduces CPU contention
4. **Choose appropriate model** - Larger models handle accents better

### Optimizing Speed

1. **Use smaller models** - `base` is fastest
2. **Close other applications** - More CPU available for inference
3. **Keep model in memory** - First transcription may be slower
4. **Use SSD storage** - Faster model loading

## Model Updates

### New Model Versions

Whisper.cpp periodically releases improved models:

- **ggml-base.bin** → improved base model
- **ggml-large-v2** → v3 migration

To update:
1. Download new model
2. Update model path in Settings
3. Old model can be deleted

### Version Compatibility

Clanker Yap v0.1.0 is compatible with:
- All whisper.cpp GGML models (v1.0+)
- Both `.en` and multilingual variants
- INT8 and INT4 quantized models

## Troubleshooting Models

| Issue | Solution |
|-------|----------|
| Download fails | Check internet connection, try different mirror |
| Model not loading | Verify file path, check file isn't corrupted |
| Slow transcription | Use smaller model, close other apps |
| Poor accuracy | Try larger model, improve audio quality |
| Memory issues | Use smaller model, close other apps |