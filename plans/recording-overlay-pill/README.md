# Plans Index

Documentation for all plans in `plans/`.

## Plan Template

Templates live in `~/.pi/agent/skills/planning/templates/`:
- `TEMPLATE.md` — Plan document template
- `AGENT-PROMPT.md` — Agent execution directive
- `PROGRESS.md` — Phase tracking

## Active Plans

| Plan | Status | Version | Description |
|------|--------|---------|-------------|
| [recording-overlay-pill](./recording-overlay-pill/PLAN.md) | Draft | 3.0 | Floating always-on-top recording indicator pill with FFT-based EQ visualization |

## Archived Plans

| Plan | Status | Version | Notes |
|------|--------|---------|-------|
| — | — | — | — |

---

## Creating a New Plan

```bash
mkdir plans/my-feature
TEMPLATE_DIR="$HOME/.pi/agent/skills/planning/templates"
cp "$TEMPLATE_DIR"/*.md plans/my-feature/
mv plans/my-feature/TEMPLATE.md plans/my-feature/PLAN.md
```

## Executing a Plan

```
Execute the next phase of the recording-overlay-pill plan.
PLAN_PATH: plans/recording-overlay-pill/
```
