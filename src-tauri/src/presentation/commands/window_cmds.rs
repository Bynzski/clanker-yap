//! Window-related Tauri commands.

use tauri::{AppHandle, LogicalSize, Size, WebviewWindow};

const WINDOW_WIDTH: f64 = 480.0;
const MIN_WINDOW_HEIGHT: f64 = 196.0;
const MAX_WINDOW_HEIGHT: f64 = 920.0;

#[tauri::command]
pub fn sync_window_size(window: WebviewWindow, content_height: f64) -> Result<(), String> {
    let clamped_height = content_height.clamp(MIN_WINDOW_HEIGHT, MAX_WINDOW_HEIGHT);

    window
        .set_size(Size::Logical(LogicalSize::new(
            WINDOW_WIDTH,
            clamped_height,
        )))
        .map_err(|err| err.to_string())
}

#[tauri::command]
pub fn minimize_window(window: WebviewWindow) -> Result<(), String> {
    window.minimize().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn close_window(app: AppHandle) -> Result<(), String> {
    app.exit(0);
    Ok(())
}

#[tauri::command]
pub fn start_window_drag(window: WebviewWindow) -> Result<(), String> {
    window.start_dragging().map_err(|err| err.to_string())
}

#[tauri::command]
pub fn get_app_version(app: AppHandle) -> String {
    format!("v{}", app.package_info().version)
}
