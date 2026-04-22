# AGENTS.md — Repo Rules for AI Agents

This file is the source of truth for how agents work in `clanker-yap`. Read it first on every fresh session. Phase-specific instructions live in `plans/voice-transcribe/AGENT-PROMPT.md`; this file covers the rules that apply **regardless of phase**.

---

## What this repo is

A single desktop application: **Voice Transcription** — a Tauri v2 app with a Rust backend and a vanilla HTML/JS/CSS frontend. Hold a global hotkey, speak, release; whisper.cpp transcribes locally and the text is pasted into the focused input field.

The entire implementation is **plan-driven**. The plan lives in `plans/voice-transcribe/`. Do not invent work that isn't in the plan.

---

## Start-of-session protocol

Before making any change:

1. Read `plans/voice-transcribe/PROGRESS.md` to find the next unstarted phase.
2. Read `plans/voice-transcribe/PLAN.md` in full (it is the approved spec, v2.2+).
3. Read the matching phase section in `plans/voice-transcribe/AGENT-PROMPT.md`.
4. If any of those three documents contradict each other, STOP and report — do not pick a winner silently.

---

## Plan-driven workflow

- **One phase per commit.** Do not merge phases. Do not split a phase across commits unless the phase explicitly says so.
- **Do exactly what the phase scope describes.** No bonus features, no opportunistic refactors, no "while I'm here" cleanup.
- **Out-of-scope is a hard boundary.** If you think something out-of-scope is required, STOP and report.
- After finishing a phase, update `plans/voice-transcribe/PROGRESS.md`:
  - Set the completed phase to ✅ and fill in the commit hash.
  - Leave the next phase as 🔲 (the next agent moves it to 🔧 on start).

---

## Architecture layering

```
src-tauri/src/
├── domain/           # Pure types + constants + AppError. No I/O, no deps on other layers.
├── application/      # Use cases + AppState + orchestrator. Depends on domain.
├── infrastructure/   # whisper, audio (cpal+rubato), persistence (SQLite), paste (enigo). Depends on domain.
└── presentation/     # Tauri commands + DTOs. Depends on application.
```

Rules:

- **Domain has no dependencies on other layers.** No `rusqlite`, no `cpal`, no `tauri` types in `domain/`.
- **Presentation does not talk to infrastructure directly.** It calls use cases in `application/`.
- **Infrastructure does not import from presentation.**
- Every new module must be registered in its parent `mod.rs`.

---

## Error handling

- One error type: `crate::domain::error::AppError`. All fallible functions return `Result<T, AppError>` (`Result` alias in that module).
- Do not introduce ad-hoc `Box<dyn Error>`, `anyhow::Error`, or per-module error enums.
- `AppError` implements `serde::Serialize`, so Tauri commands can return it directly.
- Add a new `AppError` variant only if the PLAN calls for one, or if a new failure mode is genuinely unclassifiable — log the reasoning in the commit message.

---

## Concurrency rules

- **Mutexes:** `parking_lot::Mutex`, never `std::sync::Mutex`.
- **CPU-bound work (transcription):** wrap in `tauri::async_runtime::spawn_blocking`. Never block Tokio threads or the Tauri main loop.
- **`cpal::Stream` is `!Send`.** Do not store it in `AppState`, do not send it across threads. It lives on a dedicated `std::thread` owned by `RecorderHandle`; orchestrator talks to it via `crossbeam_channel`.
- **`whisper_rs::WhisperContext` is `Send + Sync`**, but `WhisperState` is not. Share the context via `Arc<WhisperEngine>`; create a fresh `WhisperState` per transcription inside the blocking task.
- **Hotkey callback must not block.** It only flips state and sends a message.

---

## Logging

- Use `tracing::{info, warn, error, debug, trace}`. Never `println!`, `eprintln!`, `dbg!`.
- Initialise via `voice_transcribe::init_logging()` exactly once in `main.rs`.
- Structured fields preferred: `tracing::info!(samples = n, elapsed_ms = x, "transcribed")` over string interpolation.

---

## Dependencies

- **All Cargo dependencies are added in Phase Prereq.** Every later phase must leave `Cargo.toml` untouched.
- If you think a new dep is required mid-phase, STOP and report. Do not add one speculatively.
- Versions are pinned (see `plans/voice-transcribe/PLAN.md` "Dependency Summary"). Do not bump them as part of unrelated work.
- Frontend has no bundler and no `npm install` step beyond the Tauri scaffold. `@tauri-apps/api` is vendored as a static file under `src/vendor/`.

---

## Platform rules

- Platform-specific code uses `#[cfg(target_os = "...")]`. No runtime string matching on `std::env::consts::OS` unless the plan explicitly calls for it (currently: nowhere).
- **Never shell out for paste.** `enigo` handles keystroke simulation across macOS / Windows / Linux X11 / Linux Wayland. Historical plans that mentioned `osascript`, `xdotool`, or `cmd /c "echo v | clip"` were wrong and are superseded.
- Paths: always go through `infrastructure::persistence::paths::app_data_dir()`, which wraps `dirs::data_dir()`. Never hard-code `~/.voice-transcribe/` or platform-specific paths.

---

## Testing & verification gates

Before every commit:

```sh
cargo check --manifest-path src-tauri/Cargo.toml
cargo test  --manifest-path src-tauri/Cargo.toml
```

Both must pass. If `cargo test` fails because of a pre-existing issue unrelated to your phase, STOP and report — do not fix or mask it silently.

Where the phase explicitly requires it, also run `cargo tauri build --debug`.

Do not invoke `--no-verify`, `--no-gpg-sign`, or any hook-bypass flag.

---

## Commit rules

- One commit per phase.
- Stage only the files you modified or created — `git add <paths>`, never `git add -A` / `git add .`.
- Do **not** push. The user pushes when they're ready.
- Do not amend prior commits. If a pre-commit hook fails, fix the issue and create a **new** commit.
- Do not modify `.git/config`, don't change the author, don't set up signing.

Message format:

```
<type>(<scope>): phase <N> — <short description>

- <bullet of what was done>
- <...>

Phase <N> of plans/voice-transcribe/PLAN.md (v2.2)
```

- `<type>`: `feat`, `fix`, `refactor`, `docs`, `chore`.
- `<scope>`: usually `app` or a module name (`audio`, `whisper`, `paste`, `ui`, `persistence`).

---

## Frontend rules (phase 4)

- Vanilla HTML/CSS/JS only. No bundler, no framework, no npm runtime deps beyond what `create-tauri-app` produced.
- Vendor `@tauri-apps/api` under `src/vendor/tauri.min.js` and reference it locally. No CDN at runtime.
- Dark theme, system font stack, 480×560 window.
- No external CSS or webfonts.

---

## What NOT to do

- Do not rewrite historical commits.
- Do not add CI, linting, or formatting configs unless the plan says so.
- Do not create planning / analysis / summary documents — this file and the `plans/` folder are the only long-lived docs.
- Do not introduce backward-compatibility shims, feature flags, or "in case we need it later" abstractions.
- Do not add comments that explain *what* well-named code already says. Comments are only for non-obvious *why* — hidden constraints, workarounds, invariants.
- Do not write multi-paragraph docstrings.
- Do not ship mock/fake data paths in production code.
- Do not change the plan (`plans/voice-transcribe/PLAN.md`) during phase execution. If the plan is wrong, STOP and report; the user bumps the version.

---

## Escalation — hard stops

Stop immediately and report to the user if any of these occur. Do not work around them.

- `cargo check` fails and the fix is outside the phase scope.
- `cargo test` fails on a test you didn't touch, in a way that predates your changes.
- You need a `Cargo.toml` dependency that isn't in the PLAN's Dependency Summary.
- You need to modify a file outside the current phase's declared Scope / Files list.
- The plan describes an API, module, or file that doesn't exist in the repo.
- Host tooling is missing during Prereq (do not `sudo`, do not auto-install — just report).
- `whisper-rs` or `enigo` builds fail with linker errors that a careful read of the crate's README can't resolve.
- A file, branch, or config you didn't create is already present and you don't understand what it is.

Reporting a blocker is always better than papering over it.

---

## File map — where things live

| Path | Role |
|------|------|
| `plans/voice-transcribe/PLAN.md` | The spec. Source of truth for scope. |
| `plans/voice-transcribe/AGENT-PROMPT.md` | Per-phase execution prompt. |
| `plans/voice-transcribe/PROGRESS.md` | Phase status. Update after each commit. |
| `plans/voice-transcribe/README.md` | User-facing overview + host-deps setup. |
| `plans/voice-transcribe/CHECKLIST.md` | Plan review record (v2.2+). |
| `src-tauri/src/domain/` | Pure types (`AppError`, constants, entities). |
| `src-tauri/src/application/` | Use cases, `AppState`, orchestrator. |
| `src-tauri/src/infrastructure/` | External-world integrations (SQLite, cpal, whisper, enigo). |
| `src-tauri/src/presentation/` | Tauri commands + DTOs. |
| `src/` | Frontend (vanilla HTML/CSS/JS). |
| `AGENTS.md` | This file. |
