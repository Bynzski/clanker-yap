# Releasing Clanker Yap

This document defines the release process, versioning rules, changelog rules, and Git hook workflow for Clanker Yap.

## Current release target

As of `0.1.0`, Clanker Yap ships as:

- **Linux x86_64 AppImage**

Release confidence for `0.1.0`:
- **Wayland:** smoke tested
- **X11:** smoke tested
- **macOS:** not yet a supported release target
- **Windows:** not yet a supported release target

## Versioning rules

Clanker Yap uses **SemVer-style versioning**:

`MAJOR.MINOR.PATCH`

### Patch release (`0.1.x`)
Use a patch release for:
- bug fixes
- packaging fixes
- docs-only corrections
- small UX improvements with no workflow breakage
- dependency bumps with no user-facing behavior change

Examples:
- `0.1.1`
- `0.1.2`

### Minor release (`0.x.0` → `0.y.0`)
Use a minor release for:
- new user-facing features
- meaningful workflow improvements
- new configuration options
- new release-platform support
- backward-compatible desktop behavior changes

Examples:
- `0.2.0`
- `0.3.0`

### Major release (`1.0.0`, `2.0.0`)
Use a major release for:
- breaking changes to user workflows
- incompatible settings or storage changes
- default behavior changes that require migration guidance
- significant platform/support policy changes

Examples:
- `1.0.0`
- `2.0.0`

## What counts as a breaking change here?

For Clanker Yap, a change is considered breaking if it:
- removes or changes expected push-to-talk behavior
- breaks compatibility with existing persisted settings/history
- changes required model setup in a non-compatible way
- removes a supported platform or release artifact
- significantly changes paste behavior without a migration note

## Changelog rules

We keep a human-written changelog in [`CHANGELOG.md`](./CHANGELOG.md).

Format:
- Keep entries grouped under release headings
- Use these sections where relevant:
  - `Added`
  - `Changed`
  - `Fixed`
  - `Removed`
  - `Notes`
- Write for users, not just developers
- Mention packaging/platform changes explicitly
- Mention important known limitations explicitly

### Changelog workflow

1. Add noteworthy changes to `Unreleased`
2. Before release, convert `Unreleased` into the version heading
3. Add the release date
4. Start a fresh `Unreleased` section after tagging/publishing

## Release checklist

Use [`docs/release-checklist.md`](./docs/release-checklist.md) for the concrete release checklist.

## Required verification

Before every release:

```sh
npm run verify:rust
npm run tauri:build
```

This means:
- tests pass
- clippy passes with zero warnings
- formatting is clean
- the Linux AppImage bundles successfully

## Release procedure

### 1. Update versions

Update version numbers in:
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- `package.json`

### 2. Update docs and changelog

- Update `CHANGELOG.md`
- Confirm `README.md` reflects the current release target
- Confirm `docs/build.md` is accurate
- Confirm `docs/release-checklist.md` still matches reality

### 3. Run verification

```sh
npm run verify:rust
npm run tauri:build
```

### 4. Smoke test the artifact

At minimum:
- launch the AppImage
- confirm hotkey registration
- confirm record → transcribe → paste flow
- confirm editor paste works
- confirm terminal paste works
- confirm overlay behavior
- confirm settings/history/word count persistence

For now, note whether testing was done on:
- Wayland
- X11

### 5. Compute checksum

```sh
sha256sum src-tauri/target/release/bundle/appimage/*.AppImage
```

### 6. Commit release prep

Use a conventional commit, for example:

```text
chore(release): prepare v0.1.0
```

### 7. Tag the release

```sh
git tag v0.1.0
git push origin main --tags
```

### 8. Publish GitHub release

Include:
- AppImage artifact
- SHA256 checksum
- summary of notable features/fixes
- Linux AppImage-only note for `0.1.0`
- Wayland/X11 smoke-test note

## Git hooks

This repo includes Git hooks in [`.githooks/`](./.githooks):

- `commit-msg` — enforces Conventional Commit format
- `pre-push` — runs Rust verification before push

Install them with:

```sh
npm run hooks:install
```

This sets:

```sh
git config core.hooksPath .githooks
```

## Commit rules

Commit messages should follow Conventional Commits:

```text
<type>[optional scope]: <description>
```

Examples:
- `fix: treat short recordings as empty captures`
- `docs(release): add AppImage release workflow`
- `feat(settings): add cumulative word count`

Recommended types:
- `feat`
- `fix`
- `docs`
- `refactor`
- `test`
- `build`
- `ci`
- `chore`

## Release note template

Suggested structure for GitHub releases:

```md
## Clanker Yap vX.Y.Z

### Highlights
- ...

### Included in this release
- ...

### Artifact
- Linux x86_64 AppImage

### Validation
- Wayland smoke tested
- X11 smoke tested

### Known limitations
- macOS not yet supported
- Windows not yet supported
```
