# Windows Build Plan — Index

## Overview

This plan prepares Clanker Yap for reliable Windows builds by:
- aligning config and code for cross-platform compatibility,
- enforcing CI validation on a Windows runner,
- and keeping actual Windows artifact generation on a Windows machine.

## Key Rule

**Windows installers are not built on this Linux machine.**

## Documents

- `PLAN.md` — full implementation plan (v1.2)
- `AGENT-PROMPT.md` — execution instructions
- `PROGRESS.md` — phase tracking
- `CHECKLIST.md` — quality and completion checks
- `QUICKSTART.md` — fast execution path

## Phase Summary

| Phase | Description |
|---|---|
| 0 | Add Windows bundle targets in `tauri.conf.json` |
| 1 | Linux-gate overlay initialization in `main.rs` |
| 2 | Clarify Linux-only dependency comments in `Cargo.toml` |
| 3 | Update `AGENTS.md` for Windows-host artifact build flow |
| 4 | Add Windows GitHub Actions workflow (`windows-latest`, PR/push to `main`) |
| 5 | Final readiness verification (local checks + at least one green Windows CI run) |

## Completion Requirement

Plan execution is not complete until:
- local Linux Rust checks are green, and
- at least one real run of `.github/workflows/windows-build.yml` is green.
