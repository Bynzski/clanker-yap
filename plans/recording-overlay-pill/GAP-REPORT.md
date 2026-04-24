# Gap Report: Recording Overlay Pill Plan

**Date:** 2026-04-24
**Plan:** `plans/recording-overlay-pill/PLAN.md` v3.0
**Status:** 17 gaps found (4 critical, 4 significant, 5 minor, 4 informational)
**Audited against:** actual codebase at commit HEAD

---

## Critical Gaps (v1.0 — addressed in v2.0)

### GAP-1: Tauri v2 Capabilities/Permissions Not Addressed ✅ Fixed in v2.0

**Severity:** Critical
**Phase:** Phase 0

The plan does not mention Tauri v2's capability/permission system at all. The current `src-tauri/capabilities/default.json` only lists `"main"` in the `windows` array:

```json
{
  "windows": ["main"],
  "permissions": ["core:default", "global-shortcut:default", "clipboard-manager:default"]
}
```

The overlay window (`"overlay"` label) must be added to the `windows` array and granted specific window operation permissions. Without these, the overlay window will silently fail to perform any window operations from the frontend, and Rust-side `WebviewWindow` methods may also be gated.

**Evidence:** Handy's `default.json` explicitly includes `"recording_overlay"` in its windows array. Keyless includes `"overlay"` and `"toast"`.

---

### GAP-2: `pipeline()` Runs on `spawn_blocking` — Cannot Call Window Methods Directly ✅ Fixed in v2.0

**Severity:** Critical
**Phase:** Phase 3

The orchestrator's `pipeline()` function runs on `tauri::async_runtime::spawn_blocking()` (line 189 of orchestrator.rs). In Tauri v2, window operations must run on the main thread. `hide_overlay()` must use `app.run_on_main_thread()` to marshal window operations back to the main thread.

**Evidence:** Confirmed — `run_on_main_thread` is used zero times in the current codebase. All existing window operations happen in Tauri commands (which run on the main thread) or in `on_press`/`on_release` (which run on the main thread via the global shortcut callback).

---

### GAP-3: RMS Cannot Produce Multi-Band EQ Visualization ✅ Fixed in v2.0

**Severity:** Critical
**Phase:** Phase 1, Phase 2

RMS gives a single amplitude value per buffer — you cannot produce 7 distinct animated bars from one value. Both Handy (`rustfft`) and Keyless (`realfft`) use real FFT pipelines with Hann window → 1024pt FFT → log-spaced frequency bands. The plan now correctly specifies FFT-based extraction using `realfft`.

---

### GAP-4: Transparent Window Requires Explicit CSS + shadow(false) ✅ Fixed in v2.0

**Severity:** Critical
**Phase:** Phase 0

Both `<html>` AND `<body>` must have `background: transparent`. The window builder must include `.shadow(false)`. The current `tauri.conf.json` has `"decorations": false` on the main window but no transparency-related settings anywhere.

---

## Significant Gaps (v1.0 — addressed in v2.0)

### GAP-5: No `WebviewUrl` Strategy for Dev vs Production ✅ Fixed in v2.0

**Severity:** Significant
**Phase:** Phase 0

Current app uses `frontendDist: "../src"` with `devUrl: null`. `WebviewUrl::App("overlay.html".into())` is correct for this configuration. A code comment noting this will break if dev mode is ever enabled is sufficient.

---

### GAP-6: GTK Layer Shell Must Be Initialized at Window Creation ✅ Fixed in v2.0

**Severity:** Significant
**Phase:** Phase 0

GTK Layer Shell changes the window type fundamentally. On Wayland, `set_always_on_top` may not work at all. Layer Shell initialization must happen at window creation time in Phase 0, not retrofitted later.

---

### GAP-7: Level Emission Architecture Underspecified ✅ Fixed in v2.0

**Severity:** Significant
**Phase:** Phase 1, Phase 3

The architecture now specifies: `EqState` in infrastructure, channel in `RecorderHandle`, level emission task spawned by orchestrator. Responsibility split is clear: Phase 1 = channel + FFT, Phase 3 = event emission task.

---

## Minor Gaps (v1.0 — addressed in v2.0)

### GAP-8: Single-Instance Plugin Interaction ✅ Fixed in v2.0

**Severity:** Minor
**Phase:** Phase 4

The single-instance callback in `main.rs` (line 19-23) focuses the `"main"` window. The overlay should never be focused. Trivially handled since the overlay has label `"overlay"` not `"main"`, but a code comment is warranted.

---

### GAP-9: Phase 1/3 Responsibility Split for Level Emission ✅ Fixed in v2.0

**Severity:** Minor
**Phase:** Phase 1, Phase 3

Phase 1 = infrastructure (channel, EqState). Phase 3 = application (orchestrator spawns task, emits events). No `AppHandle` in the recorder.

---

## NEW Gaps (v3.0 — found during codebase grounding)

### GAP-10: `RecorderHandle` Architecture Doesn't Support `eq_rx` Channel Naturally

**Severity:** Significant
**Phase:** Phase 1

**Codebase Reality:**

The `RecorderHandle` struct (in `recorder.rs`) is:
```rust
pub struct RecorderHandle {
    cmd_tx: Sender<RecorderCmd>,
    result_rx: Receiver<Result<Vec<f32>>>,
    _join: JoinHandle<()>,
    pub device_name: String,
}
```

Channels are created in `spawn_for_device()`:
```rust
let (cmd_tx, cmd_rx) = crossbeam_channel::unbounded();
let (result_tx, result_rx) = crossbeam_channel::bounded(1);
```

The cpal stream is created fresh inside the `Start` arm of the `recorder_thread` loop. The `EqState` must be created inside the `Start` arm and used within the cpal callback closure.

**Design Decision:** Create the `eq_tx`/`eq_rx` channel in `spawn_for_device()` (alongside existing channels), store `eq_rx` as a public field on `RecorderHandle`, and pass `eq_tx` to the recorder thread. Inside the `Start` arm, create `EqState::new(sample_rate)` and clone `eq_tx` into the cpal callback. On `Stop`, the stream is dropped (dropping the cloned sender), which naturally closes the channel.

The `eq_rx` field is always present on `RecorderHandle` but only produces values while recording is active.

**Impact on Plan:** Phase 1 must specify this exact pattern. The `eq_rx` is a `crossbeam_channel::Receiver<Vec<f32>>` — non-blocking reads return `Err(TryRecvError::Empty)` when no FFT output is available.

---

### GAP-11: `EqState` Lifetime Tied to cpal Stream — Must Be Created Per-Recording

**Severity:** Significant
**Phase:** Phase 1

**Codebase Reality:**

The recorder thread's `Start` arm creates a new `cpal::Stream` each time:
```rust
Ok(RecorderCmd::Start) => {
    buffer_clone.lock().clear();
    let stream_result = match sample_format {
        cpal::SampleFormat::F32 => device.build_input_stream(&stream_config, move |data, _| { ... }, err_fn, None),
        ...
    };
}
```

The callback closure captures `buffer_for_stream` by `Arc::clone`. The `EqState` must follow the same pattern — created in the `Start` arm, captured by the closure, dropped when the stream is dropped on `Stop`.

**Design Decision:** In the `Start` arm:
1. Create `EqState::new(sample_rate)`
2. Clone `eq_tx` for the closure
3. In the cpal callback, after appending samples to buffer, call `eq_state.feed(samples)` and send results through `eq_tx`
4. On `Stop`, `stream.take()` drops the stream and the captured `EqState` + `eq_tx` clone

**Impact on Plan:** Phase 1 scope should explicitly say "create EqState inside the Start arm, not as a persistent field on any struct."

---

### GAP-12: Level Task Cancellation Pattern for `AppState`

**Severity:** Significant
**Phase:** Phase 3

**Codebase Reality:**

`AppState` uses `Arc<Mutex<Option<T>>>` for optional resources:
```rust
pub whisper: Arc<Mutex<Option<Arc<WhisperEngine>>>>,
pub recorder: Arc<Mutex<Option<RecorderHandle>>>,
```

The plan says "Store level emission task handle in AppState" but doesn't specify the pattern. Since `on_press`/`on_release` receive `&AppState` (not `&mut`), mutation must go through interior mutability.

**Design Decision:** Two options:

**Option A (Cancellation flag):** Add `pub level_cancel: Arc<AtomicBool>` to `AppState`. The level emission task checks this flag before each emit. Set to `true` in `on_release()`, reset to `false` in `on_press()`. Simpler and doesn't require joining.

**Option B (JoinHandle):** Add `pub level_task: Arc<Mutex<Option<JoinHandle<()>>>>`. Join in `on_release()` to ensure clean shutdown. More correct but requires careful handling to avoid blocking the main thread on join.

**Recommendation:** Option A (AtomicBool) — the level task reads from a channel that will be closed when the recorder stops, so the task exits naturally. The AtomicBool is a secondary safety signal. No join needed.

**Impact on Plan:** Phase 3 must specify `pub level_cancel: Arc<AtomicBool>` on `AppState` and the create/reset/check pattern.

---

### GAP-13: Overlay Frontend Uses `window.__TAURI__` Directly

**Severity:** Minor
**Phase:** Phase 2

**Codebase Reality:**

`tauri.conf.json` has `"withGlobalTauri": true`. The existing `main.js` uses:
```js
const { invoke } = window.__TAURI__.core;
const { listen } = window.__TAURI__.event;
```

The `src/vendor/tauri.js` is a shim loaded via `<script>` tag. The overlay HTML must either:
- **Option A:** Include `<script src="vendor/tauri.js"></script>` — but this is a relative path, and the overlay HTML is served at the root like `index.html`
- **Option B:** Use `window.__TAURI__` directly since `withGlobalTauri: true` injects it into ALL windows automatically — no script tag needed

**Design Decision:** Option B. The `withGlobalTauri: true` config injects `window.__TAURI__` into every window, including the overlay. No need to load `vendor/tauri.js`. The overlay JS should use:
```js
const { listen } = window.__TAURI__.event;
```

**Impact on Plan:** Phase 2 must note: no `<script>` tag needed for Tauri APIs; use `window.__TAURI__` directly.

---

### GAP-14: Hidden Windows DO Receive Tauri Events — Show/Hide Timing Matters

**Severity:** Minor
**Phase:** Phase 3

**Codebase Reality:**

In Tauri v2, hidden windows still have active JS runtimes and receive events. This means:

1. The overlay JS sets up listeners on `DOMContentLoaded` (while window is hidden)
2. When `recording-started` is emitted, the overlay receives it and renders — even while hidden
3. Then `show_overlay()` makes the window visible — already in the correct state

**Design Decision:** Emit events first, then show/hide the window. This ensures the overlay renders the correct state before becoming visible (no flash of wrong state).

For `on_press()`:
```rust
app.emit("recording-started", ());  // overlay JS renders recording state
show_overlay(app);                   // window appears already in correct state
```

For `pipeline()` completion:
```rust
app.emit("transcription-complete", json);  // overlay JS starts exit animation
hide_overlay(app);                         // after 150ms delay, window hides
```

**Impact on Plan:** Phase 3 must specify event emission order: emit first, then show/hide.

---

### GAP-15: `on_press`/`on_release` Receive `&AppHandle`, Not Owned

**Severity:** Minor
**Phase:** Phase 3

**Codebase Reality:**

```rust
pub fn on_press(app: &AppHandle, state: &AppState) { ... }
pub fn on_release(app: &AppHandle, state: &AppState) { ... }
```

Overlay module functions must accept `&AppHandle` and `.clone()` for closures. This is trivial since `AppHandle` is `Clone + Send + Sync`.

**Impact on Plan:** Function signatures in overlay.rs should show `pub fn show_overlay(app: &AppHandle)`.

---

### GAP-16: Crate is lib+binary Split — Module Organization

**Severity:** Informational
**Phase:** Phase 0

**Codebase Reality:**

- `src-tauri/src/lib.rs` — exports `voice_transcribe_lib` with all modules (application, domain, infrastructure, presentation)
- `src-tauri/src/main.rs` — thin binary that calls into the lib

New modules (`overlay.rs`, `eq.rs`) go in the lib crate. The `create_overlay()` function is `voice_transcribe_lib::infrastructure::overlay::create_overlay()`. `main.rs` calls it from the `.setup()` closure via `app.handle()`.

**Impact on Plan:** No changes needed — file paths in the plan are already correct (`src-tauri/src/infrastructure/overlay.rs`).

---

### GAP-17: `realfft` Is Already a Transitive Dependency via `rubato`

**Severity:** Informational
**Phase:** Phase 1

**Codebase Reality:**

The `Cargo.lock` shows `rubato` v0.15.0 (used for audio resampling) depends directly on `realfft` v3.5.0 (no feature flag — it's a direct dependency, not behind a feature gate):
```
rubato 0.15.0 -> dependencies: [num-complex, num-integer, num-traits, realfft]
realfft 3.5.0 -> dependencies: [rustfft]
```

Adding `realfft = "3"` to `Cargo.toml` as a direct dependency won't increase compile time — it's already compiled. The plan correctly identifies it as a new direct dependency.

**Impact on Plan:** Add a note in Dependencies section that `realfft` is already compiled transitively via `rubato` v0.15.0 (direct dependency, not feature-gated).

---

## Summary

### v1.0 Gaps (all addressed in v2.0)

| Gap | Severity | Status | One-line Summary |
|-----|----------|--------|-----------------|
| GAP-1 | **Critical** | ✅ Fixed | Capabilities/permissions — overlay window needs explicit permissions |
| GAP-2 | **Critical** | ✅ Fixed | `pipeline()` is blocking thread — need `run_on_main_thread()` |
| GAP-3 | **Critical** | ✅ Fixed | RMS gives 1 value — FFT required for multi-band EQ |
| GAP-4 | **Critical** | ✅ Fixed | Transparency needs `.shadow(false)` + explicit CSS on both `<html>` and `<body>` |
| GAP-5 | Significant | ✅ Fixed | `WebviewUrl::App` strategy for dev vs production |
| GAP-6 | Significant | ✅ Fixed | GTK Layer Shell at window creation, not retrofitted |
| GAP-7 | Significant | ✅ Fixed | Level emission architecture — concrete channel + task design |
| GAP-8 | Minor | ✅ Fixed | Single-instance callback should not focus overlay |
| GAP-9 | Minor | ✅ Fixed | Phase 1/3 responsibility split for level emission |

### v3.0 Gaps (found during codebase grounding)

| Gap | Severity | Status | One-line Summary |
|-----|----------|--------|-----------------|
| GAP-10 | Significant | ✅ Addressed | `RecorderHandle` needs `eq_rx` channel — created in `spawn_for_device`, not in `start()` |
| GAP-11 | Significant | ✅ Addressed | `EqState` created per-recording inside `Start` arm of recorder thread |
| GAP-12 | Significant | ✅ Addressed | Level task cancellation: `Arc<AtomicBool>` on `AppState` |
| GAP-13 | Minor | ✅ Addressed | Overlay uses `window.__TAURI__` directly — no `<script>` tag needed |
| GAP-14 | Minor | ✅ Addressed | Hidden windows receive events — emit first, then show/hide |
| GAP-15 | Minor | ✅ Addressed | `on_press`/`on_release` receive `&AppHandle` references |
| GAP-16 | Informational | ✅ Noted | lib+binary split — modules go in `voice_transcribe_lib` |
| GAP-17 | Informational | ✅ Noted | `realfft` already compiled transitively via `rubato` v0.15.0 (direct dep, not feature-gated) |

---

## Recommended Implementation Priority

1. **GAP-1** (Capabilities) — Must be fixed in Phase 0 or the overlay won't work
2. **GAP-2** (Threading) — Must be designed into `overlay.rs` from Phase 0
3. **GAP-3** (FFT) — Changes Phase 1 scope and adds dependency
4. **GAP-4** (Transparency) — Quick fix but blocks Phase 0 verification
5. **GAP-10 + GAP-11** (Recorder integration) — Critical for Phase 1 correctness
6. **GAP-12** (Cancellation) — Needed for Phase 3
7. **GAP-6** (GTK Layer Shell timing) — Restructured in Phase 0
8. **GAP-7** (Level architecture) — Concrete design for Phase 1/3
9. **GAP-13–17** — Can be resolved during execution
