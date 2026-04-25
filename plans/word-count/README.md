# Plans Index

Documentation for all plans in `plans/`.

## Plan Template

Templates live in `~/.pi/agent/skills/planning/templates/`:
- `TEMPLATE.md` — Plan document template
- `AGENT-PROMPT.md` — Agent execution directive
- `PROGRESS.md` — Phase tracking
- `QUICKSTART.md` — Creation guide
- `CHECKLIST.md` — Quality review checklist

## Active Plans

| Plan | Status | Version | Description |
|------|--------|---------|-------------|
| [PLAN.md](./PLAN.md) | Draft | 1.0 | [One-line description] |

## Archived Plans

| Plan | Status | Version | Notes |
|------|--------|---------|-------|
| — | — | — | — |

---

## Creating a New Plan

```bash
# Create folder
mkdir plans/my-feature

# Copy templates
TEMPLATE_DIR="$HOME/.pi/agent/skills/planning/templates"
cp "$TEMPLATE_DIR"/*.md plans/my-feature/
mv plans/my-feature/TEMPLATE.md plans/my-feature/PLAN.md
```

Then fill in `PLAN.md`, customize `PROGRESS.md`, and update this README.

## Executing a Plan

### User-Prompted Session

```
Execute the next phase of the my-feature plan.
PLAN_PATH: plans/my-feature/
```

### Programmatic Loop

Automated runner invokes the agent repeatedly until all phases are ✅.

---

## Plan Documents

### [Plan Name]

**Status:** Draft → Approved → In Progress → Completed

**Purpose:** [What this plan accomplishes]

**Phases:**
| Phase | Description | Status |
|-------|-------------|--------|
| 0 | [Phase 0 description] | 🔲 |
| 1 | [Phase 1 description] | 🔲 |

**Key Files:**
- `PLAN.md` — Full plan document
- `AGENT-PROMPT.md` — Agent execution directive
- `PROGRESS.md` — Phase tracking
