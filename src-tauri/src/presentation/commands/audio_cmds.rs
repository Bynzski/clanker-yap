//! Audio device Tauri commands.

use crate::domain::error::Result;
use crate::infrastructure::audio::device;
use crate::infrastructure::audio::AudioDeviceInfo;

/// Returns all available audio input devices.
#[tauri::command]
pub fn list_audio_inputs() -> Result<Vec<AudioDeviceInfo>> {
    device::list_audio_inputs()
}
