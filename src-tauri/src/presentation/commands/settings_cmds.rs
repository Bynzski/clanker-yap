//! Settings-related Tauri commands.

use tauri::{AppHandle, State};

use crate::application::orchestrator;
use crate::application::AppState;
use crate::application::use_cases::settings as settings_usecase;
use crate::domain::error::{AppError, Result};
use crate::presentation::dto::{
    SettingsResponse, UpdateSettingsRequest, UpdateSettingsResponse,
};

/// Returns the current settings.
#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<SettingsResponse> {
    let settings = state.settings.lock();
    Ok(SettingsResponse {
        hotkey: settings.hotkey.clone(),
        model_path: settings.model_path.clone(),
        model_name: settings.model_name.clone(),
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

    let mut requires_restart = false;

    if let Some(hotkey) = request.hotkey {
        let changed = hotkey != settings.hotkey;
        settings.hotkey = hotkey.clone();
        if changed {
            requires_restart = !orchestrator::update_hotkey(&app, state.inner());
            if requires_restart {
                tracing::warn!("Hotkey re-registration failed — restart required");
            }
        }
    }

    if let Some(model_path) = request.model_path {
        settings.model_path = model_path;
    }

    if let Some(model_name) = request.model_name {
        settings.model_name = model_name;
    }

    let updated_settings = settings.clone();
    drop(settings);

    settings_usecase::update_settings(&state.db, updated_settings)?;

    Ok(UpdateSettingsResponse {
        success: true,
        message: "Settings updated".into(),
        requires_restart,
    })
}