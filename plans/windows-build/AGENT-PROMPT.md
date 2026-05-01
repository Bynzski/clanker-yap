# Agent Directive: Execute Windows Build Plan

## Core Rule

Windows artifacts are built on a Windows machine only.
Do not attempt Windows installer artifact generation on Linux.

## Workflow

1. Read `plans/windows-build/PROGRESS.md` and pick first 🔲 phase.
2. Read `plans/windows-build/PLAN.md` for that phase scope and done criteria.
3. Read `AGENTS.md` and follow project rules.
4. Implement only the current phase scope.
5. Validate.
6. Commit.
7. Update `PROGRESS.md`.

## Validation

### Linux local validation

```bash
cd src-tauri
cargo test
cargo clippy -- -D warnings
cargo fmt --check
```

### CI expectation

Windows pipeline checks run on `windows-latest` in GitHub Actions.
Phase 4/5 are not complete until at least one real workflow run is green.

## Commit Format

```text
<type>(windows-build): phase <N> — <short description>

- <change 1>
- <change 2>

Phase <N> of plans/windows-build/PLAN.md
```

## Completion Report Format

```text
## Phase <N> Complete

### Files Changed
- <path>: <summary>

### Validation
- local rust checks: PASS/FAIL
- windows CI run: PASS/FAIL (link)

### Commit
<hash>

### Ready for Next Phase
YES/NO
```

## Hard Stops

- If plan text conflicts with “Windows artifact on Windows host only,” stop and report.
- If workflow tries to build Windows artifacts on Linux, stop and report.
- If Rust checks fail, stop and report exact error.
