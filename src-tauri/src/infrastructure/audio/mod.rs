//! Audio capture infrastructure.

pub mod device;
pub mod recorder;
pub mod resample;

pub use device::{AudioDeviceInfo, DeviceState};
pub use recorder::RecorderHandle;
