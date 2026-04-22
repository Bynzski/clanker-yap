//! Audio device enumeration and resolution.

use std::collections::HashMap;

use cpal::traits::{DeviceTrait, HostTrait};
use serde::Serialize;

use crate::domain::error::{AppError, Result};
use crate::domain::settings::AudioInputSelection;

/// Single audio input device for UI display.
#[derive(Debug, Serialize)]
pub struct AudioDeviceInfo {
    pub name: String,
    pub is_default: bool,
    pub name_suffix: String,
    pub session_index: usize,
    pub state: DeviceState,
}

/// Device availability state.
#[derive(Debug, Clone, Copy, Serialize)]
pub enum DeviceState {
    Available,
    Unavailable,
    FormatUnsupported,
}

fn probe_device_state(device: &cpal::Device) -> DeviceState {
    match device.supported_input_configs() {
        Ok(mut configs) => {
            if configs.any(|c| c.min_sample_rate().0 <= c.max_sample_rate().0) {
                DeviceState::Available
            } else {
                DeviceState::FormatUnsupported
            }
        }
        Err(_) => DeviceState::Unavailable,
    }
}

/// List all available audio input devices.
pub fn list_audio_inputs() -> Result<Vec<AudioDeviceInfo>> {
    let host = cpal::default_host();

    let default_name = host.default_input_device().and_then(|d| d.name().ok());

    let devices: Vec<_> = host
        .input_devices()
        .map_err(|e| AppError::Audio(format!("Device enumeration failed: {}", e)))?
        .filter_map(|d| d.name().ok().map(|name| (d, name)))
        .collect();

    let mut name_counts: HashMap<String, usize> = HashMap::new();
    for (_, name) in &devices {
        *name_counts.entry(name.clone()).or_default() += 1;
    }

    let mut name_positions: HashMap<String, usize> = HashMap::new();

    let results: Vec<AudioDeviceInfo> = devices
        .into_iter()
        .enumerate()
        .map(|(idx, (device, name))| {
            let count = name_counts.get(&name).copied().unwrap_or(1);
            let is_default = default_name.as_deref() == Some(&name);

            let name_suffix = if count > 1 {
                let pos = name_positions.entry(name.clone()).or_insert(0);
                *pos += 1;
                format!(" \u{2014} #{}", *pos)
            } else {
                String::new()
            };

            let state = probe_device_state(&device);

            AudioDeviceInfo {
                name,
                is_default,
                name_suffix,
                session_index: idx,
                state,
            }
        })
        .collect();

    Ok(results)
}

/// Resolve an `AudioInputSelection` to a cpal device and its name.
pub fn resolve_audio_input(selection: &AudioInputSelection) -> Result<(cpal::Device, String)> {
    let host = cpal::default_host();

    match selection {
        AudioInputSelection::SystemDefault => {
            let device = host
                .default_input_device()
                .ok_or(AppError::MicrophoneUnavailable)?;
            let name = device.name().unwrap_or_else(|_| "system default".into());
            Ok((device, name))
        }
        AudioInputSelection::ByName(target) => {
            let devices: Vec<_> = host
                .input_devices()
                .map_err(|e| AppError::Audio(format!("Device enumeration failed: {}", e)))?
                .filter_map(|d| d.name().ok().map(|n| (d, n)))
                .collect();

            let matches: Vec<_> = devices
                .into_iter()
                .filter(|(_, name)| name == target)
                .collect();

            match matches.len() {
                1 => {
                    let (device, name) = matches.into_iter().next().unwrap();
                    Ok((device, name))
                }
                0 => {
                    tracing::warn!(
                        target_device = %target,
                        "Selected audio device not found, falling back to system default"
                    );
                    let device = host
                        .default_input_device()
                        .ok_or(AppError::MicrophoneUnavailable)?;
                    let name = device.name().unwrap_or_else(|_| "system default".into());
                    Ok((device, name))
                }
                count => {
                    tracing::warn!(
                        target_device = %target,
                        count,
                        "Multiple devices match selected name, falling back to system default"
                    );
                    let device = host
                        .default_input_device()
                        .ok_or(AppError::MicrophoneUnavailable)?;
                    let name = device.name().unwrap_or_else(|_| "system default".into());
                    Ok((device, name))
                }
            }
        }
    }
}
