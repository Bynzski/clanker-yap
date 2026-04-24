//! Shared application state shared across all Tauri commands and handlers.

use parking_lot::Mutex;
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

    /// Current recording/processing state.
    pub recording: Arc<Mutex<RecordingState>>,

    /// Last pipeline error surfaced to the UI.
    pub last_error: Arc<Mutex<Option<String>>>,
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
            recording: Arc::new(Mutex::new(RecordingState::Idle)),
            last_error: Arc::new(Mutex::new(None)),
        }
    }
}
