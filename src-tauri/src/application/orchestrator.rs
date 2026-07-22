//! Orchestration — wires global hotkey to record → transcribe → paste → save.
//!
//! Push-to-talk: Pressed = start recording, Released = stop + pipeline.
//!
//! Shortcut events come from `tauri-plugin-global-shortcut` (main.rs callback).
//! Audio flows through `RecorderHandle::stop_and_collect()` (blocking on dedicated thread).
//! Events are emitted to the frontend via `app.emit(...)`.

use cpal::traits::{DeviceTrait, HostTrait};
use tauri::{AppHandle, Emitter};

use crate::application::state::{AppState, RecordingState};
use crate::application::use_cases::paste;
use crate::application::use_cases::paste::PasteOutcome;
use crate::application::use_cases::transcribe as transcribe_usecase;
use crate::application::use_cases::transcription as transcription_usecase;
use crate::domain::transcription::Transcription;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::infrastructure::overlay::spawn_level_emission_task;
#[cfg(any(target_os = "linux", target_os = "windows"))]
use crate::infrastructure::overlay::{hide_overlay, hide_overlay_before_paste, show_overlay};

// ── Helpers ─────────────────────────────────────────────────────────────────

fn transition_to_idle(state: &AppState) {
    let mut r = state.recording.lock();
    *r = RecordingState::Idle;
}

fn set_last_error(state: &AppState, message: impl Into<String>) {
    *state.last_error.lock() = Some(message.into());
}

fn clear_last_error(state: &AppState) {
    *state.last_error.lock() = None;
}

fn completion_transcription_id(
    transcription: &Transcription,
    save_result: &crate::domain::Result<()>,
) -> Option<String> {
    save_result
        .as_ref()
        .ok()
        .map(|()| transcription.id.to_string())
}

// ── Level Emission Task ─────────────────────────────────────────────────────

// ── Hotkey Press ────────────────────────────────────────────────────────────

/// Called when the global hotkey is pressed — starts recording.
pub fn on_press(app: &AppHandle, state: &AppState) {
    tracing::info!("on_press called");
    let recording = state.recording.lock();

    match &*recording {
        RecordingState::Idle => {
            tracing::debug!("on_press: transitioning from Idle -> Recording");
        }
        RecordingState::Recording { .. } => {
            tracing::debug!("Hotkey pressed while already recording — debounced");
            return;
        }
        RecordingState::Processing => {
            tracing::debug!("Hotkey pressed while processing — debounced");
            return;
        }
    }

    drop(recording);

    #[cfg(target_os = "linux")]
    {
        let active_window = state.active_window.lock().clone();
        let mut paste_target = state.paste_target_window.lock();
        *paste_target = active_window;

        if let Some(info) = paste_target.as_ref() {
            tracing::info!(
                resource_class = %info.resource_class,
                desktop_file = %info.desktop_file,
                caption = %info.caption,
                "Captured paste target window"
            );
        } else {
            tracing::info!("No paste target window available at hotkey press");
        }
    }

    // Resolve audio device from settings and (re)spawn recorder if needed
    {
        let settings = state.settings.lock();
        let selection = settings
            .audio_input
            .clone()
            .unwrap_or(crate::domain::settings::AudioInputSelection::SystemDefault);
        drop(settings);

        let (device, device_name) =
            match crate::infrastructure::audio::device::resolve_audio_input(&selection) {
                Ok(d) => d,
                Err(_) => {
                    let host = cpal::default_host();
                    match host.default_input_device() {
                        Some(d) => {
                            let name = d.name().unwrap_or_else(|_| "system default".into());
                            (d, name)
                        }
                        None => {
                            let error_message = "No audio input devices available".to_string();
                            set_last_error(state, error_message.clone());
                            let _ = app.emit(
                                "transcription-error",
                                serde_json::json!({ "error": error_message }),
                            );
                            return;
                        }
                    }
                }
            };

        let mut recorder_slot = state.recorder.lock();
        let needs_respawn = match recorder_slot.as_ref() {
            None => true,
            Some(existing) => existing.device_name != device_name,
        };

        if needs_respawn {
            if let Some(old) = recorder_slot.take() {
                old.shutdown();
            }
            match crate::infrastructure::audio::RecorderHandle::spawn_for_device(
                device,
                device_name.clone(),
            ) {
                Ok(handle) => {
                    *recorder_slot = Some(handle);
                    tracing::info!(device = %device_name, "Audio recorder spawned");
                }
                Err(e) => {
                    tracing::error!(error = ?e, "Failed to spawn audio recorder");
                    let error_message = format!("Microphone unavailable: {}", e);
                    set_last_error(state, error_message.clone());
                    let _ = app.emit(
                        "transcription-error",
                        serde_json::json!({ "error": error_message }),
                    );
                    return;
                }
            }
        }
    }

    // Start recording
    {
        let rec_guard = state.recorder.lock();
        if let Some(rec) = rec_guard.as_ref() {
            if let Err(e) = rec.start() {
                tracing::error!(error = ?e, "Failed to start recorder");
                let error_message = format!("Failed to start recording: {}", e);
                set_last_error(state, error_message.clone());
                let _ = app.emit(
                    "transcription-error",
                    serde_json::json!({
                        "error": error_message
                    }),
                );
                return;
            }
        }
    }

    clear_last_error(state);
    let mut recording = state.recording.lock();
    *recording = RecordingState::Recording {
        started_at: std::time::Instant::now(),
    };

    tracing::info!("Recording started");

    // ── Phase 3 wiring: emit first, then show overlay ─────────────────────
    // Emit event BEFORE show so overlay JS renders the correct state while still hidden
    tracing::debug!("on_press: emitting recording-started and showing overlay");
    let _ = app.emit("recording-started", ());

    // Overlay and level emission — Linux only
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    show_overlay(app);

    // Reset cancel flag and spawn level emission task
    tracing::debug!("on_press: spawning level emission task");
    state
        .level_cancel
        .store(false, std::sync::atomic::Ordering::Relaxed);

    #[cfg(any(target_os = "linux", target_os = "windows"))]
    {
        if let Some(rec) = state.recorder.lock().as_ref() {
            tracing::debug!("on_press: recorder found, spawning level task");
            let eq_rx = rec.eq_rx.clone();
            let cancel = state.level_cancel.clone();
            let app_clone = app.clone();
            spawn_level_emission_task(app_clone, eq_rx, cancel);
        } else {
            tracing::warn!("on_press: no recorder available for level emission");
        }
    }
}

// ── Hotkey Release ──────────────────────────────────────────────────────────

/// Called when the global hotkey is released — triggers the full pipeline.
pub fn on_release(app: &AppHandle, state: &AppState) {
    tracing::info!("on_release called");
    let duration_ms = {
        let mut recording = state.recording.lock();
        match std::mem::replace(&mut *recording, RecordingState::Processing) {
            RecordingState::Recording { started_at } => {
                tracing::debug!("on_release: transitioning from Recording -> Processing");
                started_at.elapsed().as_millis() as i64
            }
            other => {
                let state_name = match other {
                    RecordingState::Idle => "Idle",
                    RecordingState::Recording { .. } => "Recording",
                    RecordingState::Processing => "Processing",
                };
                tracing::warn!(
                    "Release received without matching press — state is {} (not Recording). Check for: 1) Key repeat events 2) Late release after pipeline completes 3) Multiple press events without release",
                    state_name
                );
                return;
            }
        }
    };

    // ── Phase 3 wiring: emit event BEFORE overlay transition ────────────────
    let _ = app.emit(
        "recording-stopped",
        serde_json::json!({ "duration_ms": duration_ms }),
    );

    // Signal level emission task to stop
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    state
        .level_cancel
        .store(true, std::sync::atomic::Ordering::Relaxed);

    tracing::info!(
        duration_ms,
        "Recording stopped, running transcription pipeline"
    );

    // Clone handles for the blocking task
    let app_clone = app.clone();
    let state_clone = std::sync::Arc::new(state.clone());

    tauri::async_runtime::spawn_blocking(move || {
        pipeline(&app_clone, &state_clone, duration_ms);
    });
}

// ── Pipeline (runs on blocking thread) ─────────────────────────────────────

/// Record → transcribe → paste → save.
fn pipeline(app: &AppHandle, state: &AppState, duration_ms: i64) {
    let pipeline_started = std::time::Instant::now();
    tracing::debug!("pipeline started, duration_ms={}", duration_ms);
    // 1. Stop recorder and collect samples
    let collect_started = std::time::Instant::now();
    let samples = {
        tracing::debug!("pipeline: acquiring recorder lock to stop_and_collect");
        let guard = state.recorder.lock();
        tracing::debug!("pipeline: recorder lock acquired, calling stop_and_collect");
        match guard.as_ref().map(|r| r.stop_and_collect()) {
            Some(Ok(s)) => s,
            Some(Err(e)) => {
                transition_to_idle(state);
                let error_message = format!("Recording failed: {}", e);
                set_last_error(state, error_message.clone());
                let _ = app.emit(
                    "transcription-error",
                    serde_json::json!({
                        "error": error_message
                    }),
                );
                #[cfg(any(target_os = "linux", target_os = "windows"))]
                hide_overlay(app);
                return;
            }
            None => {
                transition_to_idle(state);
                let error_message = "No recorder available".to_string();
                set_last_error(state, error_message.clone());
                let _ = app.emit(
                    "transcription-error",
                    serde_json::json!({
                        "error": error_message
                    }),
                );
                #[cfg(any(target_os = "linux", target_os = "windows"))]
                hide_overlay(app);
                return;
            }
        }
    };
    let collect_ms = collect_started.elapsed().as_millis() as u64;

    if samples.is_empty() {
        tracing::info!("No audio samples collected — not enough for transcription");
        transition_to_idle(state);
        // Emit so overlay hides (user released PTT, pipeline can't continue without audio)
        let _ = app.emit(
            "transcription-complete",
            serde_json::json!({ "text": "", "duration_ms": duration_ms }),
        );
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        hide_overlay(app);
        return;
    }

    tracing::debug!(samples = samples.len(), "Collected audio samples");

    // 2. Transcribe (CPU-bound whisper runs on this blocking thread)
    let transcribe_started = std::time::Instant::now();
    let transcribe_result = transcribe_usecase::execute(&samples, state);
    let transcribe_ms = transcribe_started.elapsed().as_millis() as u64;
    let realtime_factor = transcribe_ms as f64 / duration_ms.max(1) as f64;
    tracing::info!(
        recording_duration_ms = duration_ms,
        collect_ms,
        transcribe_ms,
        realtime_factor,
        success = transcribe_result.is_ok(),
        "Transcription pipeline inference phase completed"
    );

    let text = match transcribe_result {
        Ok(t) => t,
        Err(e) => {
            transition_to_idle(state);
            let error_message = format!("Transcription failed: {}", e);
            set_last_error(state, error_message.clone());
            let _ = app.emit(
                "transcription-error",
                serde_json::json!({
                    "error": error_message
                }),
            );
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            hide_overlay(app);
            return;
        }
    };

    if text.is_empty() {
        tracing::info!("Transcription produced empty text — skipping paste and save");
        transition_to_idle(state);
        // Emit so overlay hides (nothing to paste or save, but pill should disappear)
        let _ = app.emit(
            "transcription-complete",
            serde_json::json!({ "text": "", "duration_ms": duration_ms }),
        );
        #[cfg(any(target_os = "linux", target_os = "windows"))]
        hide_overlay(app);
        return;
    }

    tracing::info!(text_len = text.len(), text_preview = %&text[..text.len().min(50)], "Transcribed");

    // 3. Paste (clipboard copy always; keyboard paste via persistent controller)
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    hide_overlay_before_paste(app);
    tracing::info!(text_len = text.len(), "Attempting clipboard copy + paste");
    let paste_started = std::time::Instant::now();
    let paste_outcome = match paste::execute(app, &text, state) {
        Ok(outcome) => {
            match &outcome {
                PasteOutcome::CopiedOnly => {
                    tracing::info!("Text copied to clipboard (auto-paste disabled or unavailable)");
                }
                PasteOutcome::CopiedAndPasted => {
                    tracing::info!("Text copied and auto-pasted");
                }
            }
            outcome
        }
        Err(e) => {
            tracing::warn!(error = ?e, "Clipboard/paste failed");
            // Continue pipeline — the transcription text is still available in memory
            // and will be saved to history.
            PasteOutcome::CopiedOnly
        }
    };
    let paste_ms = paste_started.elapsed().as_millis() as u64;

    // 4. Save transcription
    let transcription = match Transcription::new(text.clone(), duration_ms) {
        Ok(t) => t,
        Err(e) => {
            tracing::warn!(error = ?e, "Transcription entity invalid — not saving");
            transition_to_idle(state);
            clear_last_error(state);
            // Emit BEFORE hiding so overlay can play exit animation
            let _ = app.emit(
                "transcription-complete",
                serde_json::json!({
                    "text": text,
                    "duration_ms": duration_ms
                }),
            );
            #[cfg(any(target_os = "linux", target_os = "windows"))]
            hide_overlay(app);
            return;
        }
    };

    let save_started = std::time::Instant::now();
    let save_result = transcription_usecase::save_transcription(&state.db, &transcription, state);
    if let Err(e) = &save_result {
        tracing::warn!(error = ?e, "Failed to persist transcription");
    }
    let persisted_id = completion_transcription_id(&transcription, &save_result);
    let save_ms = save_started.elapsed().as_millis() as u64;

    transition_to_idle(state);
    clear_last_error(state);

    // ── Phase 3 wiring: emit event BEFORE hide overlay ─────────────────────
    // hide_overlay() uses run_on_main_thread() so it's safe to call from this
    // blocking thread (GAP-2)
    let clipboard_only = paste_outcome == PasteOutcome::CopiedOnly;
    let _ = app.emit(
        "transcription-complete",
        serde_json::json!({
            "id": persisted_id,
            "text": text,
            "duration_ms": duration_ms,
            "clipboard_only": clipboard_only
        }),
    );

    // Hide overlay (Linux only)
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    hide_overlay(app);

    tracing::info!(
        recording_duration_ms = duration_ms,
        samples = samples.len(),
        collect_ms,
        transcribe_ms,
        paste_ms,
        save_ms,
        total_pipeline_ms = pipeline_started.elapsed().as_millis() as u64,
        realtime_factor,
        "Transcription pipeline completed"
    );
}

// ── Hotkey Re-registration ─────────────────────────────────────────────────

/// Re-registers the global shortcut when hotkey setting changes.
/// Returns `true` on success, `false` on failure.
pub fn update_hotkey(app: &AppHandle, state: &AppState, hotkey_str: &str) -> bool {
    use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutState};

    let shortcut: Shortcut = match hotkey_str.parse() {
        Ok(s) => s,
        Err(e) => {
            tracing::error!(error = ?e, "Invalid hotkey string");
            return false;
        }
    };

    // Unregister all first
    if let Err(e) = app.global_shortcut().unregister_all() {
        tracing::warn!(error = ?e, "Failed to unregister existing shortcuts");
    }

    let app_handle = app.clone();
    // Wrap in Arc so the closure can be 'static
    let state_handle = std::sync::Arc::new(state.clone());

    if let Err(e) = app
        .global_shortcut()
        .on_shortcut(shortcut, move |_app, _shortcut, event| {
            tracing::debug!(state = ?event.state, "Global shortcut event received (update_hotkey)");
            match event.state {
                ShortcutState::Pressed => on_press(&app_handle, &state_handle),
                ShortcutState::Released => on_release(&app_handle, &state_handle),
            }
        })
    {
        tracing::error!(error = ?e, "Failed to register new hotkey");
        return false;
    }

    tracing::info!(hotkey = %hotkey_str, "Hotkey re-registered");
    true
}

// ── Shutdown ───────────────────────────────────────────────────────────────

/// Cleans up resources on application exit.
///
/// Hides the overlay window first (if visible) so it doesn't linger during shutdown,
/// then releases the recorder and whisper engine.
pub fn shutdown(app: &AppHandle, state: &AppState) {
    tracing::info!("Orchestrator shutdown");

    #[cfg(target_os = "linux")]
    crate::infrastructure::target_app::unload_kwin_helper();

    // Hide overlay first so it doesn't linger during exit (Linux only)
    #[cfg(any(target_os = "linux", target_os = "windows"))]
    hide_overlay(app);
    let _ = app;

    // Stop active recording if any
    let is_recording = matches!(&*state.recording.lock(), RecordingState::Recording { .. });
    if is_recording {
        let guard = state.recorder.lock();
        if let Some(rec) = guard.as_ref() {
            rec.shutdown();
        }
        tracing::info!("Recording abandoned on shutdown");
    }

    // Drop whisper engine to release model memory
    let mut whisper_slot = state.whisper.lock();
    *whisper_slot = None;
    tracing::info!("Whisper engine dropped");

    // Drop the persistent paste controller (releases Enigo / input session)
    let mut paste_slot = state.paste_controller.lock();
    paste_slot.reset();
    tracing::info!("Paste controller reset");

    #[cfg(target_os = "linux")]
    {
        *state.paste_target_window.lock() = None;
    }
}

#[cfg(test)]
mod tests {
    use super::completion_transcription_id;
    use crate::domain::error::AppError;
    use crate::domain::transcription::Transcription;

    #[test]
    fn completion_id_is_present_after_successful_persistence() {
        let transcription =
            Transcription::new("persisted text".into(), 500).expect("valid transcription");

        assert_eq!(
            completion_transcription_id(&transcription, &Ok(())),
            Some(transcription.id.to_string())
        );
    }

    #[test]
    fn completion_id_is_absent_after_persistence_failure() {
        let transcription =
            Transcription::new("unsaved text".into(), 500).expect("valid transcription");
        let save_result = Err(AppError::SettingsInvalid("save failed".into()));

        assert_eq!(
            completion_transcription_id(&transcription, &save_result),
            None
        );
    }
}
