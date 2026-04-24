//! Audio capture infrastructure.

pub mod device;
pub mod eq;
pub mod recorder;
pub mod resample;

pub use device::{AudioDeviceInfo, DeviceState};
pub use eq::{EqState, EQ_BAND_COUNT};
pub use recorder::RecorderHandle;
