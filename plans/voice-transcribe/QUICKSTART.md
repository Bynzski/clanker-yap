# Quickstart: Creating a New Plan

This guide walks you through creating a new plan folder in `plans/` using the template.

## Step 1: Determine Plan Scope

Before creating the plan, answer these questions:

1. **What work needs to happen?** (Feature, refactor, migration, etc.)
2. **What's the estimated number of phases?** (1-2 = simple, 3+ = complex)
3. **Does it build on existing work?** (Schema changes, new entities, etc.)

## Step 2: Create the Folder

```bash
# Navigate to plans directory
cd plans

# Create folder with descriptive name
# Use kebab-case: feature-name, not FeatureName
mkdir my-new-feature

# For versioned plans (if updating an existing one):
mkdir my-feature-v2
```

## Step 3: Copy Template Files

```bash
# Copy from plan-template
cp plan-template/README.md my-new-feature/
cp plan-template/TEMPLATE.md my-new-feature/PLAN.md
cp plan-template/AGENT-PROMPT.md my-new-feature/
cp plan-template/PROGRESS.md my-new-feature/
```

## Step 4: Fill in the Templates

### 4.1 Create PLAN.md

Open `PLAN.md` and fill in:

```markdown
# My New Feature Plan

**Author:** Your Name
**Date:** 2026-04-18
**Status:** Draft
**Version:** 1.0
```

Then complete these sections:

| Section | What to Write |
|---------|--------------|
| Purpose | Why does this work exist? What problem does it solve? |
| Scope | What's in scope (table format) and out of scope |
| What Already Exists | Any existing code you're building on |
| Implementation Plan | Phases with dependencies |
| Phase Details | Specific tasks per phase |
| Testing Strategy | What tests are needed |
| Checklist | Actionable items per phase |

### 4.2 Customize AGENT-PROMPT.md

Replace placeholders:

| Find | Replace With |
|------|-------------|
| `[PLAN NAME]` | Your plan name |
| `<plan-name>` | Your folder name |
| `Phase Prereq` | Your actual first phase |
| `Phase 0`, `Phase 1` | Your actual phases |
| Context file paths | Real paths from your codebase |

### 4.3 Initialize PROGRESS.md

| Find | Replace With |
|------|-------------|
| `[Plan Name]` | Your plan name |
| `[Phase Name]` | Next phase to work on |
| Phase rows | Your actual phases |
| `[Prerequisite description]` | Your actual descriptions |

### 4.4 Update README.md

Replace the template header and adjust the tables:

```markdown
# My New Feature Plans

Documentation for [what this plan does].

## Active Plans

| Document | Status | Version | Description |
|----------|--------|---------|-------------|
| [PLAN.md](./PLAN.md) | Draft | 1.0 | [One-line description] |
```

## Step 5: Review and Approve

Before marking as ready for execution:

- [ ] All sections filled in
- [ ] Phase order makes sense (no circular dependencies)
- [ ] Each phase scope is clear and actionable
- [ ] File structure matches what you'll actually create
- [ ] Test strategy is defined

Update status from `Draft` to `Approved` when ready.

## Step 6: Execute

Now you can start executing phases:

1. Read the phase details in PLAN.md
2. Copy the phase section from AGENT-PROMPT.md
3. Open a fresh agent session
4. Execute and commit
5. Update PROGRESS.md with the commit hash

## Example: Creating a Simple Two-Phase Plan

```bash
# Create folder
mkdir plans/ui-refresh
cd plans/ui-refresh

# Copy templates
cp ../plan-template/* .

# Fill in PLAN.md with:
# - Purpose: "Redesign the library view to show album art"
# - Phase 0: "Add album art display to existing track cards"
# - Phase 1: "Add grid view option with masonry layout"
# - Scope: In = UI changes, Out = Backend logic
# - Dependencies: None

# Update PROGRESS.md:
# - Phase 0: 🔲
# - Phase 1: 🔲

# Update AGENT-PROMPT.md:
# - Add frontend file paths
# - Add component patterns to follow

# Mark as Approved
# Execute Phase 0
# Update progress
# Execute Phase 1
# Mark complete
```

## Troubleshooting

### "I don't know how many phases to have"

Start with coarse phases (Prereq, Phase 0, Phase 1). Split later if needed.

### "The scope is too large"

Split into multiple plans:
- `plans/my-feature-backend/` for Rust work
- `plans/my-feature-frontend/` for UI work

### "A phase depends on another plan"

Add a note in the Dependencies section and wait for that plan to complete.
