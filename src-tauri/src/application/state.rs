//! Shared application state shared across all Tauri commands and handlers.

use parking_lot::Mutex;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::domain::settings::Settings;
use crate::infrastructure::persistence::Db;

/// Application state holding all shared resources.
#[derive(Clone)]
pub struct AppState {
    /// SQLite database connection.
    pub db: Arc<Db>,

    /// User settings (hotkey, model path, etc.).
    pub settings: Arc<Mutex<Settings>>,

    /// Whisper engine (lazy-loaded on first transcription).
    pub whisper: Arc<Mutex<Option<Arc<crate::infrastructure::whisper::WhisperEngine>>>>,

    /// Audio recorder handle (spawned on first use).
    pub recorder: Arc<Mutex<Option<crate::infrastructure::audio::RecorderHandle>>>,

    /// Persistent paste controller — owns a single Enigo instance for the
    /// app lifetime so that repeated pastes don't trigger new input-device
    /// sessions (which causes KDE/Wayland "Remote Control" prompts).
    pub paste_controller: Arc<Mutex<crate::infrastructure::paste::PasteController>>,

    /// Current recording/processing state.
    pub recording: Arc<Mutex<RecordingState>>,

    /// Last pipeline error surfaced to the UI.
    pub last_error: Arc<Mutex<Option<String>>>,

    /// Flag to cancel the level-emission background task.
    /// Set to `true` in `on_release()`; checked by the level task before each channel read.
    pub level_cancel: Arc<AtomicBool>,
}

/// Current state of the recording pipeline.
#[derive(Default)]
pub enum RecordingState {
    /// Idle, ready to start a new recording.
    #[default]
    Idle,
    /// Actively recording audio.
    Recording { started_at: std::time::Instant },
    /// Transcribing and processing the last recording.
    Processing,
}

impl AppState {
    /// Creates a new AppState from database and settings.
    pub fn new(db: Db, settings: Settings) -> Self {
        Self {
            db: Arc::new(db),
            settings: Arc::new(Mutex::new(settings)),
            whisper: Arc::new(Mutex::new(None)),
            recorder: Arc::new(Mutex::new(None)),
            paste_controller: Arc::new(Mutex::new(
                crate::infrastructure::paste::PasteController::new(),
            )),
            recording: Arc::new(Mutex::new(RecordingState::Idle)),
            last_error: Arc::new(Mutex::new(None)),
            level_cancel: Arc::new(AtomicBool::new(false)),
        }
    }
}
