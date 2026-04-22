//! Model download use cases.

use std::path::PathBuf;

use crate::application::AppState;
use crate::domain::{Result, DEFAULT_MODEL_FILE, DEFAULT_MODEL_SIZE_LABEL, DEFAULT_MODEL_URL};
use crate::infrastructure::persistence::settings_repo;
use crate::infrastructure::whisper::downloader;

pub struct ModelDownloadInfo {
    pub model_name: String,
    pub size_label: String,
    pub destination_path: PathBuf,
    pub source_url: String,
    pub installed: bool,
}

pub fn get_default_model_info() -> Result<ModelDownloadInfo> {
    let destination_path = downloader::default_model_destination()?;
    Ok(ModelDownloadInfo {
        model_name: DEFAULT_MODEL_FILE.to_string(),
        size_label: DEFAULT_MODEL_SIZE_LABEL.to_string(),
        installed: destination_path.exists(),
        destination_path,
        source_url: DEFAULT_MODEL_URL.to_string(),
    })
}

pub fn download_default_model(state: &AppState) -> Result<ModelDownloadInfo> {
    let destination_path = downloader::download_default_model()?;

    {
        let mut settings = state.settings.lock();
        settings.model_name = DEFAULT_MODEL_FILE.to_string();
        settings.model_path = destination_path.to_string_lossy().to_string();
        settings_repo::save(&state.db, &settings)?;
    }

    *state.whisper.lock() = None;
    *state.last_error.lock() = None;

    get_default_model_info()
}
