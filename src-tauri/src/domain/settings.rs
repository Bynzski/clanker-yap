//! User-configurable settings persisted in SQLite.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Settings stored as a single JSON row in SQLite.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Settings {
    /// Global hotkey for push-to-talk (e.g. "CmdOrCtrl+Shift+V").
    pub hotkey: String,

    /// Full path to the GGML model file.
    pub model_path: String,

    /// Human-readable model name (e.g. "base.en").
    pub model_name: String,

    /// Clipboard paste strategy for different target apps.
    #[serde(default = "default_paste_mode")]
    pub paste_mode: String,

    /// Schema version for future migrations.
    #[serde(default = "current_schema")]
    pub schema_version: u32,
}

fn current_schema() -> u32 {
    1
}

fn default_paste_mode() -> String {
    "auto".to_string()
}

/// Returns the default model path using the dirs crate.
fn default_model_path() -> PathBuf {
    dirs::data_dir()
        .map(|p| {
            p.join(crate::domain::APP_DATA_SUBDIR)
                .join(crate::domain::DEFAULT_MODEL_FILE)
        })
        .unwrap_or_else(|| PathBuf::from(crate::domain::DEFAULT_MODEL_FILE))
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            hotkey: crate::domain::DEFAULT_HOTKEY.to_string(),
            model_path: default_model_path().to_string_lossy().to_string(),
            model_name: crate::domain::DEFAULT_MODEL_FILE.to_string(),
            paste_mode: default_paste_mode(),
            schema_version: current_schema(),
        }
    }
}
