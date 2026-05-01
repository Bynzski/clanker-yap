# Troubleshooting Guide

Solutions to common issues with Clanker Yap.

## Quick Fixes

### Restart the App

Many issues are resolved by restarting:
1. Close the application completely
2. Wait 5 seconds
3. Start again

### Check the Logs

Enable verbose logging and check output:

```sh
RUST_LOG=trace npm run tauri:dev 2>&1 | head -100
```

## Installation Issues

### "Command not found: npm"

**Problem:** npm is not installed or not in PATH.

**Solution:**
```sh
# Linux (Arch)
sudo pacman -S nodejs npm

# Debian/Ubuntu
sudo apt install nodejs npm

# macOS
brew install node
```

### "Rust not found"

**Problem:** Rust is not installed.

**Solution:**
```sh
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
```

Verify: `rustc --version`

### "Could not run tauri"

**Problem:** Tauri CLI not installed.

**Solution:**
```sh
npm install
npm run tauri -- --version
```

## Build Issues

### "Failed to run build command"

**Error:**
```
Error: failed to run custom build command for `webkit2gtk`
```

**Solution:**
```sh
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel

# Arch
sudo pacman -S webkit2gtk-4.1 libappindicator-gtk3 librsvg
```

### "Linker not found"

**Error:**
```
could not find linker (lld or gcc)
```

**Solution:**
```sh
# Arch
sudo pacman -S base-devel

# Debian/Ubuntu
sudo apt install build-essential

# macOS
xcode-select --install
```

### "Package target mismatch"

**Error:**
```
package target mismatch for package X
```

**Solution:**
```sh
cargo clean --manifest-path src-tauri/Cargo.toml
rm -rf node_modules/.cache
npm run tauri:build
```

## Runtime Issues

### Hotkey Not Working

**Symptoms:** Holding hotkey doesn't start recording.

**Diagnosis:**
1. Is another app using the same hotkey?
2. Does the app have keyboard permissions?
3. Is the app window focused (not minimized)?

**Solutions:**

1. Check if hotkey works in another app:
   - Try a different app with Ctrl+Shift+V
   - If it triggers there, change Clanker Yap's hotkey

2. Check system permissions:
   - **Linux:** Check if AppArmor or similar is blocking
   - **macOS:** System Settings > Privacy & Security > Keyboard Monitoring
   - **Windows:** Run as administrator

3. Try a different hotkey:
   ```javascript
   // In settings, change to something unique
   // e.g., "Alt+Shift+V" or "Super+H"
   ```

### Microphone Not Detected

**Symptoms:** "Microphone unavailable" error or no audio captured.

**Diagnosis:**

1. Check microphone is detected:
```sh
# Linux
arecord -l
pactl list sources short

# macOS
system_profiler SPAudioDataType

# Windows
Get-PnpDevice -Class AudioEndpoint
```

2. Check microphone is working elsewhere:
   - Open system audio settings
   - Test recording with another app

**Solutions:**

1. **Linux: PulseAudio**
   ```sh
   # Check if PulseAudio is running
   pulseaudio --check
   
   # Restart if needed
   pulseaudio --kill
   pulseaudio --start
   ```

2. **Linux: ALSA**
   ```sh
   # Check default device
   cat /proc/asound/cards
   ```

3. **Select specific device in app:**
   - Click Mic in toolbar
   - Select your microphone from dropdown
   - Save

4. **Permission issue:**
   ```sh
   # Add user to audio group
   sudo usermod -aG audio $USER
   # Log out and back in
   ```

### Repeated Microphone Permission Prompts (Linux)

**Symptoms:** Desktop/portal microphone permission prompts appear intermittently between push-to-talk cycles.

**What changed in current builds:**
- Recorder now keeps one long-lived CPAL input stream per recorder worker.
- Push-to-talk no longer opens/closes the input stream each Start/Stop cycle.
- Stream is dropped only on recorder shutdown.

**If prompts still appear occasionally:**
1. Verify you launch via the installed desktop entry/AppImage (not mixed launch paths).
2. Ensure no duplicate app instances are running.
3. Check portal/session-manager logs for policy prompts from outside Clanker Yap.

### Model Not Loading

**Error:** "Model not found at: /path/to/model.bin"

**Diagnosis:**
1. Does the file exist?
2. Is the path correct?
3. Is the file readable?

**Solutions:**

1. Verify file exists:
   ```sh
   ls -la ~/.local/share/voice-transcribe/
   ```

2. Download model:
   ```sh
   mkdir -p ~/.local/share/voice-transcribe
   curl -L "https://huggingface.co/ggml-org/whisper.cpp/resolve/main/ggml-base.en.bin" \
     -o ~/.local/share/voice-transcribe/ggml-base.en.bin
   ls -lh ~/.local/share/voice-transcribe/ggml-base.en.bin
   ```

3. Check file permissions:
   ```sh
   chmod 644 ~/.local/share/voice-transcribe/ggml-base.en.bin
   ```

4. Update path in app settings if file is elsewhere

### Model Download Fails

**Symptoms:** Download button doesn't work or stalls.

**Solutions:**

1. Check internet connection:
   ```sh
   ping huggingface.co
   ```

2. Try manual download:
   ```sh
   curl -L --retry 3 "URL" -o output.bin
   ```

3. Use alternative mirror or download from browser

4. Check disk space:
   ```sh
   df -h ~/.local/share/voice-transcribe/
   ```

### No Text Pasted

**Symptoms:** Transcription succeeds but text doesn't appear.

**Diagnosis:**
1. Is clipboard being set?
2. Is paste simulation working?
3. Is target app accepting paste?

**Solutions:**

1. **Check clipboard:**
   - Open the app, make a transcription
   - Manually Ctrl+V in another app
   - Does text appear?

2. **Try different paste mode:**
   - Settings → Paste → Edit
   - Try "Terminal" mode instead of "Auto" or "Standard"

3. **Check target app:**
   - Try pasting into different apps
   - Some apps block simulated paste

4. **Manual copy:**
   - The text is also copied to clipboard
   - Manually Ctrl+V to paste

### KDE/Wayland "Remote Control" Permission Prompt

**Symptoms:** KDE shows a "Remote Control — An application is asking for special privileges: Control input devices" dialog.

**What is happening:**
Clanker Yap simulates Ctrl+V to automatically paste transcribed text. On KDE Plasma with Wayland, this requires input-device access through the desktop portal. The prompt is legitimate and expected.

**Expected behavior (v0.1.2+):**
- You may see **one** prompt when auto-paste first initialises (typically on your first transcription after launching the app)
- Subsequent transcriptions reuse the same input session — no repeated prompts

**If prompts keep appearing:**
1. Make sure you're running v0.1.2 or later (earlier versions created a new input session on every paste)
2. Restart the app to reset the input controller
3. If the issue persists, you can disable auto-paste in Settings → Paste → toggle "Auto-paste" off, and paste manually with Ctrl+V

**Granting permanent permission (KDE):**
You can configure KDE to remember the permission:
1. When the prompt appears, click "Remember"
2. Or go to System Settings → Applications → Clanker Yap → grant "Control input devices"

### Audio Quality Issues

**Symptoms:** Transcriptions are inaccurate or have noise.

**Solutions:**

1. **Speak clearly** - Whisper works best with clear speech

2. **Reduce background noise** - Find a quieter environment

3. **Use headphones** - Prevents feedback loops

4. **Check microphone quality:**
   ```sh
   # Linux: Test recording
   arecord -d 5 test.wav
   aplay test.wav
   ```

5. **Move microphone closer** - Reduce distance to mouth

6. **Use larger model** - `small.en` or `medium.en` handles noise better

### App Won't Start

**Symptoms:** App crashes on launch or doesn't open window.

**Diagnosis:**

1. Check for errors:
   ```sh
   npm run tauri:dev 2>&1 | head -50
   ```

2. Check for existing process:
   ```sh
   # Linux
   ps aux | grep clanker
   killall "Clanker Yap"
   
   # macOS
   pgrep "Clanker Yap"
   killall "Clanker Yap"
   ```

**Solutions:**

1. **Delete database and restart:**
   ```sh
   rm ~/.local/share/voice-transcribe/voice-transcribe.db
   npm run tauri:dev
   ```

2. **Check for missing dependencies:**
   ```sh
   ldd ~/.local/share/voice-transcribe/voice-transcribe
   ```

3. **Try debug mode:**
   ```sh
   RUST_BACKTRACE=1 npm run tauri:dev
   ```

## Performance Issues

### Slow Transcription

**Symptoms:** Transcription takes too long.

**Solutions:**

1. Use smaller model (e.g., `base` instead of `medium`)
2. Close other applications
3. Ensure no CPU-heavy tasks running
4. First transcription is always slower (model loading)

### High CPU Usage

**Symptoms:** App uses too much CPU when idle.

**Diagnosis:**

1. Check what processes are using CPU:
   ```sh
   top -p $(pgrep -d',' "voice-transcribe")
   ```

2. This is normal during transcription, but should be low when idle

**Solutions:**
- Close other apps
- Check no other recording is happening

## Database Issues

### Database Locked

**Error:** "database is locked"

**Solution:**
1. Close any other Clanker Yap instances
2. Delete lock file if exists:
   ```sh
   rm ~/.local/share/voice-transcribe/voice-transcribe.db-lock
   ```

### Database Corruption

**Error:** Database errors on read/write

**Solution:**
1. Backup existing database:
   ```sh
   cp ~/.local/share/voice-transcribe/voice-transcribe.db \
      ~/.local/share/voice-transcribe/voice-transcribe.db.bak
   ```

2. Delete and restart:
   ```sh
   rm ~/.local/share/voice-transcribe/voice-transcribe.db
   npm run tauri:dev
   ```

## Getting Help

If you can't solve your issue:

1. Check existing issues at https://github.com/your-username/clanker-yap/issues
2. Create a new issue with:
   - Steps to reproduce
   - Expected vs actual behavior
   - Logs (run with `RUST_LOG=trace`)
   - Platform and version info

### Debug Information

Run this to get system info:

```sh
echo "=== System Info ===" > debug.txt
uname -a >> debug.txt
echo "=== Rust Version ===" >> debug.txt
rustc --version >> debug.txt
echo "=== Node Version ===" >> debug.txt
node --version >> debug.txt
echo "=== Build Check ===" >> debug.txt
cargo check --manifest-path src-tauri/Cargo.toml 2>&1 >> debug.txt
echo "=== Log ===" >> debug.txt
RUST_LOG=trace npm run tauri:dev 2>&1 | head -100 >> debug.txt
```

Include `debug.txt` in your issue.