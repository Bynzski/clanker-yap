# Architecture Documentation

Understanding the internal structure of Clanker Yap.

## Overview

Clanker Yap follows a layered architecture pattern with clear separation of concerns:

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                        │
│                    (Tauri Commands)                          │
├─────────────────────────────────────────────────────────────┤
│                    Application Layer                         │
│                    (Use Cases, State)                        │
├─────────────────────────────────────────────────────────────┤
│                    Domain Layer                              │
│                    (Types, Constants, Errors)               │
├─────────────────────────────────────────────────────────────┤
│                    Infrastructure Layer                       │
│              (Whisper, Audio, Paste, Persistence)           │
└─────────────────────────────────────────────────────────────┘
         │
         ▼
┌─────────────────────────────────────────────────────────────┐
│                    External World                            │
│           (OS, File System, Microphone, Clipboard)          │
└─────────────────────────────────────────────────────────────┘
```

## Directory Structure

```
src-tauri/src/
├── lib.rs                 # Entry point, logging init
├── domain/                # Pure domain logic
│   ├── mod.rs
│   ├── constants.rs       # App-wide constants
│   ├── error.rs           # Unified error type
│   ├── settings.rs        # Settings entity
│   └── transcription.rs   # Transcription entity
├── application/           # Application orchestration
│   ├── mod.rs
│   ├── state.rs           # Runtime state (AppState)
│   ├── orchestrator.rs    # Main workflow coordinator
│   └── use_cases/         # Business operations
│       ├── mod.rs
│       ├── settings.rs    # Settings CRUD
│       ├── transcription.rs
│       ├── transcribe.rs  # Main transcription flow
│       ├── paste.rs      # Paste logic
│       └── model_download.rs
├── infrastructure/        # External integrations
│   ├── mod.rs
│   ├── audio/             # Audio recording
│   │   ├── mod.rs
│   │   ├── recorder.rs    # cpal-based recording
│   │   ├── device.rs      # Device enumeration
│   │   ├── resample.rs    # 48kHz → 16kHz conversion
│   │   └── eq.rs          # FFT-based frequency band extraction
│   ├── whisper/           # ML transcription
│   │   ├── mod.rs
│   │   ├── engine.rs      # whisper-rs wrapper
│   │   └── downloader.rs  # Model download logic
│   ├── paste/             # Clipboard injection
│   │   ├── mod.rs
│   │   └── service.rs     # enigo keyboard simulation
│   └── persistence/       # SQLite storage
│       ├── mod.rs
│       ├── db.rs          # Database initialization
│       ├── paths.rs       # Path resolution
│       ├── settings_repo.rs
│       └── transcription_repo.rs
└── overlay.rs         # Floating always-on-top overlay window
└── presentation/          # Tauri interface
    ├── mod.rs
    └── commands/          # Tauri command handlers
        ├── mod.rs
        ├── settings_cmds.rs
        ├── transcription_cmds.rs
        ├── audio_cmds.rs
        └── window_cmds.rs
```

## Layer Descriptions

### Domain Layer (`domain/`)

**Purpose:** Pure domain types with no external dependencies.

**Contents:**
- `constants.rs` - Application-wide constants (timeouts, limits, defaults)
- `error.rs` - Unified `AppError` type for all error handling
- `settings.rs` - Settings entity with validation
- `transcription.rs` - Transcription entity with validation

**Principles:**
- No I/O operations
- No external dependencies
- All types are `Clone`, `Debug`, `Serialize`
- Validation at construction time

### Application Layer (`application/`)

**Purpose:** Orchestrates domain types and infrastructure.

**Contents:**
- `state.rs` - Runtime state (recording status, app state)
- `orchestrator.rs` - Coordinates the main workflow
- `use_cases/` - Business operations

**Key Types:**

```rust
// AppState - mutable application state
pub struct AppState {
    pub settings: Settings,
    pub recording: RecordingState,
    pub db: Database,
}

// RecordingState - current recording status
pub enum RecordingState {
    Idle,
    Recording { start_time: Instant },
    Processing,
}
```

### Infrastructure Layer (`infrastructure/`)

**Purpose:** External world integrations.

**Audio Module:**
```
recorder.rs   - Records audio via cpal
device.rs     - Lists/selects audio devices
resample.rs   - 48kHz → 16kHz via rubato
eq.rs        - FFT-based frequency band extraction (EqState)
```

**Overlay Module:**
```
overlay.rs     - Floating always-on-top recording indicator pill
               - Thread-safe show/hide via run_on_main_thread
               - GTK Layer Shell support for Wayland compositors
               - X11 fallback via set_always_on_top
```

**Whisper Module:**
```
engine.rs     - whisper-rs wrapper, handles transcription
downloader.rs - HTTP download for model files
```

**Paste Module:**
```
service.rs    - enigo-based keyboard simulation
```

**Persistence Module:**
```
db.rs             - SQLite connection management
settings_repo.rs  - Settings CRUD
transcription_repo.rs - History storage
```

### Presentation Layer (`presentation/`)

**Purpose:** Tauri command handlers that bridge frontend and backend.

**Commands exposed to frontend:**

| Command | Description |
|---------|-------------|
| `get_settings` | Retrieve current settings |
| `save_settings` | Persist settings changes |
| `start_recording` | Begin audio capture |
| `stop_recording` | End capture and transcribe |
| `get_audio_devices` | List available microphones |
| `get_transcription_history` | Fetch recent transcriptions |
| `download_model` | Download Whisper model |
| `minimize_window` | Minimize to taskbar |
| `close_window` | Exit application |

## Data Flow

### Recording Overlay

A floating, always-on-top pill appears when recording starts:

```
Hold hotkey → recording-started event → overlay shown (recording state)
                                               ↓
         mic-level events → FFT EQ bars update in real-time
                                               ↓
Release hotkey → recording-stopped event → overlay transitions to processing state
                                               ↓
           Transcription completes → overlay hides after 150ms animation
```

**Key properties:**
- Click-through (ignores cursor events)
- Transparent background, no decorations or shadow
- GTK Layer Shell on Wayland, X11 fallback on other Linux compositors
- Emits 7 frequency band values (0.0–1.0) at ~30fps via `EqState` (FFT with realfft crate)
- Frontend uses JS-side exponential smoothing (attack=0.45, decay=0.3) for fluid bar animations

### Recording Pipeline

```
┌────────────┐     ┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Hotkey   │────►│   cpal      │────►│   rubato     │────►│  whisper-rs │
│  pressed   │     │  recorder   │     │  resampler   │     │  engine     │
└────────────┘     └─────────────┘     └──────────────┘     └─────────────┘
                   │                                              │
                   │          ┌─────────────┐     ┌──────────────┐  │
                   └─────────►│   EqState   │────►│  mic-level  │  │
                   │ (FFT)     │  (eq.rs)    │     │   events    │  │
                   │          └─────────────┘     └──────┬───────┘  │
                   │                                     │          │
                   │                                     ▼          │
                   │                            ┌──────────────┐     │
                   │                            │  Recording   │     │
                   │                            │  Overlay      │     │
                   │                            │  (EQ bars)     │     │
                   │                            └──────────────┘     │
                   └──────────────────────────────────────────────┘
                                                                      │
                                                                      ▼
                                                            ┌─────────────┐
                                                            │   Text      │
                                                            │   output    │
                                                            └─────────────┘
                                                                      │
                                                                      ▼
                                                            ┌─────────────┐
                                                            │   Paste     │
                                                            │   injection │
                                                            └─────────────┘
```

### Settings Flow

```
┌────────────┐     ┌─────────────┐     ┌──────────────┐
│  Frontend  │────►│  Tauri      │────►│  settings.rs │
│  UI        │     │  command    │     │  + repo      │
└────────────┘     └─────────────┘     └──────────────┘
                                              │
                                              ▼
                                        ┌──────────────┐
                                        │   SQLite     │
                                        │   database   │
                                        └──────────────┘
```

## Key Design Decisions

### 1. Rust Backend with Vanilla Frontend

**Decision:** Use vanilla HTML/CSS/JS instead of a framework.

**Rationale:**
- Smaller bundle size
- No build step complexity for frontend
- Tauri handles the complexity anyway
- Focus is on native functionality, not rich web UI

### 2. Single-Instance via Tauri Plugin

**Decision:** Use `tauri-plugin-single-instance`.

**Rationale:**
- Prevents multiple app instances competing for hotkey
- Ensures clean state
- Handles the "bring to front" requirement

### 3. SQLite for Persistence

**Decision:** Use rusqlite with bundled SQLite.

**Rationale:**
- Zero external dependencies for database
- Simple schema evolution (JSON in single row)
- Sufficient for settings + history storage
- ACID compliance for data integrity

### 4. Global Shortcut Registration

**Decision:** Use `tauri-plugin-global-shortcut`.

**Rationale:**
- Works when app is in background
- Cross-platform abstraction
- Handles modifier key combinations well

### 5. Audio Resampling

**Decision:** Use `rubato` for arbitrary-sample-rate conversion.

**Rationale:**
- cpal gives 48kHz, whisper needs 16kHz
- Simple API, no external deps
- High-quality resampling

## Error Handling

All errors flow through `AppError`:

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

**Error mapping:**
- SQLite errors → `AppError::Sqlite`
- JSON parse errors → `AppError::Json`
- File operations → `AppError::Io`
- Whisper inference → `AppError::Whisper`
- Audio device errors → `AppError::Audio`

## State Management

### Startup State Machine

```
┌─────────┐     ┌─────────────┐     ┌──────────────┐
│  Init   │────►│   Loading   │────►│    Ready    │
└─────────┘     │  settings   │     │  (idle)     │
                │   & model   │     └──────────────┘
                └─────────────┘
```

### Recording State Machine

```
┌─────────┐   hotkey down   ┌─────────────┐  hotkey up  ┌─────────────┐
│   Idle  │────────────────►│  Recording  │────────────►│ Processing  │
└─────────┘                 └─────────────┘             └──────┬──────┘
                                                                │
                                                         ┌──────▼──────┐
                                                         │   Pasting   │
                                                         └──────┬──────┘
                                                                │
                                                         ┌──────▼──────┐
                                                         │    Done     │──► Idle
                                                         └─────────────┘
```

### Overlay State Machine

The overlay pill tracks the same recording state:

```
┌─────────┐  recording-started  ┌─────────────┐ recording-stopped ┌─────────────┐
│  Hidden │─────────────────────►│  Recording  │──────────────────►│ Processing  │
└─────────┘                       └─────────────┘                   └──────┬──────┘
                                                                             │
                                                    transcription-complete /  │
                                                    transcription-error      │
                                                                             │
                                                                     ┌──────▼──────┐
                                                                     │   Hidden    │
                                                                     │ (150ms anim) │
                                                                     └─────────────┘
```

**Overlay events:**
| Event | Effect |
|-------|--------|
| `recording-started` | Pill appears with scale-in animation, EQ bars active |
| `mic-level` | 7-band FFT values update EQ bar heights in real-time |
| `recording-stopped` | Pill transitions to amber pulsing "Processing" state |
| `transcription-complete` | Pill animates out, then window hidden |
| `transcription-error` | Pill animates out, then window hidden |

## Testing Strategy

### Unit Tests

Domain layer has unit tests:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn rejects_text_longer_than_max_length() { ... }
}
```

### Integration Tests

Use `cargo test` for Rust code:
```sh
cargo test --manifest-path src-tauri/Cargo.toml
```

### Manual Testing

Frontend requires manual testing:
1. Hold hotkey → recording starts
2. Speak → audio captured
3. Release → transcription appears
4. Verify text in target application

## Performance Considerations

### Startup Time

- Model loading is synchronous
- First transcription is slowest
- Subsequent transcriptions use cached model

### Memory Usage

- Model: ~148 MB for base.en
- Audio buffer: ~1 MB per recording
- SQLite: < 1 MB for typical usage

### CPU Usage

- Recording: Low (~1-2%)
- Transcription: High (~100% on one core)
- Pasting: Low (~1-2%)