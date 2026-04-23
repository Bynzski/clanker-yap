# Command Reference

Tauri commands available to the frontend.

## Overview

Commands are exposed via the Tauri IPC bridge and called from JavaScript using `invoke()`.

```javascript
const { invoke } = window.__TAURI__.core;

// Example call
const result = await invoke('command_name', { arg: 'value' });
```

## Settings Commands

### `get_settings`

Retrieves the current application settings.

**Parameters:** None

**Returns:** `Settings` object

```javascript
const settings = await invoke('get_settings');
console.log(settings.hotkey); // "CmdOrCtrl+Shift+V"
```

**Returns:**
```json
{
  "hotkey": "CmdOrCtrl+Shift+V",
  "model_path": "/home/user/.local/share/voice-transcribe/ggml-base.en.bin",
  "model_name": "ggml-base.en.bin",
  "paste_mode": "auto",
  "audio_input": null,
  "schema_version": 1
}
```

### `save_settings`

Persists settings changes.

**Parameters:** `Settings` object

```javascript
await invoke('save_settings', {
  hotkey: 'Alt+R',
  model_path: '/custom/path/model.bin',
  model_name: 'custom-model',
  paste_mode: 'terminal',
  audio_input: null
});
```

**Returns:** `Ok(())` on success, `Err(AppError)` on failure

## Audio Commands

### `get_audio_devices`

Lists available audio input devices.

**Parameters:** None

**Returns:** Array of device objects

```javascript
const devices = await invoke('get_audio_devices');
// [
//   { id: "default", name: "System Default" },
//   { id: "hw:1", name: "USB Microphone" }
// ]
```

### `start_recording`

Initiates audio recording. Called internally when hotkey is pressed.

**Parameters:** None

**Returns:** `Ok(true)` if recording started, `Err(AppError)` on failure

### `stop_recording`

Stops recording and triggers transcription. Called internally when hotkey is released.

**Parameters:** None

**Returns:** `Ok(Transcription)` on success

```javascript
const result = await invoke('stop_recording');
// {
//   id: "uuid-here",
//   text: "Hello world",
//   duration_ms: 1500,
//   created_at: "2024-01-15T10:30:00Z"
// }
```

## Transcription Commands

### `get_transcription_history`

Retrieves recent transcription history.

**Parameters:** None

**Returns:** Array of `Transcription` objects (max 10)

```javascript
const history = await invoke('get_transcription_history');
console.log(history.length); // 5
console.log(history[0].text); // "Most recent transcription"
```

### `get_model_info`

Returns information about the configured Whisper model.

**Parameters:** None

**Returns:** Model info object

```javascript
const info = await invoke('get_model_info');
// {
//   path: "/path/to/model.bin",
//   name: "ggml-base.en.bin",
//   exists: true,
//   size_bytes: 155000000
// }
```

### `download_model`

Downloads the default Whisper model.

**Parameters:** None

**Returns:** `Ok(true)` on success

```javascript
await invoke('download_model');
```

**Progress:** Can be monitored via events (see Events section)

## Model Commands

### `set_model_path`

Sets a custom path to the Whisper model file.

**Parameters:** `String` (path)

```javascript
await invoke('set_model_path', {
  path: '/custom/path/to/model.bin'
});
```

### `validate_model`

Checks if the model file at the configured path is valid.

**Parameters:** None

**Returns:** `Ok(true)` or `Err(AppError::ModelNotFound(...))`

## Window Commands

### `minimize_window`

Minimizes the application window.

**Parameters:** None

**Returns:** `Ok(())`

```javascript
await invoke('minimize_window');
```

### `close_window`

Closes the application window (and the app).

**Parameters:** None

**Returns:** `Ok(())`

```javascript
await invoke('close_window');
```

## Paste Commands

### `test_paste`

Tests the paste functionality by simulating paste in the foreground app.

**Parameters:** None

**Returns:** `Ok(true)` on success

```javascript
await invoke('test_paste');
```

## Events

The frontend can listen for these events emitted by the backend.

### `recording-state`

Emitted when recording state changes.

```javascript
window.__TAURI__.event.listen('recording-state', (event) => {
  console.log(event.payload);
  // "idle" | "recording" | "processing"
});
```

### `transcription-progress`

Emitted during model download or long operations.

```javascript
window.__TAURI__.event.listen('transcription-progress', (event) => {
  console.log(event.payload);
  // { progress: 0.5, message: "Downloading..." }
});
```

### `error`

Emitted when an error occurs.

```javascript
window.__TAURI__.event.listen('error', (event) => {
  console.error(event.payload);
  // "Model not found at: /path/to/model.bin"
});
```

## Error Handling

All commands return `Result<T, String>` where the error is the `AppError` display string.

```javascript
try {
  await invoke('start_recording');
} catch (error) {
  console.error('Failed to start recording:', error);
  // "Microphone unavailable"
}
```

## TypeScript Definitions

For TypeScript projects, define these types:

```typescript
interface Settings {
  hotkey: string;
  model_path: string;
  model_name: string;
  paste_mode: string;
  audio_input: AudioInput | null;
  schema_version: number;
}

interface Transcription {
  id: string;
  text: string;
  duration_ms: number;
  created_at: string;
}

interface AudioDevice {
  id: string;
  name: string;
}

interface ModelInfo {
  path: string;
  name: string;
  exists: boolean;
  size_bytes: number;
}

type RecordingState = "idle" | "recording" | "processing";
```