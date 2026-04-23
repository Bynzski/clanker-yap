# API Reference

Internal Rust API documentation for Clanker Yap.

## Crate Structure

```
src-tauri/src/
├── lib.rs           # Library root
├── domain/          # Domain types
├── application/     # Use cases and state
├── infrastructure/  # External integrations
└── presentation/   # Tauri commands
```

## Domain Layer

### AppError

Unified error type for all application operations.

```rust
use crate::domain::AppError;

pub type Result<T> = std::result::Result<T, AppError>;
```

**Variants:**
```rust
pub enum AppError {
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    Io(std::io::Error),
    Whisper(String),
    Audio(String),
    ModelNotFound(String),
    MicrophoneUnavailable,
    PasteFailed(String),
    SettingsInvalid(String),
}
```

**Usage:**
```rust
fn fallible_function() -> Result<String> {
    Err(AppError::ModelNotFound("/path".into()))
}
```

### Settings

User-configurable settings persisted in SQLite.

```rust
use crate::domain::Settings;

pub struct Settings {
    pub hotkey: String,
    pub model_path: String,
    pub model_name: String,
    pub paste_mode: String,
    pub audio_input: Option<AudioInputSelection>,
    pub schema_version: u32,
}
```

### Transcription

A completed transcription entry.

```rust
use crate::domain::Transcription;

pub struct Transcription {
    pub id: Uuid,
    pub text: String,
    pub duration_ms: i64,
    pub created_at: DateTime<Utc>,
}

impl Transcription {
    pub fn new(text: String, duration_ms: i64) -> Result<Self>
}
```

### AudioInputSelection

Device selection semantics.

```rust
use crate::domain::AudioInputSelection;

pub enum AudioInputSelection {
    SystemDefault,
    ByName(String),
}
```

### Constants

```rust
use crate::domain::constants::*;

pub const DEFAULT_HOTKEY: &str = "CmdOrCtrl+Shift+V";
pub const DEFAULT_MODEL_FILE: &str = "ggml-base.en.bin";
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;
pub const MAX_RECORDING_DURATION_MS: i64 = 60_000;
pub const MIN_RECORDING_DURATION_MS: i64 = 150;
```

## Application Layer

### AppState

Mutable application state container.

```rust
use crate::application::AppState;

pub struct AppState {
    pub settings: Settings,
    pub recording: RecordingState,
    pub db: Database,
}
```

### RecordingState

Current recording status.

```rust
use crate::application::RecordingState;

pub enum RecordingState {
    Idle,
    Recording { start_time: Instant },
    Processing,
}
```

### Orchestrator

Main workflow coordinator.

```rust
use crate::application::orchestrator::Orchestrator;

pub struct Orchestrator {
    state: AppState,
    whisper_engine: WhisperEngine,
    audio_recorder: AudioRecorder,
    paste_service: PasteService,
}
```

**Methods:**
```rust
impl Orchestrator {
    pub fn new(state: AppState) -> Result<Self>
    pub fn start_recording(&mut self) -> Result<()>
    pub fn stop_recording(&mut self) -> Result<Transcription>
    pub fn get_audio_devices(&self) -> Vec<AudioDevice>
    pub fn download_model(&self, url: &str, dest: &Path) -> Result<()>
}
```

## Infrastructure Layer

### WhisperEngine

Whisper transcription wrapper.

```rust
use crate::infrastructure::whisper::WhisperEngine;

pub struct WhisperEngine {
    ctx: whisper_rs::Context,
    params: whisper_rs::FullParams,
}

impl WhisperEngine {
    pub fn new(model_path: &str) -> Result<Self>
    pub fn transcribe(&self, samples: &[f32]) -> Result<String>
}
```

**Usage:**
```rust
let engine = WhisperEngine::new("/path/to/model.bin")?;
let text = engine.transcribe(&samples)?;
```

### AudioRecorder

Audio capture via cpal.

```rust
use crate::infrastructure::audio::AudioRecorder;

pub struct AudioRecorder {
    device_id: Option<String>,
    sample_rate: u32,
}

impl AudioRecorder {
    pub fn new(device_id: Option<String>) -> Result<Self>
    pub fn start(&self) -> Result<()>
    pub fn stop(&self) -> Result<Vec<f32>>
    pub fn list_devices() -> Vec<AudioDevice>
}
```

**Usage:**
```rust
let recorder = AudioRecorder::new(Some("hw:1"))?;
recorder.start();
// ... capture audio ...
let samples = recorder.stop()?;
```

### PasteService

Keyboard simulation for paste injection.

```rust
use crate::infrastructure::paste::PasteService;

pub struct PasteService;

impl PasteService {
    pub fn new() -> Self
    pub fn paste_text(&self, text: &str, mode: &str) -> Result<()>
    pub fn copy_to_clipboard(&self, text: &str) -> Result<()>
}
```

**Usage:**
```rust
let service = PasteService::new();
service.copy_to_clipboard("Hello")?;
service.paste_text("Hello", "auto")?;
```

### Database

SQLite persistence layer.

```rust
use crate::infrastructure::persistence::Database;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new(path: &Path) -> Result<Self>
    pub fn init(&self) -> Result<()>
    
    // Settings
    pub fn get_settings(&self) -> Result<Settings>
    pub fn save_settings(&self, settings: &Settings) -> Result<()>
    
    // Transcriptions
    pub fn save_transcription(&self, t: &Transcription) -> Result<()>
    pub fn get_transcription_history(&self, limit: u32) -> Result<Vec<Transcription>>
}
```

**Usage:**
```rust
let db = Database::new(Path::new("~/data.db"))?;
db.init()?;

let settings = db.get_settings()?;
db.save_settings(&new_settings)?;

let history = db.get_transcription_history(10)?;
```

## Presentation Layer

### Tauri Commands

Commands exposed to the frontend.

#### Settings Commands

```rust
#[tauri::command]
pub fn get_settings(state: tauri::State<'_, AppState>) 
    -> Result<Settings, String>
```

```rust
#[tauri::command]
pub fn save_settings(
    state: tauri::State<'_, AppState>, 
    settings: Settings
) -> Result<(), String>
```

#### Audio Commands

```rust
#[tauri::command]
pub fn get_audio_devices() -> Result<Vec<AudioDevice>, String>
```

```rust
#[tauri::command]
pub fn start_recording(state: tauri::State<'_, AppState>) 
    -> Result<(), String>
```

```rust
#[tauri::command]
pub fn stop_recording(state: tauri::State<'_, AppState>) 
    -> Result<Transcription, String>
```

#### Transcription Commands

```rust
#[tauri::command]
pub fn get_transcription_history(state: tauri::State<'_, AppState>) 
    -> Result<Vec<Transcription>, String>
```

#### Window Commands

```rust
#[tauri::command]
pub fn minimize_window(window: tauri::Window) -> Result<(), String>

#[tauri::command]
pub fn close_window(window: tauri::Window) -> Result<(), String>
```

### Command Registration

In `lib.rs`:

```rust
use crate::presentation::commands::*;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            // Focus existing instance
            if let Some(window) = app.get_webview_window("main") {
                let _ = window.set_focus();
            }
        }))
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings,
            get_audio_devices,
            start_recording,
            stop_recording,
            get_transcription_history,
            minimize_window,
            close_window,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

## Module Quick Reference

| Module | Public API |
|--------|-----------|
| `domain::AppError` | `Result<T>` type alias |
| `domain::Settings` | Settings entity |
| `domain::Transcription` | Transcription entity |
| `application::orchestrator` | `Orchestrator` struct |
| `infrastructure::whisper` | `WhisperEngine` struct |
| `infrastructure::audio` | `AudioRecorder` struct |
| `infrastructure::paste` | `PasteService` struct |
| `infrastructure::persistence` | `Database` struct |

## Error Propagation

All errors use the `?` operator with `AppError`:

```rust
fn read_and_process() -> Result<String> {
    let data = read_file()?;      // Io error
    let parsed = parse_json(&data)?;  // Json error
    Ok(parsed)
}
```

In Tauri commands, errors are converted to strings:

```rust
#[tauri::command]
pub fn my_command() -> Result<String, String> {
    let result = risky_operation().map_err(|e| e.to_string())?;
    Ok(result)
}
```