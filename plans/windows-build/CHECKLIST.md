# Windows Build Plan Checklist

## Alignment
- [ ] Plan explicitly states: Windows artifacts are built on Windows machine only
- [ ] No requirement to build Windows artifact on Linux

## Phase 0
- [ ] `src-tauri/tauri.conf.json` includes `nsis`
- [ ] `src-tauri/tauri.conf.json` includes `msi`
- [ ] Linux targets remain present

## Phase 1
- [ ] `main.rs` overlay initialization is Linux-only
- [ ] No unintended behavior changes to Linux overlay

## Phase 2
- [ ] `Cargo.toml` comments clearly mark `wayland-overlay` as Linux-only
- [ ] No unnecessary Windows dependency placeholders added

## Phase 3
- [ ] `AGENTS.md` has clear Windows-host build instructions
- [ ] `AGENTS.md` states CI is for validation, not Linux artifact generation
- [ ] `AGENTS.md` includes Windows artifact output paths

## Phase 4
- [ ] `.github/workflows/windows-build.yml` exists
- [ ] Uses `windows-latest`
- [ ] Triggers on `pull_request` to `main`
- [ ] Triggers on `push` to `main`
- [ ] Runs in `src-tauri`
- [ ] Runs `cargo test`
- [ ] Runs `cargo clippy -- -D warnings`
- [ ] Runs `cargo fmt --check`
- [ ] Workflow had at least one green run

## Phase 5
- [ ] Local Linux validation passes (`cargo test`, `clippy`, `fmt --check`)
- [ ] Windows CI pipeline passes
- [ ] CI run reference recorded in `PROGRESS.md`

## Done
- [ ] Plan is implementation-ready and consistent across docs
