//! Audio recorder using cpal.
//!
//! The cpal::Stream is !Send, so it lives on a dedicated worker thread.
//! Communication with the recorder happens via crossbeam_channel.

use crossbeam_channel::{Receiver, Sender};
use parking_lot::Mutex;
use std::sync::Arc;
use std::thread::{self, JoinHandle};

use super::eq::EqState;
use crate::domain::constants::{MIN_RECORDING_DURATION_MS, WHISPER_SAMPLE_RATE};
use crate::domain::error::{AppError, Result};

enum RecorderCmd {
    Start,
    Stop,
    Shutdown,
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
    // Store stream config separately so we can re-use it across commands
    let stream_config = config.into();

    let err_fn = |err| tracing::error!(error = ?err, "Audio stream error");

    let mut stream: Option<cpal::Stream> = None;
    let buffer: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let buffer_clone = Arc::clone(&buffer);

    loop {
        match cmd_rx.recv() {
            Ok(RecorderCmd::Start) => {
                tracing::debug!("recorder_thread: received Start command");
                // Clear and restart the buffer
                buffer_clone.lock().clear();

                let buffer_for_stream = Arc::clone(&buffer);
                let channels_for_dw = channels;
                let eq_tx_clone = eq_tx.clone();
                let sample_rate_for_eq = sample_rate;

                // Create EQ state for this recording session
                let mut eq_state = EqState::new(sample_rate_for_eq);

                let stream_result = match sample_format {
                    cpal::SampleFormat::F32 => {
                        device.build_input_stream(
                            &stream_config,
                            move |data: &[f32], _: &cpal::InputCallbackInfo| {
                                let mut buf = buffer_for_stream.lock();
                                // Downmix to mono and append
                                if channels_for_dw == 1 {
                                    buf.extend_from_slice(data);
                                    // Feed mono samples to EQ
                                    if let Some(bands) = eq_state.feed(data) {
                                        let _ = eq_tx_clone.try_send(bands);
                                    }
                                } else {
                                    let mut mono = Vec::with_capacity(data.len() / channels_for_dw);
                                    for frame in 0..data.len() / channels_for_dw {
                                        let mut sum = 0.0f32;
                                        for ch in 0..channels_for_dw {
                                            sum += data[frame * channels_for_dw + ch];
                                        }
                                        let mono_sample = sum / channels_for_dw as f32;
                                        buf.push(mono_sample);
                                        mono.push(mono_sample);
                                    }
                                    // Feed mono samples to EQ
                                    if let Some(bands) = eq_state.feed(&mono) {
                                        let _ = eq_tx_clone.try_send(bands);
                                    }
                                }
                            },
                            err_fn,
                            None,
                        )
                    }
                    cpal::SampleFormat::I16 => device.build_input_stream(
                        &stream_config,
                        move |data: &[i16], _: &cpal::InputCallbackInfo| {
                            let mut buf = buffer_for_stream.lock();
                            if channels_for_dw == 1 {
                                buf.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                                // Convert to mono f32 for EQ
                                let mono: Vec<f32> =
                                    data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                                if let Some(bands) = eq_state.feed(&mono) {
                                    let _ = eq_tx_clone.try_send(bands);
                                }
                            } else {
                                let mut mono = Vec::with_capacity(data.len() / channels_for_dw);
                                for frame in 0..data.len() / channels_for_dw {
                                    let mut sum = 0.0f32;
                                    for ch in 0..channels_for_dw {
                                        sum += data[frame * channels_for_dw + ch] as f32
                                            / i16::MAX as f32;
                                    }
                                    let mono_sample = sum / channels_for_dw as f32;
                                    buf.push(mono_sample);
                                    mono.push(mono_sample);
                                }
                                // Feed mono samples to EQ
                                if let Some(bands) = eq_state.feed(&mono) {
                                    let _ = eq_tx_clone.try_send(bands);
                                }
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
                                let mut buf = buffer_for_stream.lock();
                                if channels_for_dw == 1 {
                                    buf.extend(data.iter().map(|&s| s as f32 / i16::MAX as f32));
                                    let mono: Vec<f32> =
                                        data.iter().map(|&s| s as f32 / i16::MAX as f32).collect();
                                    if let Some(bands) = eq_state.feed(&mono) {
                                        let _ = eq_tx_clone.try_send(bands);
                                    }
                                } else {
                                    let mut mono = Vec::with_capacity(data.len() / channels_for_dw);
                                    for frame in 0..data.len() / channels_for_dw {
                                        let mut sum = 0.0f32;
                                        for ch in 0..channels_for_dw {
                                            sum += data[frame * channels_for_dw + ch] as f32
                                                / i16::MAX as f32;
                                        }
                                        let mono_sample = sum / channels_for_dw as f32;
                                        buf.push(mono_sample);
                                        mono.push(mono_sample);
                                    }
                                    if let Some(bands) = eq_state.feed(&mono) {
                                        let _ = eq_tx_clone.try_send(bands);
                                    }
                                }
                            },
                            err_fn,
                            None,
                        )
                    }
                };

                if let Ok(s) = stream_result {
                    stream = Some(s);
                    if let Err(e) = stream.as_ref().unwrap().play() {
                        tracing::warn!(error = ?e, "recorder_thread: Failed to start audio stream");
                    } else {
                        tracing::debug!("recorder_thread: audio stream playing");
                    }
                }
            }
            Ok(RecorderCmd::Stop) => {
                tracing::debug!("recorder_thread: received Stop command");
                // Drop stream, collect buffer, resample
                if let Some(s) = stream.take() {
                    drop(s);
                }
                let samples = buffer_clone.lock().split_off(0);
                tracing::debug!("recorder_thread: collected {} raw samples", samples.len());

                let resampled =
                    super::resample::resample(&samples, sample_rate, WHISPER_SAMPLE_RATE);
                tracing::debug!(
                    "recorder_thread: resampled to {} samples ({} Hz)",
                    resampled.len(),
                    WHISPER_SAMPLE_RATE
                );

                let duration_ms = resampled.len() as i64 * 1000 / WHISPER_SAMPLE_RATE as i64;
                if duration_ms < MIN_RECORDING_DURATION_MS {
                    tracing::warn!(
                        duration_ms,
                        "recorder_thread: audio too short (< {}ms minimum)",
                        MIN_RECORDING_DURATION_MS
                    );
                    let _ = result_tx.send(Err(AppError::Audio("Recording too short".into())));
                } else {
                    let _ = result_tx.send(Ok(resampled));
                }
            }
            Ok(RecorderCmd::Shutdown) | Err(_) => {
                if let Some(s) = stream.take() {
                    drop(s);
                }
                break;
            }
        }
    }
}
