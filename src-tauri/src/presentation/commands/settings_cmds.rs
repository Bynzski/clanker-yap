//! Settings-related Tauri commands.

use tauri::{AppHandle, State};

use crate::application::orchestrator;
use crate::application::use_cases::model_download as model_download_usecase;
use crate::application::use_cases::settings as settings_usecase;
use crate::application::AppState;
use crate::domain::error::{AppError, Result};
use crate::presentation::dto::{
    DownloadModelResponse, ModelDownloadInfoResponse, SettingsResponse, UpdateSettingsRequest,
    UpdateSettingsResponse,
};

/// Returns the current settings.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsResponse> {
    let settings = state.settings.lock();
    Ok(SettingsResponse {
        hotkey: settings.hotkey.clone(),
        model_path: settings.model_path.clone(),
        model_name: settings.model_name.clone(),
        paste_mode: settings.paste_mode.clone(),
        audio_input: settings.audio_input.clone(),
    })
}

/// Returns information about the built-in base.en model download.
#[tauri::command]
pub fn get_default_model_download_info() -> Result<ModelDownloadInfoResponse> {
    let info = model_download_usecase::get_default_model_info()?;
    Ok(ModelDownloadInfoResponse {
        model_name: info.model_name,
        size_label: info.size_label,
        destination_path: info.destination_path.to_string_lossy().to_string(),
        source_url: info.source_url,
        installed: info.installed,
    })
}

/// Updates the settings.
#[tauri::command]
pub fn update_settings(
    app: AppHandle,
    state: State<'_, AppState>,
    request: UpdateSettingsRequest,
) -> Result<UpdateSettingsResponse> {
    let mut settings = state.settings.lock();
    let mut hotkey_change: Option<(String, String)> = None;
    let mut rollback_result: Option<UpdateSettingsResponse> = None;

    if let Some(ref hotkey) = request.hotkey {
        if hotkey.is_empty() {
            return Err(AppError::SettingsInvalid("Hotkey cannot be empty".into()));
        }
    }

    if let Some(ref model_path) = request.model_path {
        if !model_path.is_empty() && !std::path::Path::new(model_path).exists() {
            return Err(AppError::ModelNotFound(model_path.clone()));
        }
    }

    if let Some(ref paste_mode) = request.paste_mode {
        if !matches!(paste_mode.as_str(), "auto" | "standard" | "terminal") {
            return Err(AppError::SettingsInvalid(format!(
                "Invalid paste mode: {paste_mode}"
            )));
        }
    }

    let mut requires_restart = false;

    if let Some(hotkey) = request.hotkey {
        let previous_hotkey = settings.hotkey.clone();
        let changed = hotkey != previous_hotkey;
        settings.hotkey = hotkey.clone();
        if changed {
            hotkey_change = Some((previous_hotkey, hotkey));
        }
    }

    if let Some(model_path) = request.model_path {
        settings.model_path = model_path;
    }

    if let Some(model_name) = request.model_name {
        settings.model_name = model_name;
    }

    if let Some(paste_mode) = request.paste_mode {
        settings.paste_mode = paste_mode;
    }

    if let Some(audio_input) = request.audio_input {
        settings.audio_input = Some(audio_input);
    }

    let updated_settings = settings.clone();
    drop(settings);

    if let Some((previous_hotkey, new_hotkey)) = hotkey_change {
        if !orchestrator::update_hotkey(&app, state.inner(), &new_hotkey) {
            tracing::warn!("Hotkey re-registration failed — attempting rollback");

            {
                let mut settings = state.settings.lock();
                settings.hotkey = previous_hotkey.clone();
            }
            *state.last_error.lock() =
                Some(format!("Hotkey conflict: {} is already in use", new_hotkey));

            if orchestrator::update_hotkey(&app, state.inner(), &previous_hotkey) {
                rollback_result = Some(UpdateSettingsResponse {
                    success: false,
                    message: format!(
                        "Hotkey '{}' is unavailable; restored '{}'",
                        new_hotkey, previous_hotkey
                    ),
                    requires_restart: false,
                });
            } else {
                requires_restart = true;
                tracing::warn!("Hotkey rollback failed — restart required");
            }
        }
    }

    if let Some(response) = rollback_result {
        return Ok(response);
    }

    settings_usecase::update_settings(&state.db, updated_settings)?;
    *state.last_error.lock() = None;

    Ok(UpdateSettingsResponse {
        success: true,
        message: "Settings updated".into(),
        requires_restart,
    })
}

/// Downloads the default model into the application data directory and updates settings.
#[tauri::command]
pub async fn download_default_model(state: State<'_, AppState>) -> Result<DownloadModelResponse> {
    let app_state = state.inner().clone();

    let info = tauri::async_runtime::spawn_blocking(move || {
        model_download_usecase::download_default_model(&app_state)
    })
    .await
    .map_err(|err| {
        AppError::Io(std::io::Error::other(format!(
            "Download task failed: {err}"
        )))
    })??;

    Ok(DownloadModelResponse {
        success: true,
        message: format!(
            "Downloaded {} to {}",
            info.model_name,
            info.destination_path.to_string_lossy()
        ),
        model_name: info.model_name,
        model_path: info.destination_path.to_string_lossy().to_string(),
    })
}
