//! Settings-related Tauri commands.

use tauri::State;

use crate::application::AppState;
use crate::application::use_cases::settings as settings_usecase;
use crate::domain::settings::Settings;
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
    state: State<'_, AppState>,
    request: UpdateSettingsRequest,
) -> Result<UpdateSettingsResponse> {
    let mut settings = state.settings.lock();
    
    if let Some(hotkey) = request.hotkey {
        if hotkey.is_empty() {
            return Err(AppError::SettingsInvalid("Hotkey cannot be empty".into()));
        }
        settings.hotkey = hotkey;
    }
    
    if let Some(model_path) = request.model_path {
        if !model_path.is_empty() && !std::path::Path::new(&model_path).exists() {
            return Err(AppError::ModelNotFound(model_path));
        }
        settings.model_path = model_path;
    }
    
    if let Some(model_name) = request.model_name {
        settings.model_name = model_name;
    }
    
    settings_usecase::update_settings(&state.db, settings.clone())?;
    
    Ok(UpdateSettingsResponse {
        success: true,
        message: "Settings updated".into(),
        requires_restart: false,
    })
}