//! Audio recorder using cpal.
//!
//! The cpal::Stream is !Send, so it lives on a dedicated worker thread.
//! Communication with the recorder happens via crossbeam_channel.

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread::{self, JoinHandle};

use super::eq::EqState;
use crate::domain::constants::{MIN_RECORDING_DURATION_MS, WHISPER_SAMPLE_RATE};
use crate::domain::error::{AppError, Result};

enum RecorderCmd {
    Start,
    Stop,
    Shutdown,
}

fn finalize_recording(samples: &[f32], sample_rate: u32) -> Vec<f32> {
    let resampled = super::resample::resample(samples, sample_rate, WHISPER_SAMPLE_RATE);
    let duration_ms = resampled.len() as i64 * 1000 / WHISPER_SAMPLE_RATE as i64;

    if duration_ms < MIN_RECORDING_DURATION_MS {
        tracing::info!(
            duration_ms,
            min_duration_ms = MIN_RECORDING_DURATION_MS,
            "recorder_thread: treating short capture as empty recording"
        );
        Vec::new()
    } else {
        resampled
    }
}

/// Handle to the audio recorder worker thread.
pub struct RecorderHandle {
    cmd_tx: Sender<RecorderCmd>,
    result_rx: Receiver<Result<Vec<f32>>>,
    _join: JoinHandle<()>,
    /// Receives EQ band values (~30fps) while recording.
    pub eq_rx: Receiver<Vec<f32>>,
    pub device_name: String,
}

impl RecorderHandle {
    /// Spawns the recorder worker thread using the system default input device.
    pub fn spawn() -> Result<Self> {
        use cpal::traits::{DeviceTrait, HostTrait};
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .ok_or(AppError::MicrophoneUnavailable)?;
        let name = device.name().unwrap_or_else(|_| "system default".into());
        Self::spawn_for_device(device, name)
    }

    /// Spawns the recorder worker thread for a pre-resolved device.
    pub fn spawn_for_device(device: cpal::Device, device_name: String) -> Result<Self> {
        let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
        let (result_tx, result_rx) = crossbeam_channel::bounded(1);
        let (eq_tx, eq_rx) = crossbeam_channel::unbounded();

        let join = thread::Builder::new()
            .name("audio-recorder".into())
            .spawn(move || recorder_thread(device, cmd_rx, result_tx, eq_tx))?;

        Ok(Self {
            cmd_tx,
            result_rx,
            eq_rx,
            _join: join,
            device_name,
        })
    }

    /// Starts recording.
    pub fn start(&self) -> Result<()> {
        self.cmd_tx.send(RecorderCmd::Start).ok();
        Ok(())
    }

    /// Stops recording and returns the collected samples.
    pub fn stop_and_collect(&self) -> Result<Vec<f32>> {
        self.cmd_tx.send(RecorderCmd::Stop).ok();
        self.result_rx.recv().unwrap_or(Ok(Vec::new()))
    }

    /// Signals the worker to shut down immediately (drop stream, exit thread).
    pub fn shutdown(&self) {
        let _ = self.cmd_tx.send(RecorderCmd::Shutdown);
    }
}

fn recorder_thread(
    device: cpal::Device,
    cmd_rx: Receiver<RecorderCmd>,
    result_tx: Sender<Result<Vec<f32>>>,
    eq_tx: Sender<Vec<f32>>,
) {
    use cpal::traits::{DeviceTrait, StreamTrait};

    let config = match device.default_input_config() {
        Ok(c) => c,
        Err(e) => {
            let _ = result_tx.send(Err(AppError::Audio(format!("Stream build failed: {}", e))));
            return;
        }
    };

    let sample_rate = config.sample_rate().0;
    let channels = config.channels() as usize;
    let sample_format = config.sample_format();
    let stream_config = config.into();

    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let is_recording = Arc::new(AtomicBool::new(false));
    let eq_state = Arc::new(Mutex::new(EqState::new(sample_rate)));

    let err_fn = |err| tracing::error!(error = ?err, "Audio stream error");

    let stream = {
        let buffer_for_stream = Arc::clone(&buffer);
        let is_recording_for_stream = Arc::clone(&is_recording);
        let eq_state_for_stream = Arc::clone(&eq_state);
        let eq_tx_for_stream = eq_tx.clone();

        let stream_result = match sample_format {
            cpal::SampleFormat::F32 => device.build_input_stream(
                &stream_config,
                move |data: &[f32], _: &cpal::InputCallbackInfo| {
                    if !is_recording_for_stream.load(Ordering::Relaxed) {
                        return;
                    }

                    let mut mono = if channels == 1 {
                        data.to_vec()
                    } else {
                        let mut mono = Vec::with_capacity(data.len() / channels);
                        for frame in 0..data.len() / channels {
                            let mut sum = 0.0f32;
                            for ch in 0..channels {
                                sum += data[frame * channels + ch];
                            }
                            mono.push(sum / channels as f32);
                        }
                        mono
                    };

                    buffer_for_stream.lock().extend_from_slice(&mono);
                    if let Some(bands) = eq_state_for_stream.lock().feed(&mono) {
                        let _ = eq_tx_for_stream.try_send(bands);
                    }

                    mono.clear();
                },
                err_fn,
                None,
            ),
            cpal::SampleFormat::I16 => device.build_input_stream(
                &stream_config,
                move |data: &[i16], _: &cpal::InputCallbackInfo| {
                    if !is_recording_for_stream.load(Ordering::Relaxed) {
                        return;
                    }

                    let mono = if channels == 1 {
                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect()
                    } else {
                        let mut mono = Vec::with_capacity(data.len() / channels);
                        for frame in 0..data.len() / channels {
                            let mut sum = 0.0f32;
                            for ch in 0..channels {
                                sum += data[frame * channels + ch] as f32 / i16::MAX as f32;
                            }
                            mono.push(sum / channels as f32);
                        }
                        mono
                    };

                    buffer_for_stream.lock().extend_from_slice(&mono);
                    if let Some(bands) = eq_state_for_stream.lock().feed(&mono) {
                        let _ = eq_tx_for_stream.try_send(bands);
                    }
                },
                err_fn,
                None,
            ),
            _ => {
                // U16 or other formats - treat as i16
                device.build_input_stream(
                    &stream_config,
                    move |data: &[i16], _: &cpal::InputCallbackInfo| {
                        if !is_recording_for_stream.load(Ordering::Relaxed) {
                            return;
                        }

                        let mono = if channels == 1 {
                            data.iter().map(|&s| s as f32 / i16::MAX as f32).collect()
                        } else {
                            let mut mono = Vec::with_capacity(data.len() / channels);
                            for frame in 0..data.len() / channels {
                                let mut sum = 0.0f32;
                                for ch in 0..channels {
                                    sum += data[frame * channels + ch] as f32 / i16::MAX as f32;
                                }
                                mono.push(sum / channels as f32);
                            }
                            mono
                        };

                        buffer_for_stream.lock().extend_from_slice(&mono);
                        if let Some(bands) = eq_state_for_stream.lock().feed(&mono) {
                            let _ = eq_tx_for_stream.try_send(bands);
                        }
                    },
                    err_fn,
                    None,
                )
            }
        };

        match stream_result {
            Ok(s) => s,
            Err(e) => {
                let _ = result_tx.send(Err(AppError::Audio(format!("Stream build failed: {}", e))));
                return;
            }
        }
    };

    if let Err(e) = stream.play() {
        let _ = result_tx.send(Err(AppError::Audio(format!("Stream play failed: {}", e))));
        return;
    }
    tracing::debug!("recorder_thread: audio stream created and playing");

    loop {
        match cmd_rx.recv() {
            Ok(RecorderCmd::Start) => {
                tracing::debug!("recorder_thread: received Start command");
                buffer.lock().clear();
                *eq_state.lock() = EqState::new(sample_rate);
                is_recording.store(true, Ordering::Relaxed);
            }
            Ok(RecorderCmd::Stop) => {
                tracing::debug!("recorder_thread: received Stop command");
                is_recording.store(false, Ordering::Relaxed);

                let samples = buffer.lock().split_off(0);
                tracing::debug!("recorder_thread: collected {} raw samples", samples.len());

                let finalized = finalize_recording(&samples, sample_rate);
                tracing::debug!(
                    "recorder_thread: finalized {} samples ({} Hz)",
                    finalized.len(),
                    WHISPER_SAMPLE_RATE
                );

                let _ = result_tx.send(Ok(finalized));
            }
            Ok(RecorderCmd::Shutdown) | Err(_) => {
                is_recording.store(false, Ordering::Relaxed);
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::finalize_recording;
    use crate::domain::constants::{MIN_RECORDING_DURATION_MS, WHISPER_SAMPLE_RATE};

    #[test]
    fn finalize_recording_returns_empty_for_short_capture() {
        let short_duration_ms = MIN_RECORDING_DURATION_MS - 1;
        let short_samples =
            vec![0.0; (WHISPER_SAMPLE_RATE as i64 * short_duration_ms / 1000) as usize];

        let finalized = finalize_recording(&short_samples, WHISPER_SAMPLE_RATE);

        assert!(finalized.is_empty());
    }

    #[test]
    fn finalize_recording_keeps_valid_capture() {
        let valid_duration_ms = MIN_RECORDING_DURATION_MS + 50;
        let valid_samples =
            vec![0.0; (WHISPER_SAMPLE_RATE as i64 * valid_duration_ms / 1000) as usize];

        let finalized = finalize_recording(&valid_samples, WHISPER_SAMPLE_RATE);

        assert_eq!(finalized.len(), valid_samples.len());
    }
}
