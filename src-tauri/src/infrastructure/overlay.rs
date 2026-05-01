//! Overlay window management for the floating recording pill.
//!
//! The overlay is a transparent, always-on-top, click-through window that appears
//! during recording to show real-time audio level bars.
//!
//! All window operations are safe to call from any thread because they are
//! marshaled through `app.run_on_main_thread()`. The orchestrator's `pipeline()`
//! runs on a blocking thread, so overlay operations must go through this thread-safety
//! layer — see GAP-2.
//!
//! ## Window lifecycle
//!
//! The overlay window is created once at app startup (in `main.rs` setup hook),
//! hidden by default. It is shown/hidden at recording state transitions.
//!
//! ## Wayland / GTK Layer Shell
//!
//! When compiled with the `wayland-overlay` feature and the `gtk-layer-shell`
//! system library is available, the overlay uses Layer Shell for reliable overlay
//! positioning on Wayland compositors (Hyprland, Sway, etc.). On X11 or Wayland
//! without Layer Shell support, it falls back to `set_always_on_top(true)` via
//! Tauri's window API.
//!
//! ## Thread-safety
//!
//! All `WebviewWindow` method calls must be wrapped in `app.run_on_main_thread()`.
//! `AppHandle` is `Clone + Send + Sync` and crosses thread boundaries safely.
//! The level-emission task and pipeline both call overlay functions from non-main
//! threads — the `run_on_main_thread()` bridge handles this.
//!
//! ## Rapid toggle handling
//!
//! When PTT is pressed while the overlay is mid-hide-animation, the pending
//! hide thread is cancelled so the window doesn't disappear unexpectedly.
//! The `hiding_in_progress` flag prevents concurrent hide operations.
//!
//! ## Shutdown cleanup
//!
//! `hide_overlay()` is safe to call during app exit. It will gracefully skip
//! if the window has already been dropped.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};
use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindowBuilder};

/// Tracks whether a hide operation is in progress, preventing concurrent hides.
static HIDING_IN_PROGRESS: AtomicBool = AtomicBool::new(false);

/// Tracks the handle of the pending hide thread so it can be cancelled on rapid toggle.
static PENDING_HIDE_HANDLE: std::sync::Mutex<Option<std::thread::ThreadId>> =
    std::sync::Mutex::new(None);

/// Overlay window label — used as the unique identifier for this window.
pub const OVERLAY_LABEL: &str = "overlay";

/// Default overlay dimensions (width x height in pixels).
pub const OVERLAY_WIDTH: f64 = 200.0;
pub const OVERLAY_HEIGHT: f64 = 48.0;

/// Creates the overlay window, hidden by default, at the bottom-center of the screen.
/// Called once at app startup from `main.rs` setup hook.
///
/// All window properties are set for a clean floating pill:
/// - Transparent background (no opacity unless CSS provides it)
/// - No decorations (no title bar)
/// - No shadow (clean visual)
/// - Always on top
/// - Click-through (cursor events pass through)
/// - Not focusable (doesn't steal keyboard focus from other apps)
/// - Hidden initially (only shown at recording state transitions)
pub fn create_overlay(app: &AppHandle) -> Result<(), String> {
    let url = WebviewUrl::App("overlay.html".into());

    let builder = WebviewWindowBuilder::new(app, OVERLAY_LABEL, url)
        .title("Recording Overlay") // Accessibility only — window is undecorated
        .inner_size(OVERLAY_WIDTH, OVERLAY_HEIGHT)
        .decorations(false)
        .transparent(true)
        .shadow(false)
        .always_on_top(true)
        .skip_taskbar(true)
        .focusable(false)
        .focused(false)
        .visible(false)
        .resizable(false);

    #[allow(unused_variables)]
    let window = builder.build().map_err(|e| e.to_string())?;

    // Prevent the overlay from stealing focus when it maps (appears on screen).
    // Without this, GTK defaults focus_on_map=true, which causes the window manager
    // to give focus to the overlay when show() is called. On X11, this focus change
    // generates synthetic KeyRelease events that break the global hotkey grab,
    // causing push-to-talk to immediately stop recording.
    //
    // accept_focus=false is already set via Tauri's .focusable(false), but that
    // only prevents *subsequent* focus — focus_on_map prevents focus on the initial
    // map (show) event, which is the critical moment.
    #[cfg(target_os = "linux")]
    {
        use gtk::prelude::GtkWindowExt;
        if let Ok(gtk_win) = window.gtk_window() {
            gtk_win.set_focus_on_map(false);
            tracing::debug!("Overlay: set focus_on_map=false at creation");
        }
    }

    // Initialize GTK Layer Shell on Linux for Wayland overlay positioning.
    // Only compiled when the `wayland-overlay` feature is enabled and the
    // `gtk-layer-shell` system library is available at build time (GAP-6).
    // Layer Shell allows the overlay to appear above full-screen windows
    // and be positioned relative to screen edges on Wayland compositors.
    #[cfg(feature = "wayland-overlay")]
    {
        use gtk_layer_shell::{Edge, KeyboardMode, Layer};

        // Get the GTK window from the Tauri window and attach Layer Shell
        if let Some(gtk_window) = window.gtk_window() {
            let layer_surface = gtk_layer_shell::LayerSurface::for_window(&gtk_window);

            let _ = layer_surface.set_layer(Layer::Overlay);
            let _ = layer_surface.set_keyboard_mode(KeyboardMode::None);
            let _ = layer_surface.set_exclusive_zone(0);

            // Anchor to bottom of screen (centers horizontally by default on Layer Shell)
            let _ = layer_surface.set_anchor(Edge::Bottom, true);
            // Add margin from bottom edge
            let _ = layer_surface.set_margin(Edge::Bottom, 60);

            tracing::info!("Overlay: GTK Layer Shell initialized (Wayland)");
        } else {
            tracing::info!(
                "Overlay: gtk_window() unavailable — using always_on_top fallback (X11)"
            );
        }
    }

    // Click-through: We skip set_ignore_cursor_events here because tao-0.34.8's
    // event loop panics when calling it on a hidden GTK window (unwrap on
    // window.gdk_window()). Instead, we set it in show_overlay() where the
    // window is guaranteed to be visible. See GAP-4.
    tracing::info!("Overlay window created, hidden by default");

    Ok(())
}

/// Shows the overlay window. Safe to call from any thread.
///
/// The overlay window is positioned at bottom-center of the primary monitor.
/// On X11 (or Wayland without Layer Shell), the window uses `set_always_on_top(true)`.
/// On Wayland with Layer Shell, GTK Layer Shell handles positioning.
///
/// ### Rapid toggle handling
///
/// If a hide animation is in progress (via `hide_overlay`), this function
/// cancels the pending hide thread so the window stays visible when PTT is
/// pressed again mid-animation.
pub fn show_overlay(app: &AppHandle) {
    // Cancel any pending hide operation — user pressed PTT again
    cancel_pending_hide();

    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window(OVERLAY_LABEL) {
            // Re-apply always-on-top as some compositors drop it (X11 fallback path)
            let _ = window.set_always_on_top(true);
            let _ = window.set_focusable(false);

            // Re-apply focus_on_map=false before showing. Belt-and-suspenders
            // since we also set it at creation time, but some GTK versions or
            // window managers may reset it.
            #[cfg(target_os = "linux")]
            {
                use gtk::prelude::GtkWindowExt;
                if let Ok(gtk_win) = window.gtk_window() {
                    gtk_win.set_focus_on_map(false);
                }
            }

            let _ = window.show();

            // Set click-through now that the window is visible.
            // This is safe to call here (vs. during create_overlay) because the
            // window is shown, so gdk_window() is available. (tao panics on hidden windows)
            let _ = window.set_ignore_cursor_events(true);

            tracing::debug!("Overlay shown, click-through enabled");
        } else {
            tracing::warn!("Overlay window not found — cannot show");
        }
    });
}

/// Hides the overlay window. Safe to call from any thread.
///
/// This function first waits 150ms to allow the frontend's scale-out animation
/// to play, then hides the window. This is important for the user experience:
/// the pill should animate out smoothly rather than disappearing instantly.
///
/// ### Rapid toggle handling
///
/// If `show_overlay` is called while a hide is in progress, the pending hide
/// is cancelled (tracked via `PENDING_HIDE_HANDLE`) so the overlay doesn't
/// disappear unexpectedly when PTT is pressed again.
///
/// ### Shutdown safety
///
/// Safe to call during app exit. If the window has already been dropped,
/// the `get_webview_window` call returns `None` and the function is a no-op.
pub fn hide_overlay(app: &AppHandle) {
    // Mark hide as in progress so rapid toggle can detect it
    if HIDING_IN_PROGRESS
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_err()
    {
        // A hide is already in progress — skip to avoid double-spawning
        tracing::debug!("hide_overlay called while already hiding — skipping");
        return;
    }

    let handle = app.clone();
    let hide_thread = std::thread::current().id();

    // Store the pending hide thread ID so show_overlay can cancel it
    {
        let mut pending = PENDING_HIDE_HANDLE.lock().unwrap();
        *pending = Some(hide_thread);
    }

    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));

        let h = handle.clone();
        let _ = handle.run_on_main_thread(move || {
            // Check if this hide was cancelled (by a rapid toggle)
            let should_hide = {
                let mut pending = PENDING_HIDE_HANDLE.lock().unwrap();
                if pending.is_some_and(|id| id != hide_thread) {
                    // Cancelled by a newer show_overlay call
                    tracing::debug!("Pending hide cancelled by rapid toggle");
                    false
                } else {
                    *pending = None;
                    true
                }
            };

            if should_hide {
                if let Some(window) = h.get_webview_window(OVERLAY_LABEL) {
                    let _ = window.hide();
                    tracing::debug!("Overlay hidden after animation delay");
                }
            }

            HIDING_IN_PROGRESS.store(false, Ordering::SeqCst);
        });
    });
}

/// Hides the overlay immediately and waits briefly for the native window call
/// to run. Use this before paste injection so the window manager has a chance
/// to restore keyboard focus to the target application.
pub fn hide_overlay_before_paste(app: &AppHandle) {
    cancel_pending_hide();
    HIDING_IN_PROGRESS.store(false, Ordering::SeqCst);

    let (tx, rx) = std::sync::mpsc::channel();
    let handle = app.clone();

    if app
        .run_on_main_thread(move || {
            if let Some(window) = handle.get_webview_window(OVERLAY_LABEL) {
                let _ = window.hide();
                let _ = window.set_focusable(false);
                tracing::debug!("Overlay hidden before paste injection");
            }

            let _ = tx.send(());
        })
        .is_err()
    {
        return;
    }

    let _ = rx.recv_timeout(Duration::from_millis(250));
    std::thread::sleep(Duration::from_millis(75));
}

/// Cancels any pending hide operation so the overlay stays visible.
/// Called by `show_overlay` when the user presses PTT again mid-animation.
fn cancel_pending_hide() {
    let mut pending = PENDING_HIDE_HANDLE.lock().unwrap();
    if pending.is_some() {
        *pending = None;
        tracing::debug!("Pending overlay hide cancelled");
    }
}

// ── Level Emission Task ─────────────────────────────────────────────────────

/// Spawns a background thread that reads EQ band values from the recorder's
/// `eq_rx` channel and emits `mic-level` events to the frontend at ~30fps.
/// Runs until `level_cancel` is set to `true` (by `on_release`).
pub fn spawn_level_emission_task(
    app: AppHandle,
    eq_rx: crossbeam_channel::Receiver<Vec<f32>>,
    level_cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) {
    std::thread::spawn(move || {
        let target_interval = Duration::from_millis(33); // ~30fps

        loop {
            if level_cancel.load(std::sync::atomic::Ordering::Relaxed) {
                tracing::debug!("Level emission task: cancelled");
                break;
            }

            match eq_rx.recv_timeout(target_interval) {
                Ok(bands) => {
                    let now = Instant::now();
                    let _ = app.emit("mic-level", &bands);
                    let elapsed = now.elapsed();

                    if elapsed < target_interval {
                        let remaining = target_interval - elapsed;
                        std::thread::sleep(remaining);
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    continue;
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    tracing::debug!("Level emission task: channel disconnected");
                    break;
                }
            }
        }

        tracing::info!("Level emission task: stopped");
    });
}
