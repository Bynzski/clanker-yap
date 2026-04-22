# Voice Transcription Desktop App — Plan

**Author:** jay
**Date:** 2026-04-22
**Status:** Approved
**Version:** 2.2

---

## Purpose

A lightweight, single-purpose desktop app for local voice-to-text dictation. The user holds a global hotkey to record audio, whisper.cpp transcribes it locally, and the resulting text is auto-pasted into the focused input field.

**Pipeline (push-to-talk):**

```
Hotkey Pressed (down) → Start recording
Hotkey Released (up)  → Stop recording → Transcribe (whisper.cpp) → Paste text → Save to history
```

Key goals:

1. **Privacy** — 100% local, no network calls for transcription.
2. **Low resource** — Optimized for older hardware (`base.en` model default).
3. **Minimal UI** — Status + settings only; no distractions.

---

## Scope

### In Scope

| Item | Priority | Notes |
|------|----------|-------|
| Tauri v2 project scaffold | P0 | Cross-platform shell |
| whisper.cpp integration | P0 | Local transcription via `whisper-rs` |
| Global hotkey (push-to-talk) | P0 | Press = start, release = stop |
| Microphone recording | P0 | Resampled to 16 kHz mono f32 |
| Text injection | P0 | Clipboard + simulated paste via `enigo` |
| Settings UI (read + update) | P1 | Change hotkey + model path from UI |
| SQLite storage | P1 | Settings (single-row JSON) + transcription history |
| Single-instance enforcement | P0 | Prevents duplicate hotkey registration |

### Out of Scope

- Ollama or any external service
- LLM post-processing, summarization, command parsing
- In-app model downloading (user fetches separately)
- System tray / background-only mode
- Autostart on login (future work)
- Advanced audio processing (VAD, noise reduction)
- Full transcription-history management UI (view last 10 only)
- Code signing / notarization / installer generation
- Multi-language models (English-only for v1)

---

## What Already Exists

Nothing — `/home/jay/dev/projects/clanker-yap/` contains only `plans/`. The folder is **not yet a git repository**. Prereq initialises the repo and the Tauri project.

---

## Dependencies

| Dependency | Status | Notes |
|------------|--------|-------|
| Rust toolchain (stable) | Required | `rustc`, `cargo` on PATH |
| Node.js ≥ 20 + npm | Required | For `create-tauri-app` scaffold + `@tauri-apps/cli` |
| cmake + C++ toolchain | Required | `whisper-rs` builds whisper.cpp from source |
| Linux system libs | Required (Linux) | `webkit2gtk-4.1`, `libayatana-appindicator3-dev`, `librsvg2-dev`, `build-essential`, `pkg-config`, `libasound2-dev` (cpal on ALSA) |
| `xdotool` or `ydotool`/`wtype` | Optional (Linux) | Only needed if `enigo` fallback fails; see Paste Strategy |
| GGML model file | Blocked | User places `ggml-base.en.bin` in app data dir after install |

Host setup (Linux, CachyOS / Arch-family — run once before Prereq):

```sh
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 libappindicator-gtk3 librsvg nodejs npm
rustup default stable
npm install -g @tauri-apps/cli@latest
```

For other distros, substitute the equivalent packages (Ubuntu: `libwebkit2gtk-4.1-dev libayatana-appindicator3-dev librsvg2-dev build-essential curl wget file libssl-dev libasound2-dev`).

---

## Cross-Cutting Concerns

### Error Handling — `AppError`

Unified error type across all modules. File: `src-tauri/src/domain/error.rs`.

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Whisper error: {0}")]
    Whisper(String),

    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Model not found at: {0}")]
    ModelNotFound(String),

    #[error("Microphone unavailable")]
    MicrophoneUnavailable,

    #[error("Paste injection failed: {0}")]
    PasteFailed(String),

    #[error("Settings invalid: {0}")]
    SettingsInvalid(String),
}

pub type Result<T> = std::result::Result<T, AppError>;

// Tauri commands must return a serialisable error.
impl serde::Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        s.serialize_str(&self.to_string())
    }
}
```

### `AppState`

Single struct holding shared resources, wrapped in `tauri::State` at startup. File: `src-tauri/src/application/state.rs`.

> **Thread-safety notes**
> - `cpal::Stream` is `!Send`. We do **not** store a `Stream` in `AppState`. Instead the recorder owns an internal worker thread and communicates with the stream via message channels; `AppState` holds a handle to that worker.
> - `whisper_rs::WhisperContext` is `Send + Sync`, but `WhisperState` is not. We store the context; each transcription creates a fresh state on a blocking worker.
> - We use `parking_lot::Mutex` for non-poisoning + slightly better perf.

```rust
use std::sync::Arc;
use parking_lot::Mutex;
use crate::domain::settings::Settings;
use crate::infrastructure::whisper::WhisperEngine;
use crate::infrastructure::audio::RecorderHandle;
use crate::infrastructure::persistence::Db;

pub struct AppState {
    pub db: Arc<Db>,
    pub settings: Arc<Mutex<Settings>>,
    pub whisper: Arc<Mutex<Option<Arc<WhisperEngine>>>>,  // lazy-loaded
    pub recorder: Arc<Mutex<Option<RecorderHandle>>>,     // cpal worker handle, not the Stream
    pub recording: Arc<Mutex<RecordingState>>,            // Idle | Recording { started_at } | Processing
}

pub enum RecordingState {
    Idle,
    Recording { started_at: std::time::Instant },
    Processing,
}

impl AppState {
    pub fn new(db: Db, settings: Settings) -> Self { /* ... */ }
}
```

### Logging

`tracing` + `tracing_subscriber`. Initialised once at app start from `lib.rs::init_logging()`:

```rust
use tracing_subscriber::{fmt, EnvFilter};

pub fn init_logging() {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("voice_transcribe=info,warn"));
    fmt().with_env_filter(filter).with_target(false).init();
}
```

### Shutdown Handling

Handled via the Tauri `RunEvent::ExitRequested` hook:

1. If `RecordingState::Recording`, stop the recorder worker (abandoning buffered audio).
2. Drop `Arc<WhisperEngine>` — releases whisper context.
3. SQLite is write-through (every statement auto-commits); no explicit flush needed.
4. `AppState` drops via the `tauri::State` registry.

### Platform Detection

Paste uses the `enigo` crate which abstracts per-OS keystroke simulation (X11, Wayland via libei, macOS CGEvent, Windows SendInput). **No shelling out to `osascript` / `xdotool`.** See Paste Strategy below.

File-system paths use the `dirs` crate (`dirs::data_dir()`):

- macOS: `~/Library/Application Support/voice-transcribe/`
- Linux: `~/.local/share/voice-transcribe/`
- Windows: `%APPDATA%\voice-transcribe\`

A single helper `fn app_data_dir() -> Result<PathBuf>` is defined in `infrastructure/persistence/paths.rs` and used everywhere.

### Constants — `src-tauri/src/domain/constants.rs`

```rust
pub const MAX_HISTORY_ITEMS: u32 = 10;
pub const MAX_TRANSCRIPTION_LENGTH: usize = 10_000;   // chars
pub const MIN_RECORDING_DURATION_MS: i64 = 150;       // below this, likely accidental press
pub const MAX_RECORDING_DURATION_MS: i64 = 60_000;    // hard cap, auto-stop
pub const WHISPER_SAMPLE_RATE: u32 = 16_000;          // required by whisper.cpp
pub const DEFAULT_MODEL_FILE: &str = "ggml-base.en.bin";
pub const DEFAULT_HOTKEY: &str = "CmdOrCtrl+Shift+V";
pub const APP_DATA_SUBDIR: &str = "voice-transcribe";
```

### Push-to-Talk vs Toggle — RESOLVED

**Decision:** push-to-talk. The hotkey's **pressed** event starts recording; the **released** event stops it and kicks off transcription. No toggle mode in v1.

- Rationale: matches README; avoids a stuck "forgot to stop" state; maps to how users dictate.
- `tauri-plugin-global-shortcut` v2 exposes `ShortcutEvent { state: Pressed | Released }`; we handle both.
- If `MAX_RECORDING_DURATION_MS` elapses while held, the recorder auto-stops and transitions to Processing; a subsequent release is a no-op.

### Single-Instance Enforcement

`tauri-plugin-single-instance` registered in `main.rs`. On duplicate launch: focus the existing window and emit `app-already-running`. Without this, the second process silently fails to register the hotkey and the user has two zombies.

### Concurrency Rules

1. Hotkey callback runs on Tauri's main loop. It must not block — it only flips state and sends a message to the recorder worker.
2. Transcription is CPU-bound. It runs inside `tauri::async_runtime::spawn_blocking` so it never blocks the async command runtime or the hotkey loop.
3. Audio capture runs on a dedicated `std::thread` owned by `RecorderHandle`; the `cpal::Stream` lives on that thread and never crosses a thread boundary.

---

## Implementation Plan

### Phase Order

| Phase | Description | Depends On |
|-------|-------------|------------|
| Prereq | `git init`, host deps verified, Tauri scaffold, Cargo.toml deps, directory layout, `AppError` + `AppState` + constants | — |
| 0 | Settings module + SQLite storage + first-run bootstrap | Prereq |
| 1 | whisper.cpp integration (`WhisperEngine`, lazy load, `spawn_blocking`) | Prereq |
| 2 | Audio capture (cpal worker thread, resampling to 16 kHz mono f32) | Prereq |
| 3 | Global hotkey (push-to-talk) + orchestration (record → transcribe → paste → save) | Phase 1, Phase 2, Phase 5 |
| 4 | Frontend UI (vanilla HTML/JS/CSS, settings view + update, history, status) | Phase 0 |
| 5 | Text injection (`enigo`-based paste service) | Prereq |

**Note:** Phases 1, 2, 4, 5 can be developed in parallel after Prereq + Phase 0. Phase 3 is the integration point.

---

## Phase Details

### Phase Prereq — Project Scaffold

**Purpose:** Bring the empty folder to "buildable empty Tauri app" with all shared scaffolding in place.

**Scope:**

- [ ] `git init` in `/home/jay/dev/projects/clanker-yap/`; create `.gitignore` with:
  ```
  target/
  node_modules/
  dist/
  .DS_Store
  *.log
  src-tauri/gen/
  ```
- [ ] Verify host deps present: `cargo --version`, `node --version`, `npm --version`, `cmake --version`, `pkg-config --version`. Fail fast if any missing (do not attempt `sudo`).
- [ ] Scaffold Tauri v2 project into the existing folder:
  ```sh
  npm create tauri-app@latest -- --template vanilla --manager npm --identifier dev.jay.voice-transcribe --yes .
  ```
  (Overwrite template `README.md` only if it would clobber — otherwise leave Tauri's and add ours under `plans/`.)
- [ ] Pin Rust dependencies in `src-tauri/Cargo.toml`:
  ```toml
  [dependencies]
  tauri          = { version = "2", features = [] }
  tauri-plugin-global-shortcut    = "2"
  tauri-plugin-clipboard-manager  = "2"
  tauri-plugin-single-instance    = "2"
  serde          = { version = "1", features = ["derive"] }
  serde_json     = "1"
  thiserror      = "1"
  parking_lot    = "0.12"
  tracing        = "0.1"
  tracing-subscriber = { version = "0.3", features = ["env-filter"] }
  rusqlite       = { version = "0.32", features = ["bundled"] }
  uuid           = { version = "1", features = ["v4"] }
  chrono         = { version = "0.4", features = ["serde"] }
  dirs           = "5"
  cpal           = "0.15"
  hound          = "3.5"
  rubato         = "0.15"                # sample-rate conversion (Phase 2)
  whisper-rs     = "0.14"                # pin to a tested major
  enigo          = "0.2"                 # cross-platform keystroke sim (Phase 5)
  ```
- [ ] Create directory layout under `src-tauri/src/`:
  ```
  lib.rs
  main.rs
  domain/
    mod.rs
    error.rs
    constants.rs
    settings.rs          # Phase 0 fills
    transcription.rs     # Phase 0 fills
  application/
    mod.rs
    state.rs
    use_cases/
      mod.rs             # Phase 0+ fills
    orchestrator.rs      # Phase 3 fills
  infrastructure/
    mod.rs
    persistence/
      mod.rs
      paths.rs           # app_data_dir()
      db.rs              # Phase 0 fills
      settings_repo.rs   # Phase 0
      transcription_repo.rs # Phase 0
    whisper/
      mod.rs             # Phase 1
      engine.rs          # Phase 1
    audio/
      mod.rs             # Phase 2
      recorder.rs        # Phase 2
      resample.rs        # Phase 2
    paste/
      mod.rs             # Phase 5
      service.rs         # Phase 5
  presentation/
    mod.rs
    dto.rs
    commands/
      mod.rs
      settings_cmds.rs    # Phase 0
      transcription_cmds.rs # Phase 0
      lifecycle_cmds.rs   # Phase 3/4 (e.g. get_status)
  ```
- [ ] Implement now (skeletal, used by all phases):
  - `domain/error.rs` — full `AppError` from Cross-Cutting
  - `domain/constants.rs` — full constants block
  - `application/state.rs` — `AppState` + `RecordingState` (with empty `Option` slots for whisper/recorder)
  - `infrastructure/persistence/paths.rs` — `app_data_dir()` using `dirs::data_dir()`
  - `lib.rs` — `init_logging()`, module re-exports
  - `main.rs` — Tauri builder with plugins registered (global-shortcut, clipboard-manager, single-instance), `init_logging()` call, placeholder state (real `Db` + `Settings` come in Phase 0), `RunEvent::ExitRequested` hook
- [ ] Configure `src-tauri/tauri.conf.json`:
  ```jsonc
  {
    "productName": "Voice Transcription",
    "identifier": "dev.jay.voice-transcribe",
    "build": {
      "frontendDist": "../src",
      "devUrl": null,
      "beforeDevCommand": null,
      "beforeBuildCommand": null
    },
    "app": {
      "windows": [{
        "label": "main",
        "title": "Voice Transcription",
        "width": 480,
        "height": 560,
        "resizable": true,
        "decorations": true,
        "center": true
      }],
      "security": { "csp": null }
    }
  }
  ```
- [ ] Configure `src-tauri/capabilities/default.json`:
  ```json
  {
    "identifier": "default",
    "description": "Default capabilities",
    "windows": ["main"],
    "permissions": [
      "core:default",
      "global-shortcut:default",
      "clipboard-manager:default"
    ]
  }
  ```
  (No `shell:allow-execute` — we no longer shell out for paste.)
- [ ] macOS extras (commit even though we're developing on Linux; needed for macOS builds):
  - `src-tauri/Info.plist` snippet added to `tauri.conf.json`:
    ```json
    "macOS": {
      "entitlements": "./entitlements.plist",
      "infoPlist": {
        "NSMicrophoneUsageDescription": "Voice Transcription records audio so it can transcribe your speech locally."
      }
    }
    ```
  - `src-tauri/entitlements.plist` with `com.apple.security.device.audio-input = true`.
- [ ] `src/index.html` — minimal dark-themed shell showing "Voice Transcription — loading…" (Phase 4 replaces).
- [ ] Verify: `cd src-tauri && cargo check` passes; `cargo tauri build --debug` produces a binary.
- [ ] **Commit:** initial commit of scaffold.

**Out of Scope:** business logic, persistence, UI beyond loading shell, any use-case implementations.

---

### Phase 0 — Settings & Persistence

**Purpose:** Persist user settings and transcription history. Establish first-run bootstrap.

**Settings storage decision (RESOLVED):** one JSON blob in a single row. Simpler than per-field rows; trivially versioned.

```sql
CREATE TABLE IF NOT EXISTS app_settings (
    id          INTEGER PRIMARY KEY CHECK (id = 1),
    payload     TEXT NOT NULL,               -- JSON: { hotkey, model_path, model_name, schema_version }
    updated_at  TEXT NOT NULL                -- ISO 8601
);

CREATE TABLE IF NOT EXISTS transcriptions (
    id          TEXT PRIMARY KEY,            -- UUID v4
    text        TEXT NOT NULL,
    duration_ms INTEGER NOT NULL,
    created_at  TEXT NOT NULL                -- ISO 8601
);

CREATE INDEX IF NOT EXISTS idx_transcriptions_created_at
    ON transcriptions(created_at DESC);
```

**Scope:**

- [ ] `domain/settings.rs`:
  ```rust
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub struct Settings {
      pub hotkey: String,
      pub model_path: String,
      pub model_name: String,
      #[serde(default = "current_schema")]
      pub schema_version: u32,
  }
  impl Default for Settings { /* use DEFAULT_HOTKEY, DEFAULT_MODEL_FILE, data_dir join */ }
  ```
- [ ] `domain/transcription.rs` with validation against `MAX_TRANSCRIPTION_LENGTH`, `MIN_/MAX_RECORDING_DURATION_MS` (`Transcription::new` returns `Result`).
- [ ] `infrastructure/persistence/db.rs`:
  - `Db::open()` ensures `app_data_dir()` exists, opens `voice-transcribe.db`, runs `CREATE TABLE IF NOT EXISTS` statements.
  - `Db` exposes `conn()` returning a guarded `&Connection` (wrapped in `parking_lot::Mutex<Connection>`; swap to `r2d2` only if contention becomes a problem).
- [ ] `infrastructure/persistence/settings_repo.rs`:
  - `load_or_init(db) -> Result<Settings>` — if `app_settings` row 1 is missing, insert `Settings::default()` and return it. This is the **first-run bootstrap**.
  - `save(db, settings) -> Result<()>`.
- [ ] `infrastructure/persistence/transcription_repo.rs`:
  - `save(db, t) -> Result<()>`.
  - `recent(db, limit) -> Result<Vec<Transcription>>` ordered by `created_at DESC`.
  - `prune_to(db, keep: u32) -> Result<()>` called after each save; keeps only newest N rows (enforces `MAX_HISTORY_ITEMS` at write time).
- [ ] Use cases (thin wrappers, all returning `Result<_, AppError>`):
  - `GetSettingsUseCase`, `UpdateSettingsUseCase` (validates hotkey non-empty, model_path exists when set).
  - `SaveTranscriptionUseCase` (validates text length), `GetTranscriptionHistoryUseCase` (defaults to `MAX_HISTORY_ITEMS`).
- [ ] Wire `Db::open()` and `SettingsRepository::load_or_init()` into `main.rs`; replace Prereq's placeholder state with the real `AppState::new(db, settings)`.
- [ ] DTOs in `presentation/dto.rs`:
  ```rust
  #[derive(Serialize)]
  pub struct SettingsResponse { pub hotkey: String, pub model_path: String, pub model_name: String }
  #[derive(Deserialize)]
  pub struct UpdateSettingsRequest { pub hotkey: Option<String>, pub model_path: Option<String>, pub model_name: Option<String> }
  #[derive(Serialize)]
  pub struct UpdateSettingsResponse { pub success: bool, pub message: String, pub requires_restart: bool }
  #[derive(Serialize)]
  pub struct TranscriptionItem { pub id: String, pub text: String, pub duration_ms: i64, pub created_at: String }
  #[derive(Serialize)]
  pub struct TranscriptionHistoryResponse { pub transcriptions: Vec<TranscriptionItem> }
  ```
  `requires_restart` is `true` whenever hotkey changes (global-shortcut re-registration flow is deferred to Phase 3; for v1 we surface "restart required" in the UI).
- [ ] Tauri commands (registered in `main.rs::invoke_handler`):
  - `get_settings`, `update_settings`, `get_transcription_history`.

**Out of Scope:** UI, whisper.cpp, audio capture.

---

### Phase 1 — whisper.cpp Integration

**Purpose:** Load `ggml-base.en.bin` and transcribe 16 kHz mono f32 audio.

**Model loading strategy:** lazy — load on first transcription, cache in `AppState.whisper`.

**Real API reference (whisper-rs 0.14):**

```rust
use whisper_rs::{WhisperContext, WhisperContextParameters, FullParams, SamplingStrategy};

pub struct WhisperEngine {
    ctx: WhisperContext,
}

impl WhisperEngine {
    pub fn load(model_path: &str) -> Result<Self> {
        if !std::path::Path::new(model_path).exists() {
            return Err(AppError::ModelNotFound(model_path.into()));
        }
        let ctx = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
            .map_err(|e| AppError::Whisper(format!("load: {e}")))?;
        Ok(Self { ctx })
    }

    /// `samples` must already be 16 kHz mono f32 in [-1, 1]. Phase 2 guarantees this.
    pub fn transcribe(&self, samples: &[f32]) -> Result<String> {
        let mut state = self.ctx.create_state()
            .map_err(|e| AppError::Whisper(format!("create_state: {e}")))?;
        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });
        params.set_n_threads(num_cpus::get_physical() as i32);
        params.set_language(Some("en"));
        params.set_translate(false);
        params.set_print_special(false);
        params.set_print_progress(false);
        params.set_print_realtime(false);
        params.set_print_timestamps(false);

        let t0 = std::time::Instant::now();
        state.full(params, samples)
            .map_err(|e| AppError::Whisper(format!("full: {e}")))?;
        let n = state.full_n_segments()
            .map_err(|e| AppError::Whisper(format!("n_segments: {e}")))?;
        let mut out = String::new();
        for i in 0..n {
            out.push_str(&state.full_get_segment_text(i)
                .map_err(|e| AppError::Whisper(format!("seg_text: {e}")))?);
        }
        tracing::info!(samples = samples.len(), elapsed_ms = t0.elapsed().as_millis() as u64, "transcribed");
        Ok(out.trim().to_string())
    }
}
```

(Add `num_cpus = "1"` to `Cargo.toml`.)

**Scope:**

- [ ] `infrastructure/whisper/engine.rs` with the above `WhisperEngine`.
- [ ] `infrastructure/whisper/mod.rs` exporting `WhisperEngine` + a helper:
  ```rust
  pub fn load_or_get(state: &AppState) -> Result<Arc<WhisperEngine>> {
      let mut slot = state.whisper.lock();
      if let Some(engine) = slot.as_ref() { return Ok(engine.clone()); }
      let path = state.settings.lock().model_path.clone();
      let engine = Arc::new(WhisperEngine::load(&path)?);
      *slot = Some(engine.clone());
      Ok(engine)
  }
  ```
- [ ] `application/use_cases/transcribe.rs` — `TranscribeUseCase::execute(samples)` wraps the engine call inside `tauri::async_runtime::spawn_blocking` so the async runtime is not blocked.
- [ ] Duration validation before calling whisper:
  - samples.len() / 16_000 * 1000 < `MIN_RECORDING_DURATION_MS` → `AppError::Whisper("Audio too short")`
  - `> MAX_RECORDING_DURATION_MS` → truncate or reject (reject for v1, already prevented by recorder auto-stop).
- [ ] `tracing::info!` for load time + transcription time + sample count.

**Performance targets:** < 2× realtime on `base.en`; < 500 MB RSS.

**Out of Scope:** audio capture, paste, settings UI.

---

### Phase 2 — Audio Capture

**Purpose:** Record microphone audio at the device's native rate, resample to 16 kHz mono f32, return samples to the orchestrator.

**Design:** `RecorderHandle` owns a dedicated `std::thread` that hosts the `cpal::Stream` (which is `!Send`). Orchestrator communicates via `crossbeam_channel` messages (add `crossbeam-channel = "0.5"` to deps if not pulled transitively).

```rust
pub struct RecorderHandle {
    cmd_tx: crossbeam_channel::Sender<RecorderCmd>,
    /// Results delivered here when a recording finishes.
    result_rx: crossbeam_channel::Receiver<Result<Vec<f32>>>,
    _join: std::thread::JoinHandle<()>,
}

enum RecorderCmd { Start, Stop, Shutdown }

impl RecorderHandle {
    pub fn spawn() -> Result<Self> { /* picks default input device, opens stream lazily */ }
    pub fn start(&self) -> Result<()> { self.cmd_tx.send(RecorderCmd::Start).ok(); Ok(()) }
    pub fn stop_and_collect(&self) -> Result<Vec<f32>> { /* send Stop, recv result */ }
}
```

**Scope:**

- [ ] `infrastructure/audio/recorder.rs`:
  - Pick default input device via `cpal::default_host().default_input_device()`. Return `AppError::MicrophoneUnavailable` if `None`.
  - Query supported config; prefer f32 mono if available, otherwise match the device format and convert.
  - Handle all three cpal sample formats: `I16`, `U16`, `F32`. Each callback converts to f32 in [-1, 1] and appends to an inner `Vec<f32>` guarded by a `parking_lot::Mutex`.
  - Down-mix stereo → mono by averaging channels inside the callback.
  - On `Start`: clear buffer, build + `play()` the stream.
  - On `Stop`: `drop(stream)`, take buffer, **resample** to 16 kHz (`infrastructure/audio/resample.rs`, `rubato::SincFixedIn`), return via `result_rx`.
  - Auto-stop safeguard: if buffer duration exceeds `MAX_RECORDING_DURATION_MS`, emit a "stopped-by-limit" message and deliver the result anyway.
- [ ] `infrastructure/audio/resample.rs`:
  - Thin wrapper around `rubato::SincFixedIn<f32>` converting `input_rate → 16_000`. Short-circuits if input is already 16 kHz.
- [ ] Errors (`AppError` variants):
  - `MicrophoneUnavailable` — no device.
  - `Audio("Permission denied")` — OS-level denial (cpal surfaces `BuildStreamError::BackendSpecific`).
  - `Audio("Recording too short")` — final duration < `MIN_RECORDING_DURATION_MS`.
  - `Audio("Stream build failed: ...")` — anything else from cpal.
- [ ] Unit test: build recorder in offline mode with a synthesised sine-wave buffer (no device), verify resampler path end-to-end. Real-device test is manual.

**Out of Scope:** hotkey wiring, UI, whisper.

---

### Phase 3 — Global Hotkey & Orchestration (Push-to-Talk)

**Purpose:** Wire hotkey press/release to the record → transcribe → paste → save pipeline.

**Scope:**

- [ ] Register global shortcut via `tauri-plugin-global-shortcut` using `AppState.settings.hotkey`.
  - Handler receives `ShortcutEvent { state }`.
  - `ShortcutState::Pressed` → `orchestrator::on_press(app_handle, state)`
  - `ShortcutState::Released` → `orchestrator::on_release(app_handle, state)`
- [ ] `application/orchestrator.rs`:
  - `on_press`:
    1. Lock `state.recording`. If not `Idle`, ignore (debounce).
    2. Ensure `state.recorder` has a `RecorderHandle` (spawn on first use).
    3. Call `recorder.start()`.
    4. Transition to `Recording { started_at }`.
    5. Emit frontend event `recording-started`.
  - `on_release`:
    1. Lock `state.recording`. Only proceed if `Recording`. Compute duration.
    2. Transition to `Processing`.
    3. Emit `recording-stopped` with `duration_ms`.
    4. `spawn_blocking`:
       - `samples = recorder.stop_and_collect()?`
       - Validate duration vs `MIN_RECORDING_DURATION_MS`.
       - `engine = whisper::load_or_get(state)?`
       - `text = engine.transcribe(&samples)?`
       - `paste::inject(&text)` (log + emit error on failure, continue)
       - `save_transcription_use_case.execute(text, duration_ms)`
    5. On success: emit `transcription-complete` with text. On failure: emit `transcription-error` with message.
    6. Transition to `Idle`.
- [ ] Debounce / race handling:
  - Rapid re-press during `Processing` is ignored (logged at `debug`).
  - If `on_release` arrives without a matching `Recording` state (lost press event), log warning and ignore.
- [ ] Shutdown: `RunEvent::ExitRequested`
  - If `Recording`, send `RecorderCmd::Shutdown`.
  - Drop whisper engine.
- [ ] Hotkey conflict: plugin registration returns `Err`. Log warning, emit `hotkey-conflict` event to frontend (Phase 4 surfaces it).
- [ ] Hotkey change at runtime (via `update_settings`): unregister old shortcut, register new one; roll back on failure. If rollback succeeds, respond with `requires_restart: false`; else `true`.

**Frontend events:**

```
{ event: "recording-started", payload: {} }
{ event: "recording-stopped", payload: { duration_ms: 1500 } }
{ event: "transcription-complete", payload: { text: "Hello world" } }
{ event: "transcription-error", payload: { error: "..." } }
{ event: "hotkey-conflict", payload: { hotkey: "CmdOrCtrl+Shift+V" } }
```

**Out of Scope:** paste implementation details (Phase 5), UI (Phase 4).

---

### Phase 4 — Frontend UI

**Purpose:** Read-write settings UI with live status and recent-history display.

**Tauri API access:** vanilla template, no bundler. Use the pre-bundled global:

```html
<script src="https://cdn.jsdelivr.net/npm/@tauri-apps/api@2/dist/tauri.min.js"></script>
<!-- then: const { invoke } = window.__TAURI__.core; const { listen } = window.__TAURI__.event; -->
```

Alternatively ship `tauri.min.js` locally under `src/vendor/` and reference it — preferred for offline-first.

**Scope:**

- [ ] `src/index.html` — dark theme, ~480×560, layout:
  ```
  🎤 Voice Transcription
  [Status: Idle ●]                 (colour-coded)
  Hotkey:     [CmdOrCtrl+Shift+V] [Change…]
  Model:      base.en
  Model path: /path/to/ggml-base.en.bin [Change…]
  ─────────────────────────────
  Recent transcriptions (last 10):
    • "Hello world" — 2s ago
    • "Meeting at 3pm" — 5m ago
  ─────────────────────────────
  [error banner, hidden unless active]
  ```
- [ ] `src/style.css` — dark theme, system font stack, no external deps.
- [ ] `src/main.js`:
  - On load: `invoke('get_settings')` + `invoke('get_transcription_history')`.
  - `listen('recording-started'|'recording-stopped'|'transcription-complete'|'transcription-error'|'hotkey-conflict', ...)` updates status + history.
  - [Change…] buttons open inline inputs; submit calls `invoke('update_settings', { request: {...} })`. If response says `requires_restart`, show "Restart required" banner.
  - Error banner for: `ModelNotFound` (with download instructions link to huggingface.co), `MicrophoneUnavailable`, `HotkeyConflict`, generic `transcription-error`.
- [ ] Transcription list item: truncate text to 80 chars with ellipsis; relative timestamp ("2s ago", "5m ago") computed client-side.
- [ ] No router, no framework, no npm install step needed.

**Out of Scope:** full history view, advanced settings, model download UI, theme picker.

---

### Phase 5 — Text Injection (via `enigo`)

**Purpose:** Paste transcribed text into the focused app uniformly across platforms, with no shell-outs.

**Why `enigo`:** one API for macOS (CGEvent), Windows (SendInput), Linux X11 (XTest), Linux Wayland (libei). Eliminates the wrong `cmd /c "echo v | clip"` and the X11-only `xdotool` from v2.1.

**Scope:**

- [ ] `infrastructure/paste/service.rs`:
  ```rust
  use enigo::{Enigo, Keyboard, Settings, Key, Direction};
  use tauri::AppHandle;
  use tauri_plugin_clipboard_manager::ClipboardExt;

  pub fn inject(app: &AppHandle, text: &str) -> Result<()> {
      app.clipboard().write_text(text.to_string())
         .map_err(|e| AppError::PasteFailed(format!("clipboard: {e}")))?;
      let mut enigo = Enigo::new(&Settings::default())
         .map_err(|e| AppError::PasteFailed(format!("enigo init: {e}")))?;
      let paste_modifier = if cfg!(target_os = "macos") { Key::Meta } else { Key::Control };
      enigo.key(paste_modifier, Direction::Press)
         .and_then(|_| enigo.key(Key::Unicode('v'), Direction::Click))
         .and_then(|_| enigo.key(paste_modifier, Direction::Release))
         .map_err(|e| AppError::PasteFailed(format!("keystroke: {e}")))?;
      Ok(())
  }
  ```
- [ ] `application/use_cases/paste.rs` — `PasteUseCase` wraps the above; orchestrator calls it. On `PasteFailed`, orchestrator logs + emits `transcription-error` but still saves the transcription (text is in clipboard so user can Ctrl+V manually).
- [ ] Permission doc (shown in `plans/voice-transcribe/README.md`, surfaced in UI on first failure):
  - **macOS:** grant Accessibility permission (System Settings → Privacy → Accessibility → add Voice Transcription).
  - **Linux/Wayland:** depends on compositor; `enigo` uses libei where available, otherwise falls back. If it fails, installing `wtype` + `ydotool` is the documented user workaround.
  - **Linux/X11:** works out-of-the-box.
  - **Windows:** no additional permission.

**Out of Scope:** direct input-field targeting, OCR, accessibility-API hooking.

---

## Tauri Commands Summary

| Command | Request | Response | Notes |
|---------|---------|----------|-------|
| `get_settings` | — | `SettingsResponse` | Phase 0 |
| `update_settings` | `UpdateSettingsRequest` | `UpdateSettingsResponse` | Phase 0; Phase 3 adds live hotkey re-register |
| `get_transcription_history` | — | `TranscriptionHistoryResponse` | Phase 0 |
| `get_status` | — | `{ state: "Idle" \| "Recording" \| "Processing", last_error?: string }` | Phase 3 — polled on load by UI |

All commands return `Result<T, AppError>`; `AppError` serialises to a string on the JS side.

---

## File Structure

Legend: ✅ exists, 🔧 modify, 🆕 new (created during phases).

```
/ (git repo root)
├── .gitignore                          🆕 Prereq
├── plans/voice-transcribe/             ✅ plan files
├── package.json                        🆕 Prereq (tauri scaffold)
├── src/                                🆕 Prereq (scaffold) → 🔧 Phase 4
│   ├── index.html
│   ├── style.css
│   ├── main.js
│   └── vendor/tauri.min.js             🆕 Phase 4 (optional local copy)
├── src-tauri/
│   ├── Cargo.toml                      🆕 Prereq
│   ├── tauri.conf.json                 🆕 Prereq
│   ├── entitlements.plist              🆕 Prereq
│   ├── build.rs                        (Tauri-generated; no custom content)
│   ├── capabilities/default.json       🆕 Prereq
│   └── src/
│       ├── main.rs                     🆕 Prereq → 🔧 each phase registers cmds
│       ├── lib.rs                      🆕 Prereq
│       ├── domain/
│       │   ├── mod.rs, error.rs, constants.rs  🆕 Prereq
│       │   ├── settings.rs             🆕 Phase 0
│       │   └── transcription.rs        🆕 Phase 0
│       ├── application/
│       │   ├── mod.rs, state.rs        🆕 Prereq
│       │   ├── orchestrator.rs         🆕 Phase 3
│       │   └── use_cases/
│       │       ├── mod.rs              🆕 Prereq
│       │       ├── settings.rs         🆕 Phase 0
│       │       ├── transcription.rs    🆕 Phase 0
│       │       ├── transcribe.rs       🆕 Phase 1
│       │       └── paste.rs            🆕 Phase 5
│       ├── infrastructure/
│       │   ├── mod.rs                  🆕 Prereq
│       │   ├── persistence/
│       │   │   ├── mod.rs, paths.rs    🆕 Prereq
│       │   │   ├── db.rs               🆕 Phase 0
│       │   │   ├── settings_repo.rs    🆕 Phase 0
│       │   │   └── transcription_repo.rs 🆕 Phase 0
│       │   ├── whisper/
│       │   │   ├── mod.rs, engine.rs   🆕 Phase 1
│       │   ├── audio/
│       │   │   ├── mod.rs, recorder.rs, resample.rs 🆕 Phase 2
│       │   └── paste/
│       │       ├── mod.rs, service.rs  🆕 Phase 5
│       └── presentation/
│           ├── mod.rs, dto.rs          🆕 Prereq (dto.rs stub)
│           └── commands/
│               ├── mod.rs              🆕 Prereq
│               ├── settings_cmds.rs    🆕 Phase 0
│               ├── transcription_cmds.rs 🆕 Phase 0
│               └── lifecycle_cmds.rs   🆕 Phase 3
└── (user downloads) ~/.local/share/voice-transcribe/ggml-base.en.bin
```

---

## Dependency Summary

| Crate | Version | Purpose | Added in |
|-------|---------|---------|----------|
| `tauri` | 2 | Desktop shell | Prereq |
| `tauri-plugin-global-shortcut` | 2 | Global hotkey | Prereq |
| `tauri-plugin-clipboard-manager` | 2 | Clipboard access | Prereq |
| `tauri-plugin-single-instance` | 2 | Singleton guard | Prereq |
| `serde` / `serde_json` | 1 / 1 | Serialisation | Prereq |
| `thiserror` | 1 | `AppError` | Prereq |
| `parking_lot` | 0.12 | Non-poisoning mutex | Prereq |
| `tracing` / `tracing-subscriber` | 0.1 / 0.3 | Logging | Prereq |
| `rusqlite` (+`bundled`) | 0.32 | SQLite | Prereq |
| `uuid` | 1 | IDs | Prereq |
| `chrono` | 0.4 | Timestamps | Prereq |
| `dirs` | 5 | Platform paths | Prereq |
| `cpal` | 0.15 | Audio capture | Prereq (used Phase 2) |
| `hound` | 3.5 | WAV (debug dumps) | Prereq |
| `rubato` | 0.15 | Resampling | Prereq (used Phase 2) |
| `whisper-rs` | 0.14 | whisper.cpp bindings | Prereq (used Phase 1) |
| `num_cpus` | 1 | Thread count | Phase 1 |
| `enigo` | 0.2 | Cross-platform keystroke | Prereq (used Phase 5) |
| `crossbeam-channel` | 0.5 | Recorder messaging | Prereq (used Phase 2) |

All added in Prereq so agents don't need to touch `Cargo.toml` mid-phase (honours the Hard Stop rule "no new Cargo deps per phase").

---

## Model Guide

| Model | Size | Speed | Accuracy | Use |
|-------|------|-------|----------|-----|
| `ggml-base.en.bin` | ~148 MB | Fast | Good | **Default** |
| `ggml-small.en.bin` | ~488 MB | Medium | Better | Optional upgrade |

User downloads from https://huggingface.co/ggerganov/whisper.cpp/tree/main and places the file in the platform data dir. The app detects missing models at launch (cheap `Path::exists()` on `settings.model_path`) and the UI prompts with a download hint before the user presses the hotkey.

---

## First-Run Experience

1. User installs app, launches it.
2. `Db::open()` creates `~/.local/share/voice-transcribe/voice-transcribe.db`.
3. `SettingsRepository::load_or_init()` inserts defaults.
4. UI loads. Status reads "Model not found" because `ggml-base.en.bin` is missing.
5. UI shows a banner: "Download `ggml-base.en.bin` from [link] and place it at `~/.local/share/voice-transcribe/ggml-base.en.bin`."
6. User downloads, hits refresh, status becomes "Idle".
7. User presses hotkey → recording → transcription → paste.

---

## Platform-Specific Notes

### macOS (future, not dev target)

- `NSMicrophoneUsageDescription` set via `tauri.conf.json`.
- `com.apple.security.device.audio-input` entitlement.
- Accessibility permission required for `enigo` keystroke simulation; surfaced in UI on first `PasteFailed`.

### Linux (primary dev target — CachyOS)

- System deps installed upstream of Prereq (see Dependencies table).
- cpal uses ALSA by default; PipeWire/PulseAudio work via their ALSA shims.
- Wayland: `enigo` uses libei; if compositor denies, user installs `wtype`/`ydotool` as fallback.

### Windows

- No additional permissions.
- `enigo` uses `SendInput`.

---

## Testing Strategy

### Unit Tests (`cargo test`)

- `domain::transcription::Transcription::new` rejects over-length text and out-of-range durations.
- `domain::settings::Settings::default` returns the expected defaults given a stubbed `app_data_dir`.
- `infrastructure::persistence::settings_repo::load_or_init` inserts defaults on an empty DB, is idempotent.
- `infrastructure::persistence::transcription_repo::prune_to` keeps exactly N newest rows.
- `infrastructure::audio::resample` converts a 48 kHz sine to 16 kHz with correct length (± tolerance).
- `AppError` round-trips through `serde` cleanly.

### Integration Tests (manual)

- End-to-end push-to-talk flow in dev (`cargo tauri dev`): press hotkey, speak, release, verify text appears in a Firefox URL bar.
- Settings persistence across restart.
- Model-missing → error banner path.
- Hotkey change via UI (Phase 3 live re-registration or restart).

### Smoke

- `cargo tauri build --debug` succeeds on Linux.
- App launches, window appears, no panics in log.
- Shutdown is clean (no dangling threads; verified via `tracing` logs).

---

## Rollout / Verification Gates

| Phase | Gate |
|-------|------|
| Prereq | `cargo tauri build --debug` succeeds; app window opens showing "loading…"; git repo initialised with first commit. |
| 0 | Default settings row exists after first launch; `get_settings` returns them; SQLite file present at expected path. |
| 1 | `WhisperEngine::load` succeeds with a real `ggml-base.en.bin`; a pre-recorded 5-sec WAV transcribes in < 10 sec with plausible text. |
| 2 | Recording 3 sec of microphone audio produces a `Vec<f32>` of length ≈ 48000 (3 × 16 kHz) with non-zero energy. |
| 3 | Pressing and releasing the hotkey in dev mode fires the four frontend events in order. |
| 4 | UI reflects recording/processing/idle transitions in real time; settings change persists across restart. |
| 5 | Transcribed text appears in a separate app (e.g. Firefox URL bar) after hotkey release. |

---

## Change Log

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2026-04-22 | jay | Initial draft (Ollama-based) |
| 2.0 | 2026-04-22 | jay | Removed Ollama, added whisper.cpp, simplified pipeline |
| 2.1 | 2026-04-22 | jay | Cross-cutting: AppError, AppState, constants, logging, shutdown, concurrency, lazy model load |
| **2.2** | **2026-04-22** | **jay** | **Gap report fixes: git init, push-to-talk resolved, real whisper-rs API, cpal resampling + worker thread for `!Send` Stream, enigo replaces per-OS shell-outs (Wayland-safe, Windows-correct), `dirs::data_dir()` unified, settings stored as single JSON row with first-run bootstrap, single-instance plugin, `spawn_blocking` for transcription, history pruned at write time with index, pinned dep versions, host-deps checklist, tauri.conf window config, commands include `get_status`, Phase 4 supports settings update UI.** |

---

## Related Documents

- [whisper.cpp](https://github.com/ggerganov/whisper.cpp)
- [whisper-rs](https://crates.io/crates/whisper-rs)
- [Tauri v2 docs](https://v2.tauri.app/)
- [cpal](https://github.com/rust-audio/cpal)
- [rubato](https://github.com/HEnquist/rubato)
- [enigo](https://github.com/enigo-rs/enigo)
