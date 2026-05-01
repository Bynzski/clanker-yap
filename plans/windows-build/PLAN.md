# Windows Build Plan

**Author:** Agent
**Date:** 2026-04-30
**Status:** Approved
**Version:** 1.2

---

## Purpose

Prepare the codebase and CI so Windows builds are reliable, while keeping Windows artifact generation on a Windows machine only.

---

## Operating Model (Source of Truth)

- Linux machine: development + prep + local Rust validation
- GitHub Actions: Windows pipeline validation must pass
- Windows machine: build release artifact (NSIS/MSI)

**Windows installers are not built on this Linux machine.**

---

## Scope

### In Scope

| Item | Priority | Notes |
|---|---|---|
| Add Windows bundle targets (`nsis`, `msi`) | P0 | Required for Windows host builds |
| Make overlay creation Linux-only | P0 | Prevent Linux-only GTK behavior from affecting Windows |
| Keep Linux-only deps clearly documented | P1 | `wayland-overlay` / GTK notes in Cargo.toml |
| Add Windows build instructions to AGENTS.md | P0 | Explicit Windows-host artifact process |
| Add GitHub Actions Windows workflow | P0 | Pipeline validation on `windows-latest` |

### Out of Scope

- Building Windows artifact on Linux
- Shipping/signing changes
- Native Windows overlay implementation
- Frontend feature work

---

## Implementation Plan

| Phase | Description |
|---|---|
| 0 | Update `tauri.conf.json` bundle targets (`appimage`, `deb`, `nsis`, `msi`) |
| 1 | Guard overlay creation in `main.rs` for Linux only |
| 2 | Clarify Linux-only feature/deps comments in `Cargo.toml` |
| 3 | Update `AGENTS.md` with Windows-host build workflow |
| 4 | Add `.github/workflows/windows-build.yml` on `windows-latest` |
| 5 | Final readiness verification (real CI run + local checks) |

---

## Phase Details

### Phase 0 — Bundle targets
- Modify: `src-tauri/tauri.conf.json`
- Ensure targets include: `appimage`, `deb`, `nsis`, `msi`

### Phase 1 — Linux-only overlay bootstrap
- Modify: `src-tauri/src/main.rs`
- Wrap overlay import/call with `#[cfg(target_os = "linux")]` as needed
- No Windows overlay implementation in this plan

### Phase 2 — Cargo docs clarity
- Modify: `src-tauri/Cargo.toml`
- Keep `wayland-overlay` and GTK comments explicitly Linux-only
- No Windows-specific dependency section unless truly needed

### Phase 3 — Team instructions
- Modify: `AGENTS.md`
- Add explicit section:
  - CI validates Windows on GitHub Actions Windows runner
  - Windows artifacts are built on Windows machine
  - Linux machine does not build Windows installer artifacts
  - Artifact output paths for Windows (`.../bundle/nsis`, `.../bundle/msi`)

### Phase 4 — GitHub Actions pipeline
- New: `.github/workflows/windows-build.yml`
- Runner: `windows-latest`
- Trigger:
  - `pull_request` to `main`
  - `push` to `main`
- Required steps:
  - `actions/checkout@v4`
  - Rust toolchain setup
  - run in `src-tauri`
  - `cargo test`
  - `cargo clippy -- -D warnings`
  - `cargo fmt --check`
  - optional: `cargo check`
- Optional separate job: `npx tauri build` on Windows for extra confidence

**Done criteria (Phase 4):**
- Workflow file committed
- Workflow triggers correctly on PR/push
- At least one run is green

### Phase 5 — Final readiness verification
- Confirm local Linux validation passes:
  - `cd src-tauri`
  - `cargo test`
  - `cargo clippy -- -D warnings`
  - `cargo fmt --check`
- Confirm a real GitHub Actions Windows run passed
- Record CI run link/reference in `plans/windows-build/PROGRESS.md`

---

## Testing Strategy

### Local (Linux)
- `cargo test`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`

### CI (Windows)
- Run Rust validation checks on `windows-latest`
- Require green status before release build

### Release Build (Windows host)
- Build Windows installer only on Windows host from tagged commit
- Smoke test installer before publishing

---

## Checklist

- [ ] `tauri.conf.json` includes `nsis` and `msi`
- [ ] `main.rs` overlay init is Linux-gated
- [ ] `Cargo.toml` Linux-only dependency comments are clear
- [ ] `AGENTS.md` explicitly states Windows-host-only artifact build
- [ ] Windows GitHub Action exists and passes on a real run
- [ ] Local Linux rust validation remains green

---

## Releasing (Windows Artifact)

On a Windows machine, from repo root:

```powershell
npm ci
npx tauri build
```

Expected outputs:
- `src-tauri/target/release/bundle/nsis/`
- `src-tauri/target/release/bundle/msi/`

Smoke-test installer before publishing.

---

## Change Log

| Version | Date | Author | Changes |
|---|---|---|---|
| 1.2 | 2026-04-30 | Agent | Added concrete CI definition, done criteria, and final verification requirements |
| 1.1 | 2026-04-30 | Agent | Aligned plan to Windows-host artifact workflow; removed Linux cross-target requirement |
| 1.0 | 2026-04-30 | Agent | Initial draft |
