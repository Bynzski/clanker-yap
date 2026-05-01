# Quickstart: Windows Build Plan

## Intent

Set up code + CI so Windows pipeline passes, then build Windows artifacts on a Windows machine.

## Important

- Do **not** build Windows installer artifacts on this Linux machine.
- Use GitHub Actions Windows runner for pipeline validation.
- Use a Windows host for final artifact build.

## Execute

1. Follow phases in `PLAN.md`.
2. Track status in `PROGRESS.md`.
3. Use `CHECKLIST.md` before declaring done.

## Local validation (Linux)

```bash
cd src-tauri
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

## CI validation (required)

- Workflow: `.github/workflows/windows-build.yml`
- Runner: `windows-latest`
- Must have at least one green run on PR/push to `main`

## Windows artifact build (on Windows machine)

From repo root:

```powershell
npm ci
npx tauri build
```

Expected output paths:
- `src-tauri/target/release/bundle/nsis/`
- `src-tauri/target/release/bundle/msi/`
