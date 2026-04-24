# Recording Overlay Pill Plan

**Author:** Jay
**Date:** 2026-04-24
**Status:** Draft
**Version:** 3.0

---

## Purpose

Clanker Yap currently shows recording state **only inside the main settings window**. When the user triggers the global push-to-talk hotkey while working in another application (the entire point of a system-wide dictation tool), they receive **zero visual feedback** that the microphone is active. Every major open-source competitor (Handy, Keyless, Murmur, t2t, Voquill, Sumi, Dictate) has solved this with a floating overlay pill that appears on top of all windows during recording.

This plan adds a **floating, always-on-top overlay pill** that:
1. Appears the instant PTT is pressed (scale-in animation)
2. Shows real-time audio frequency band bars (FFT-based EQ visualization) while recording
3. Transitions to a "processing" state when PTT is released
4. Disappears after transcription completes (scale-out animation)
5. Is click-through (doesn't intercept mouse events)
6. Works on Linux (X11 and Wayland via GTK Layer Shell)

**Research sources examined:** Handy (cjpais), Keyless (hate), Murmur Voice, t2t, Voquill, Sumi, Dictate, VTTA, Hyprflow, macos-mic-keepwarm, and 8 other voice-to-text repos on GitHub.

---

## Scope

### In Scope

| Item | Priority | Notes |
|------|----------|-------|
| Rust: Overlay window creation (transparent, always-on-top, no decorations, no shadow) | P0 | Core infrastructure |
| Rust: Show/hide overlay on recording state transitions | P0 | Wired to orchestrator |
| Rust: FFT-based audio frequency band extraction | P0 | Real FFT → log-spaced frequency bands for multi-bar EQ |
| Rust: Level emission via Tauri events at ~30fps | P0 | Orchestrator spawns task on press, joins on release |
| Rust: GTK Layer Shell support for Wayland (initialized at window creation) | P0 | Modern Linux compositors need this |
| Rust: X11 fallback (set_always_on_top) | P1 | For non-Layer-Shell X11/Wayland |
| Rust: Tauri v2 capabilities/permissions for overlay window | P0 | Must grant show/hide/cursor/focus permissions |
| Frontend: Overlay HTML/CSS (pill shape, bars, animations) | P0 | The visual pill itself |
| Frontend: FFT band visualization with exponential smoothing | P0 | 7 frequency bars react to mic input |
| Frontend: State transitions (recording → processing → hidden) | P0 | Fade in/out, color changes |
| Tauri config: Overlay window definition | P0 | Window registration |

### Out of Scope

- **"Always visible" pill** (only Murmur does this; most apps show pill only during recording)
- **macOS NSPanel** or **Windows HWND_TOPMOST** support (Linux-only for now)
- **Live transcription preview** in the overlay (future enhancement)
- **Audio beep/clack feedback** on recording start/stop (future enhancement)
- **Multi-monitor cursor tracking** (position on primary monitor is sufficient)
- **Configurable overlay position** (bottom-center is the standard)
- **Mic keep-warm** (holding audio stream open between recordings to avoid Bluetooth delay)
- Any changes to the existing main window UI

---

## What Already Exists

| Component | Location | Status |
|-----------|----------|--------|
| RecordingState enum | `src-tauri/src/application/state.rs` | ✅ Exists — Idle, Recording, Processing |
| Orchestrator (on_press / on_release) | `src-tauri/src/application/orchestrator.rs` | ✅ Exists — emits `recording-started`, `recording-stopped`, `transcription-complete` |
| Audio recorder (cpal) | `src-tauri/src/infrastructure/audio/recorder.rs` | ✅ Exists — but no FFT/level metering yet |
| Tauri event system | Used throughout orchestrator | ✅ Exists — `app.emit()` |
| Main window config | `src-tauri/tauri.conf.json` | ✅ Exists — single window, no overlay |
| Capabilities config | `src-tauri/capabilities/default.json` | ⚠️ Only lists `"main"` window — must add `"overlay"` |
| Frontend JS (main.js) | `src/main.js` | ✅ Exists — listens for events via `window.__TAURI__.event` |
| Presentation layer (commands) | `src-tauri/src/presentation/commands/` | ✅ Exists — window_cmds.rs handles window sizing |
| Tauri API shim | `src/vendor/tauri.js` | ✅ Exists — `@tauri-apps/api v2.10.1` shim for vanilla JS |
| Crate structure | `src-tauri/src/lib.rs` | ✅ Exports `voice_transcribe_lib` — all modules live in lib, `main.rs` is thin binary |

### Threading Model (Critical Context)

The orchestrator has two calling contexts that affect overlay design:

| Function | Thread | Signature | Can call window methods directly? |
|----------|--------|-----------|----------------------------------|
| `on_press()` | Main thread (global shortcut callback) | `(&AppHandle, &AppState)` | ✅ Yes |
| `on_release()` | Main thread (global shortcut callback) | `(&AppHandle, &AppState)` | ✅ Yes |
| `pipeline()` | **Blocking thread** (`spawn_blocking`) | `(&AppHandle, &AppState, i64)` | ❌ No — must use `run_on_main_thread()` |

Both `on_press` and `on_release` receive **references** (`&AppHandle`, `&AppState`), not owned values. `AppHandle` is `Clone + Send + Sync`, so overlay functions should accept `&AppHandle` and clone for closures.

Every function in `overlay.rs` must be safe to call from **any thread**. Internally, all `WebviewWindow` method calls must be wrapped in `app.run_on_main_thread()`. This is the pattern both Handy and Keyless use.

### Existing Patterns to Follow

The overlay window should follow the same Tauri event-driven pattern used by the main window. The orchestrator already emits events; the overlay just needs to listen for them.

```rust
// Existing event emission pattern from orchestrator.rs:
// All called with &AppHandle — .emit() works on references
let _ = app.emit("recording-started", ());
let _ = app.emit("recording-stopped", serde_json::json!({ "duration_ms": duration_ms }));
let _ = app.emit("transcription-complete", serde_json::json!({ "text": text, "duration_ms": duration_ms }));
let _ = app.emit("transcription-error", serde_json::json!({ "error": error_message }));
```

Frontend event listening pattern (from `src/main.js`):
```js
// window.__TAURI__ is injected into ALL windows via withGlobalTauri: true in tauri.conf.json
// No <script> tag needed — the overlay uses window.__TAURI__ directly
const { listen } = window.__TAURI__.event;
await listen('recording-started', (event) => handler(event.payload || {}));
```

For overlay window creation, follow Keyless's pattern (same stack: Tauri v2 + Rust):

```rust
// From keyless/keyless-desktop/src-tauri/src/overlay.rs:
WebviewWindowBuilder::new(app, "overlay", url)
    .decorations(false)
    .always_on_top(true)
    .transparent(true)
    .skip_taskbar(true)
    .focused(false)
    .visible(false)
    .shadow(false)       // ← required for clean overlay appearance
    .build()
```

All overlay window operations must be marshaled to the main thread (GAP-2):

```rust
// From Keyless overlay.rs — safe to call from any thread:
pub fn show_overlay(app: &AppHandle) {
    let handle = app.clone();
    let _ = app.run_on_main_thread(move || {
        if let Some(window) = handle.get_webview_window("overlay") {
            let _ = window.show();
        }
    });
}
```

For GTK Layer Shell (Wayland), initialize at window creation time (GAP-6):

```rust
// From cjpais/Handy/src-tauri/src/overlay.rs — called during window creation:
#[cfg(target_os = "linux")]
if let Ok(gtk_window) = overlay_window.gtk_window() {
    gtk_window.init_layer_shell();
    gtk_window.set_layer(Layer::Overlay);
    gtk_window.set_keyboard_mode(KeyboardMode::None);
    gtk_window.set_exclusive_zone(0);
}
```

For FFT-based EQ visualization, follow Keyless's `EqState` pattern. Note: `realfft` is already compiled as a transitive dependency via `rubato` (v0.15, used for audio resampling) — adding it as a direct dependency won't increase compile time (GAP-17).

```rust
// From hate/keyless/keyless-audio/src/eq_pipeline.rs:
// Hann window → 1024pt Real FFT → log-spaced frequency bands
// → noise reduction → dB scaling → gamma curve → attack/decay smoothing
```

---

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| `gtk-layer-shell` | Needed | For Wayland overlay — Handy uses `gtk-layer-shell` v0.8 with `features = ["v0_6"]` |
| `realfft` | Needed | Pure Rust FFT for frequency band extraction — already compiled transitively via `rubato` v0.15.0, no C deps |
| Tauri v2 window API | Already present | `WebviewWindowBuilder` in tauri 2.x |
| cpal audio capture | Already present | Used by recorder — no changes to capture pipeline |
| `tauri-plugin-global-shortcut` | Already present | No changes needed |

---

## Implementation Plan

### Phase Order

| Phase | Description | Depends On |
|-------|-------------|------------|
| 0 | Overlay window infrastructure + GTK Layer Shell + capabilities | — |
| 1 | FFT-based audio frequency band extraction (infrastructure) | Phase 0 |
| 2 | Overlay frontend (HTML/CSS/JS) | Phase 0 |
| 3 | Wire overlay + level emission to orchestrator | Phase 0, Phase 1, Phase 2 |
| 4 | Polish, animations, and edge cases | Phase 3 |

### Phase Details

#### Phase 0 — Overlay Window Infrastructure + GTK Layer Shell

**Purpose:** Create the overlay window at app startup, hidden by default. Initialize GTK Layer Shell on Linux at creation time. Grant the overlay window proper Tauri v2 permissions.

**Scope:**
- [ ] Create `src-tauri/src/infrastructure/overlay.rs` with overlay window management
- [ ] All window operations (`show`, `hide`, `set_position`, `set_ignore_cursor_events`) must be wrapped in `app.run_on_main_thread()` — safe to call from any thread
- [ ] Register overlay module in `src-tauri/src/infrastructure/mod.rs`
- [ ] Add overlay HTML placeholder file at `src/overlay.html` with `html, body { background: transparent; }` (GAP-4)
- [ ] Create overlay window dynamically in `main.rs` setup hook (hidden by default)
- [ ] Use `WebviewUrl::App("overlay.html".into())` — correct since `devUrl: null` and `frontendDist: "../src"` (GAP-5). Add a code comment noting this will need updating if dev mode is ever enabled.
- [ ] Implement `create_overlay(AppHandle)`, `show_overlay(AppHandle, &str)`, `hide_overlay(AppHandle)` functions
- [ ] Position overlay at bottom-center of primary monitor
- [ ] Set window properties: transparent, always-on-top, no decorations, **no shadow** (`.shadow(false)`), skip taskbar, not focusable, ignore cursor events
- [ ] Add `"overlay"` to the `windows` array in `src-tauri/capabilities/default.json` (GAP-1)
- [ ] Grant overlay window permissions in `capabilities/default.json` (GAP-1):
  - `core:window:allow-show`
  - `core:window:allow-hide`
  - `core:window:allow-set-ignore-cursor-events`
  - `core:window:allow-set-always-on-top`
  - `core:window:allow-set-focusable`
  - `core:window:allow-set-decorations`
- [ ] Add `gtk-layer-shell` dependency to `Cargo.toml` (Linux only, feature-gated) (GAP-6)
- [ ] In `create_overlay()`, after window creation on Linux, attempt GTK Layer Shell initialization (GAP-6):
  - `gtk_window.init_layer_shell()`
  - `set_layer(Layer::Overlay)`
  - `set_keyboard_mode(KeyboardMode::None)`
  - `set_exclusive_zone(0)`
  - Anchor bottom-center
- [ ] If Layer Shell unavailable (X11 or unsupported): fall back to `set_always_on_top(true)`
- [ ] Feature gate: `#[cfg(target_os = "linux")]` for all Layer Shell code
- [ ] Verify transparency works — overlay window should be invisible (not opaque black) when shown with transparent background (GAP-4)

**Out of Scope:**
- Audio levels (Phase 1)
- Visual styling beyond placeholder (Phase 2)
- State wiring (Phase 3)

**Files to Modify:**
- `src-tauri/Cargo.toml` — add `gtk-layer-shell` dep (Linux, optional), add `realfft` dep
- `src-tauri/src/infrastructure/mod.rs` — add `overlay` module
- `src-tauri/src/main.rs` — call `create_overlay()` in setup
- `src-tauri/capabilities/default.json` — add `"overlay"` window and window permissions

**New Files:**
- `src-tauri/src/infrastructure/overlay.rs` — overlay window creation, show/hide (thread-safe), GTK Layer Shell init
- `src/overlay.html` — placeholder HTML with transparent background

**Context Files to Read:**
- `src-tauri/src/main.rs` — understand setup hook, note single-instance callback only focuses `"main"` window (not overlay)
- `src-tauri/src/application/orchestrator.rs` — understand state transitions and threading (GAP-2)
- `src-tauri/tauri.conf.json` — current window config
- `src-tauri/capabilities/default.json` — current permissions
- Keyless `overlay.rs` — thread-safe show/hide pattern with `run_on_main_thread`
- Handy `overlay.rs` — GTK Layer Shell initialization at window creation

**Validation:**
- [ ] `cargo check` passes
- [ ] App launches without panic
- [ ] Overlay window exists but is hidden (verify with `get_webview_window("overlay")`)
- [ ] If manually shown, overlay appears transparent (not opaque black)
- [ ] Overlay appears above other windows (Wayland: Layer Shell, X11: always_on_top)

---

#### Phase 1 — FFT-Based Audio Frequency Band Extraction

**Purpose:** Extract real-time frequency band data from the audio recorder using FFT so the overlay can display multi-band EQ bars. This is an infrastructure concern — the module exposes a channel of band values; the orchestrator handles event emission (Phase 3).

**Architecture (GAP-7):**

```
cpal callback
  └─ appends samples to recording buffer (existing)
  └─ feeds samples into eq_tx channel (new)
       │
       ▼
  EqState (maintains FFT buffer, Hann window, attack/decay smoothing)
  - Ring buffer accumulates samples until window size (1024) is reached
  - Runs FFT: Hann window → realfft → magnitude → log-spaced frequency bands
  - Applies noise reduction, dB scaling, gamma curve
  - Applies attack/decay smoothing per band
  - Produces Vec<f32> of 7 normalized band values (0.0–1.0)
  - Sends through levels_rx channel
```

**Scope:**
- [ ] Add `realfft` crate dependency to `Cargo.toml` (pure Rust, no C dependencies; already compiled transitively via `rubato` — no extra compile time) (GAP-17)
- [ ] Create `src-tauri/src/infrastructure/audio/eq.rs` with `EqState` struct (GAP-3):
  - Config: sample rate, FFT size (1024), band count (7), start frequency (~120 Hz), attack/decay
  - Cached FFT plan, Hann window, scratch buffers (zero per-frame allocations)
  - `new(sample_rate: u32)` — initialize FFT plan and pre-compute Hann window
  - `feed(samples: &[f32]) -> Option<Vec<f32>>` — feed samples, returns `Some(bands)` when a full FFT window is available
  - Log-spaced frequency band mapping (pre-computed bucket ranges)
  - Attack (0.45) / decay (0.3) exponential smoothing per band (same as Keyless)
- [ ] Add `eq` module to `src-tauri/src/infrastructure/audio/mod.rs`
- [ ] Add `eq_rx: Receiver<Vec<f32>>` as a public field on `RecorderHandle` (GAP-10):
  - Create the `eq_tx`/`eq_rx` channel in `spawn_for_device()` alongside existing `cmd_tx`/`result_tx` channels
  - Store `eq_rx` on `RecorderHandle` as `pub eq_rx: Receiver<Vec<f32>>`
  - Pass `eq_tx` to the `recorder_thread` function as a new parameter
- [ ] In the `recorder_thread` `Start` arm, create `EqState::new(sample_rate)` per-recording (GAP-11):
  - Clone `eq_tx` for the cpal callback closure (same pattern as `buffer_for_stream`)
  - In the cpal callback, after appending samples to the recording buffer, feed mono samples to `eq_state.feed()`. Note: the `Start` arm has **three** `build_input_stream` calls (F32, I16, fallback I16) — EqState feeding must happen inside all three closures
  - When `eq_state.feed()` returns `Some(bands)`, send through `eq_tx` via non-blocking send
  - The `eq_tx` is a `crossbeam_channel::unbounded()` sender — never blocks the audio callback
  - On `Stop`, `stream.take()` drops the stream and the captured `EqState` + `eq_tx` clone, naturally closing the channel
- [ ] The channel produces band vectors at ~30-50fps naturally (driven by cpal callback rate vs FFT window size)
- [ ] When not recording, `eq_rx.try_recv()` returns `Empty` — consumers simply skip

**Out of Scope:**
- Emitting Tauri events from the recorder (that's the orchestrator's job — Phase 3) (GAP-9)
- Passing `AppHandle` into the recorder (leaks presentation concerns) (GAP-9)
- Changing the existing audio capture / resample pipeline
- Adding `rubato`-based resampling (the existing linear interpolation in `resample.rs` is separate)

**Files to Modify:**
- `src-tauri/Cargo.toml` — add `realfft` dependency
- `src-tauri/src/infrastructure/audio/recorder.rs` — add `eq_rx` field to RecorderHandle, create eq channel in spawn_for_device, create EqState in Start arm of recorder_thread, feed samples in cpal callback
- `src-tauri/src/infrastructure/audio/mod.rs` — add `eq` module, re-export `EqState`

**New Files:**
- `src-tauri/src/infrastructure/audio/eq.rs` — `EqState` struct with FFT, band extraction, smoothing

**Context Files to Read:**
- `src-tauri/src/infrastructure/audio/recorder.rs` — understand cpal callback and `RecorderHandle` API
- Keyless `keyless-audio/src/eq_pipeline.rs` — canonical FFT EQ implementation with `realfft`
- Handy `src-tauri/src/audio_toolkit/audio/visualizer.rs` — alternative FFT implementation with `rustfft`

**Validation:**
- [ ] `cargo check` passes
- [ ] Unit test: `EqState::new()` creates without panic
- [ ] Unit test: feed 1024 samples of known frequency (e.g., 440 Hz sine), verify the correct band shows highest magnitude
- [ ] Integration: during recording, `eq_rx` receives band vectors at ~30fps

---

#### Phase 2 — Overlay Frontend (HTML/CSS/JS)

**Purpose:** Build the visual pill component with FFT-band-reactive EQ bars, state transitions, and animations.

**Scope:**
- [ ] Create `src/overlay.html` with the pill UI (replace placeholder from Phase 0)
- [ ] `html, body { background: transparent; }` — both elements, not just `body` (GAP-4)
- [ ] No `<script src="vendor/tauri.js">` needed — use `window.__TAURI__` directly (injected into ALL windows via `withGlobalTauri: true` in tauri.conf.json) (GAP-13)
- [ ] JS uses: `const { listen } = window.__TAURI__.event;` — same API as `src/main.js`
- [ ] Pill shape: rounded capsule (~180×40px), dark translucent background (`rgba(18, 18, 18, 0.85)`), `backdrop-filter: blur(16px)`
- [ ] EQ bars: 7 thin bars (matching the 7 FFT bands from Phase 1), with gradient colors
- [ ] Smooth bar animations with exponential smoothing in JavaScript (attack=0.45, decay=0.3 per Keyless) as a secondary smoothing layer on top of the Rust-side smoothing
- [ ] State transitions:
  - **Recording:** Green/cyan gradient EQ bars reacting to `mic-level` events, status text
  - **Processing:** Pulsing amber indicator with "Processing..." text (animated dots)
  - **Hidden:** Scale-out animation (150ms), wait for Rust-side hide via event
- [ ] Scale-in animation on appear (200ms, from `scaleX(0.3)` to `scaleX(1)`)
- [ ] Listen for Tauri events: `recording-started`, `recording-stopped`, `transcription-complete`, `transcription-error`, `mic-level`
- [ ] On `recording-started`: show pill with scale-in animation, switch to recording state
- [ ] On `recording-stopped`: switch to processing state
- [ ] On `transcription-complete` or `transcription-error`: trigger scale-out animation, wait for Rust to hide
- [ ] On `mic-level`: update bar heights from `Vec<f32>` payload (7 values, 0.0–1.0)

**Out of Scope:**
- Live transcription preview text
- Cancel button on pill (future)
- Multiple color themes

**Files to Modify:**
- `src/overlay.html` — replace placeholder with full pill implementation (HTML + inline CSS + inline JS)

**New Files:**
- None (overlay.html already created in Phase 0)

**Context Files to Read:**
- `src/style.css` — existing design system (colors, fonts, spacing)
- Keyless `Pill.tsx` + `Pill.css` — best reference for EQ pill with scale animations
- Handy `RecordingOverlay.tsx` + `RecordingOverlay.css` — bar rendering and state transitions

**Visual Design Notes:**
- Follow the app's existing dark theme (`--bg-primary: #121212`, `--accent-success: #3fb950`)
- Pill background: `rgba(18, 18, 18, 0.85)` with `backdrop-filter: blur(16px)` and `border: 1px solid rgba(255,255,255,0.08)`
- Recording bars: gradient from green to cyan (`#3fb950` → `#58a6ff`)
- Processing state: amber (`#c28d3d`) pulsing with `@keyframes` opacity animation
- Bar count: 7 bars (odd number for symmetric appearance)
- Bar minimum height: 4px (visible even when silent)
- Font: same `JetBrains Mono` as main window
- Add a subtle `box-shadow: 0 2px 12px rgba(0,0,0,0.3)` for visibility on light backgrounds

**Validation:**
- [ ] Open `overlay.html` in browser — transparent background, pill visible
- [ ] Manually emit `mic-level` event with test data — bars animate
- [ ] Scale-in/out animations play smoothly

---

#### Phase 3 — Wire Overlay + Level Emission to Orchestrator

**Purpose:** Connect the overlay show/hide calls and FFT level emission to the existing orchestrator state machine. This phase owns the **application-level** responsibility of reading from the `eq_rx` channel and emitting Tauri events (GAP-9).

**Architecture (GAP-7, GAP-9, GAP-14):**

**Event emission order matters:** Always emit the Tauri event *before* calling show/hide overlay. Hidden windows receive events, so the overlay JS renders the correct state before the window becomes visible.

```
on_press(&app, &state)  [main thread]
  ├─ app.emit("recording-started")     ← overlay JS renders recording state (while hidden)
  ├─ show_overlay(&app)                ← window appears already in correct state
  ├─ rec.start()
  └─ spawn level_emission_task(&app, &eq_rx, &cancel_flag)

on_release(&app, &state)  [main thread]
  ├─ app.emit("recording-stopped")     ← overlay transitions to processing
  ├─ rec.stop_and_collect()
  └─ set cancel_flag (AtomicBool)      ← level task exits on next iteration

pipeline(&app, &state, duration)  [blocking thread]
  ├─ transcribe, paste, save
  ├─ app.emit("transcription-complete") ← overlay JS starts exit animation
  └─ hide_overlay(&app)                ← after 150ms delay, uses run_on_main_thread() (GAP-2)
```

**Level Emission Task:**
- Spawned in `on_press()`, runs on its own `std::thread`
- Reads from `RecorderHandle::eq_rx` channel via `eq_rx.recv_timeout(Duration::from_millis(33))`
- Throttles to ~30fps: check `Instant::now()` before each emit, skip if < 33ms since last
- Emits via `app.emit("mic-level", bands)` — broadcast to all windows
- Stops via `level_cancel: Arc<AtomicBool>` on `AppState` (GAP-12):
  - Checked before each channel read — if `true`, exit the thread
  - Set to `true` in `on_release()`, reset to `false` in `on_press()` before spawning
  - Secondary mechanism: `eq_rx` channel closes when recorder drops the stream on Stop

**Scope:**
- [ ] Add `pub level_cancel: Arc<AtomicBool>` to `AppState` (GAP-12)
- [ ] In `on_press()`: emit `app.emit("recording-started")` **first**, then call `show_overlay(&app)` (GAP-14)
- [ ] In `on_press()`: reset `level_cancel` to `false`, then spawn level emission task
- [ ] Level emission task: reads `eq_rx` with `recv_timeout(33ms)`, emits `mic-level` events, checks `level_cancel`
- [ ] In `on_release()`: emit `app.emit("recording-stopped")` **first** (overlay transitions to processing), then set `level_cancel` to `true`
- [ ] In `pipeline()` success path: emit `app.emit("transcription-complete")` **first**, then call `hide_overlay(&app)` — internally uses `run_on_main_thread()` (GAP-2)
- [ ] In `pipeline()` error paths: emit `app.emit("transcription-error")` **first**, then call `hide_overlay(&app)` — same thread-safe hide
- [ ] `hide_overlay()` spawns a short-lived thread with 150ms sleep for exit animation delay, then calls window.hide via `run_on_main_thread()`

**Out of Scope:**
- Changes to the pipeline logic itself
- Changes to main window event handling

**Files to Modify:**
- `src-tauri/src/application/orchestrator.rs` — add overlay calls at state transitions, spawn level emission task with cancel flag
- `src-tauri/src/application/state.rs` — add `level_cancel: Arc<AtomicBool>` to `AppState`
- `src-tauri/src/infrastructure/overlay.rs` — ensure hide_overlay includes 150ms animation delay via spawned thread

**New Files:**
- None

**Context Files to Read:**
- `src-tauri/src/application/orchestrator.rs` — every transition point, understand threading model
- `src-tauri/src/application/state.rs` — understand `AppState` and where to add task handle
- Keyless `overlay.rs` — show/hide pattern with animation delay using `std::thread::spawn` + `sleep`

---

#### Phase 4 — Polish, Animations, and Edge Cases

**Purpose:** Handle edge cases, refine animations, and ensure robust behavior.

**Scope:**
- [ ] Handle rapid PTT toggle (press again while overlay is hiding) — cancel pending hide timer and re-show
- [ ] Handle app shutdown while overlay is visible — clean hide via `orchestrator::shutdown()`
- [ ] Handle monitor disconnect/reconnect — reposition overlay on `show_overlay()`
- [ ] Handle recording too short error — hide overlay
- [ ] Ensure overlay doesn't steal keyboard focus from the active app
- [ ] Verify click-through works (`set_ignore_cursor_events(true)`)
- [ ] Test on both X11 and Wayland
- [ ] Test with fullscreen apps — verify Layer Shell works on Wayland
- [ ] Test that the overlay doesn't interfere with the paste operation
- [ ] Add comment in `main.rs` single-instance callback noting overlay should never be focused (GAP-8)
- [ ] Refine shadow/glow on pill for visibility on light backgrounds

**Out of Scope:**
- Performance optimization
- Accessibility (screen reader) for the overlay
- Configurable animations

**Files to Modify:**
- `src-tauri/src/infrastructure/overlay.rs` — edge case handling
- `src-tauri/src/application/orchestrator.rs` — shutdown cleanup
- `src-tauri/src/main.rs` — comment on single-instance callback (GAP-8)
- `src/overlay.html` — visual refinements

**New Files:**
- None

---

## API / Tauri Commands

No new Tauri commands are needed. The overlay communicates entirely through:
1. **Tauri events** (broadcast via `app.emit()` — received by all windows)
2. **Direct window manipulation** (show/hide via `WebviewWindow` methods in Rust, always through `run_on_main_thread`)

### Events Used

| Event | Direction | Payload | Purpose |
|-------|-----------|---------|---------|
| `recording-started` | Rust → All | `()` | Show overlay in recording state |
| `recording-stopped` | Rust → All | `{ duration_ms: i64 }` | Transition overlay to processing |
| `transcription-complete` | Rust → All | `{ text, duration_ms }` | Hide overlay |
| `transcription-error` | Rust → All | `{ error: String }` | Hide overlay |
| `mic-level` | Rust → All | `Vec<f32>` (7 values, 0.0–1.0) | FFT frequency band bars |

The overlay reuses the same events the main window already listens for. All events are broadcast via `app.emit()` which delivers to ALL windows including hidden ones. The overlay JS sets up listeners on `DOMContentLoaded` (while hidden) and receives events immediately. **Emit events before show/hide overlay calls** so the overlay renders the correct state before becoming visible (GAP-14).

---

## File Structure

Legend: ✅ exists, 🔧 modify, 🆕 new

```
src-tauri/
├── Cargo.toml                                     🔧 add gtk-layer-shell + realfft deps
├── capabilities/
│   └── default.json                               🔧 add "overlay" window + window permissions
├── src/
│   ├── lib.rs                                     ✅ voice_transcribe_lib — all modules exported here
│   ├── main.rs                                    🔧 create overlay in setup, single-instance comment
│   ├── infrastructure/
│   │   ├── mod.rs                                 🔧 add overlay module
│   │   ├── overlay.rs                             🆕 overlay window management (thread-safe)
│   │   └── audio/
│   │       ├── mod.rs                             🔧 add eq module, re-export EqState
│   │       ├── eq.rs                              🆕 FFT-based EQ frequency band extraction
│   │       └── recorder.rs                        🔧 add eq_rx channel, create EqState in Start arm
│   └── application/
│       ├── state.rs                               🔧 add level_cancel: Arc<AtomicBool>
│       └── orchestrator.rs                        🔧 wire overlay + level emission to state transitions

src/
├── overlay.html                                   🆕 overlay pill (HTML + CSS + JS inline)
├── index.html                                     ✅ no changes needed
├── main.js                                        ✅ no changes needed
├── style.css                                      ✅ no changes needed
└── vendor/tauri.js                                ✅ no changes needed (overlay uses window.__TAURI__ directly)
```

---

## New Dependencies

| Package | Version | Purpose | Status |
|---------|---------|---------|--------|
| `gtk-layer-shell` | ~0.8 | Wayland overlay positioning via GTK Layer Shell protocol | Needed (Linux only, feature-gated) |
| `realfft` | ~3 | Pure Rust FFT for frequency band extraction | Needed (already compiled transitively via `rubato`) |

The `gtk-layer-shell` dependency should be platform-gated:
```toml
[target.'cfg(target_os = "linux")'.dependencies]
gtk-layer-shell = { version = "0.8", features = ["v0_6"], optional = true }

[features]
default = ["wayland-overlay"]
wayland-overlay = ["gtk-layer-shell"]
```

The `realfft` dependency is cross-platform. Already compiled as a direct dependency of `rubato` v0.15.0 (used for audio resampling). Confirmed in `Cargo.lock`: `realfft` v3.5.0 is a direct dep of `rubato`. Adding as a direct dependency adds zero compile time:
```toml
realfft = "3"
```

**Existing dependencies used by the plan (no additions needed):**
- `crossbeam-channel = "0.5"` — for `eq_tx`/`eq_rx` channel (already in Cargo.toml)
- `parking_lot = "0.12"` — for Mutex in AppState (already in Cargo.toml)
- `std::sync::atomic::AtomicBool` — for level_cancel flag (standard library)

---

## Testing Strategy

### Manual Testing (Primary — This is a GUI feature)
- [ ] Launch app, verify overlay window is not visible
- [ ] Press PTT hotkey, verify overlay appears at bottom-center with EQ bars
- [ ] Speak, verify bars react to voice (different frequencies should light different bars)
- [ ] Release PTT, verify overlay transitions to "Processing" state
- [ ] After transcription, verify overlay disappears with animation
- [ ] Test rapid PTT toggle — overlay should handle gracefully
- [ ] Test on X11 session
- [ ] Test on Wayland session (Hyprland, Sway, or GNOME)
- [ ] Test with fullscreen app — overlay should appear on top
- [ ] Verify click-through — clicking overlay area should pass through to app below
- [ ] Verify overlay doesn't steal keyboard focus
- [ ] Verify transparent background (not opaque black)

### Build Verification
- [ ] `cargo check` passes
- [ ] `cargo build` succeeds
- [ ] App launches without panic
- [ ] Existing main window functionality unaffected

### Unit Tests
- [ ] Overlay position calculation (bottom-center on various monitor sizes)
- [ ] `EqState`: feed known sine wave, verify correct band peaks
- [ ] `EqState`: attack/decay smoothing produces expected values
- [ ] `EqState`: handles empty and partial buffers gracefully

---

## Rollout Plan

| Phase | Scope | Verification |
|-------|-------|--------------|
| 0 | Overlay window + GTK Layer Shell + capabilities | Window created transparent but hidden; `cargo check` passes; capabilities updated |
| 1 | FFT EQ extraction | `EqState` unit tests pass; `eq_rx` receives band vectors during recording |
| 2 | Overlay frontend | Pill appears when manually shown; bars animate with test data |
| 3 | Orchestrator wiring | Pill appears/disappears with PTT automatically; bars react to real audio |
| 4 | Polish | All edge cases handled; tested on X11 + Wayland |

---

## Reference Implementations

| Project | Stack | What to Reference |
|---------|-------|-------------------|
| **Keyless** (hate/keyless) | Tauri v2 + React + Rust | `EqState` with `realfft`, thread-safe overlay show/hide, EQ pill component |
| **Handy** (cjpais/Handy) | Tauri v2 + React + Rust | GTK Layer Shell at window creation, FFT visualizer with `rustfft`, capabilities |
| **Murmur** (panda850819/murmur-voice) | Tauri v2 + vanilla JS + Rust | Pill with live preview, frosted glass, vanilla JS pattern |
| **t2t** (acoyfellow/t2t) | Tauri v2 + Svelte + Rust | Volume-reactive bottom bar, transparent window |
| **Voquill** (jackbrumley/voquill) | Tauri v2 + Preact + Rust | Gradient orb, animated status icon |

---

## Related Documents

- `plans/voice-transcribe/` — Original app plan
- `plans/recording-overlay-pill/GAP-REPORT.md` — Gap analysis that drove v2.0 changes

---

## Checklist

### Phase 0
- [ ] `infrastructure/overlay.rs` created with `create_overlay()`, `show_overlay()`, `hide_overlay()`
- [ ] All window operations wrapped in `run_on_main_thread()` — safe from any thread
- [ ] Overlay module registered in `mod.rs`
- [ ] Overlay window created in `main.rs` setup, hidden by default
- [ ] `src/overlay.html` placeholder with `html, body { background: transparent; }`
- [ ] Window builder: `.shadow(false)`, `.transparent(true)`, `.decorations(false)`, `.always_on_top(true)`, `.skip_taskbar(true)`, `.focused(false)`
- [ ] `set_ignore_cursor_events(true)` after window creation
- [ ] `WebviewUrl::App("overlay.html")` with code comment about dev mode (GAP-5)
- [ ] `"overlay"` added to `capabilities/default.json` windows array (GAP-1)
- [ ] Window permissions granted in `capabilities/default.json` (GAP-1)
- [ ] `gtk-layer-shell` dependency added (Linux, feature-gated) (GAP-6)
- [ ] GTK Layer Shell initialized at window creation on Linux (GAP-6)
- [ ] Falls back to `set_always_on_top` if Layer Shell unavailable
- [ ] Verify: shown overlay appears transparent (not opaque black) (GAP-4)
- [ ] `cargo check` passes

### Phase 1
- [ ] `realfft` dependency added to `Cargo.toml` (already compiled transitively via `rubato`)
- [ ] `infrastructure/audio/eq.rs` created with `EqState` struct
- [ ] `EqState::new(sample_rate)` pre-computes FFT plan, Hann window, log-spaced band ranges
- [ ] `EqState::feed()` accumulates samples, runs FFT when window full, returns band values
- [ ] Attack/decay smoothing applied per band (0.45/0.3)
- [ ] `eq_rx: Receiver<Vec<f32>>` added as public field on `RecorderHandle` (GAP-10)
- [ ] `eq_tx`/`eq_rx` channel created in `spawn_for_device()`, `eq_tx` passed to recorder thread (GAP-10)
- [ ] `recorder_thread()` signature updated to accept `eq_tx: Sender<Vec<f32>>` parameter; call site in `spawn_for_device()` updated
- [ ] `EqState` created inside `Start` arm of recorder thread, per-recording (GAP-11)
- [ ] cpal callback feeds mono samples to `EqState`, sends bands through `eq_tx` via non-blocking send (all three `build_input_stream` closures: F32, I16, fallback I16)
- [ ] On `Stop`, stream drop closes the channel naturally
- [ ] `eq` module registered in `audio/mod.rs`
- [ ] Unit test: known sine wave produces correct band peak
- [ ] Integration: `eq_rx` receives values at ~30fps during recording

### Phase 2
- [ ] `src/overlay.html` has pill UI with 7 EQ bars
- [ ] `html, body { background: transparent; }` set on both elements
- [ ] No `<script src="vendor/tauri.js">` tag — uses `window.__TAURI__` directly (injected by `withGlobalTauri: true`) (GAP-13)
- [ ] `const { listen } = window.__TAURI__.event;` for event listening
- [ ] Bars animate smoothly with JS-side exponential smoothing
- [ ] Recording state: green/cyan bars reacting to `mic-level` events
- [ ] Processing state: amber pulsing indicator
- [ ] Scale-in animation on show (200ms)
- [ ] Scale-out animation before hide (150ms)
- [ ] Pill has frosted glass effect + subtle shadow for light backgrounds
- [ ] Listens for `recording-started`, `recording-stopped`, `transcription-complete`, `transcription-error`, `mic-level`

### Phase 3
- [ ] `level_cancel: Arc<AtomicBool>` added to `AppState` (GAP-12)
- [ ] Event emission order: emit event first, then show/hide overlay (GAP-14)
- [ ] Overlay shows on `on_press()`: emit `recording-started`, then `show_overlay(&app)`
- [ ] Level emission task spawned in `on_press()`: reads `eq_rx` via `recv_timeout(33ms)`, emits `mic-level`, checks `level_cancel`
- [ ] `level_cancel` reset to `false` before spawning task in `on_press()`
- [ ] `level_cancel` set to `true` in `on_release()` to stop level task
- [ ] Overlay transitions to processing on `on_release()`: emit `recording-stopped`
- [ ] Overlay hides on pipeline complete: emit `transcription-complete`, then `hide_overlay(&app)` via `run_on_main_thread` (GAP-2)
- [ ] Overlay hides on pipeline error: emit `transcription-error`, then `hide_overlay(&app)` (GAP-2)
- [ ] `hide_overlay()` spawns thread with 150ms sleep for animation, then `run_on_main_thread` for window.hide
- [ ] Rapid toggle: cancel pending hide and re-show

### Phase 4
- [ ] Rapid PTT toggle handled
- [ ] Shutdown cleanup in `orchestrator::shutdown()`
- [ ] Monitor disconnect/reconnect handled
- [ ] Shadow/glow for light background visibility
- [ ] Click-through verified
- [ ] Focus stealing verified (should not steal)
- [ ] Paste operation not interfered with
- [ ] Comment in `main.rs` single-instance callback about overlay (GAP-8)
- [ ] Tested on X11
- [ ] Tested on Wayland
- [ ] Tested with fullscreen apps

---

## Change Log

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-24 | Jay | Initial draft |
| 2.0 | 2026-04-24 | Jay | Address 9 gaps from GAP-REPORT.md: capabilities (GAP-1), run_on_main_thread (GAP-2), FFT instead of RMS (GAP-3), transparency/shadow (GAP-4), WebviewUrl strategy (GAP-5), GTK Layer Shell at creation (GAP-6), level emission architecture (GAP-7), single-instance note (GAP-8), phase responsibility split (GAP-9) |
| 3.0 | 2026-04-24 | Jay | Ground in codebase: RecorderHandle eq_rx channel design (GAP-10), EqState per-recording lifecycle (GAP-11), AtomicBool cancellation pattern (GAP-12), window.__TAURI__ direct usage (GAP-13), emit-before-show ordering (GAP-14), &AppHandle signatures (GAP-15), lib+binary split note (GAP-16), realfft transitive dep note (GAP-17) |
