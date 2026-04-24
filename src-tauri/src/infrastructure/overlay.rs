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

use tauri::{AppHandle, Manager, WebviewUrl, WebviewWindowBuilder};

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
        .focused(false)
        .visible(false)
        .resizable(false);

    let window = builder.build().map_err(|e| e.to_string())?;
    let _window = &window;

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

    // Configure click-through: cursor events pass through to windows beneath.
    let app_clone = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(w) = app_clone.get_webview_window(OVERLAY_LABEL) {
            let _ = w.set_ignore_cursor_events(true);
            tracing::info!("Overlay window created, hidden by default");
        }
    });

    Ok(())
}

/// Shows the overlay window. Safe to call from any thread.
///
/// The overlay window is positioned at bottom-center of the primary monitor.
/// On X11 (or Wayland without Layer Shell), the window uses `set_always_on_top(true)`.
/// On Wayland with Layer Shell, GTK Layer Shell handles positioning.
pub fn show_overlay(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window(OVERLAY_LABEL) {
            // Re-apply always-on-top as some compositors drop it (X11 fallback path)
            let _ = window.set_always_on_top(true);
            let _ = window.show();
            tracing::debug!("Overlay shown");
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
pub fn hide_overlay(app: &AppHandle) {
    // Spawn a short-lived thread to handle the delay and main-thread callback.
    // `app` is Clone + Send + Sync, so it crosses the thread boundary safely.
    let handle_for_delay = app.clone();
    std::thread::spawn(move || {
        std::thread::sleep(std::time::Duration::from_millis(150));
        let h = handle_for_delay.clone();
        let h2 = h.clone();
        let _ = h.run_on_main_thread(move || {
            if let Some(window) = h2.get_webview_window(OVERLAY_LABEL) {
                let _ = window.hide();
                tracing::debug!("Overlay hidden after animation delay");
            }
        });
    });
}
