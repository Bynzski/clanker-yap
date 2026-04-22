//! Application-wide constants.

/// Maximum number of transcription history items to retain.
pub const MAX_HISTORY_ITEMS: u32 = 10;

/// Maximum length of a transcription text in characters.
pub const MAX_TRANSCRIPTION_LENGTH: usize = 10_000;

/// Minimum recording duration in milliseconds below which audio is discarded.
/// Prevents accidental hotkey presses from triggering transcription.
pub const MIN_RECORDING_DURATION_MS: i64 = 150;

/// Maximum recording duration in milliseconds. Auto-stops recording if exceeded.
pub const MAX_RECORDING_DURATION_MS: i64 = 60_000;

/// Sample rate required by whisper.cpp (16 kHz).
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;

/// Default GGML model file name.
pub const DEFAULT_MODEL_FILE: &str = "ggml-base.en.bin";

/// Default upstream source for the base English GGML model.
pub const DEFAULT_MODEL_URL: &str =
    "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin?download=true";

/// Display size for the default GGML model from the upstream source listing.
pub const DEFAULT_MODEL_SIZE_LABEL: &str = "148 MB";

/// Default push-to-talk hotkey.
pub const DEFAULT_HOTKEY: &str = "CmdOrCtrl+Shift+V";

/// Application data subdirectory name.
pub const APP_DATA_SUBDIR: &str = "voice-transcribe";
