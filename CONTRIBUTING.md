# Contributing

## Scope

Clanker Yap is a small desktop utility. Contributions should stay focused on:

- local voice transcription
- push-to-talk workflow
- terminal-friendly paste behavior
- stability, packaging, and UX improvements

Large product-direction changes should be discussed in an issue before implementation.

## Development Setup

Install dependencies:

```sh
npm install
```

Run the app in development:

```sh
npm run tauri:dev
```

Install the repository Git hooks:

```sh
npm run hooks:install
```

Run verification before opening a pull request:

```sh
npm run verify:rust
npm run tauri:build
```

## Project Conventions

- Keep the frontend vanilla HTML, CSS, and JavaScript.
- Avoid introducing new dependencies unless they are clearly justified.
- Prefer small, reviewable pull requests over broad refactors.
- Preserve local-first behavior. Do not add cloud transcription or analytics.
- Do not commit local model files, databases, build artifacts, or `node_modules`.

## Commit Messages

Use Conventional Commits:

```text
<type>[optional scope]: <description>
```

Examples:
- `fix: handle short push-to-talk capture`
- `docs(release): update 0.1.0 release process`
- `feat(settings): add cumulative word count`

The repo includes Git hooks for:
- commit message validation
- pre-push Rust verification

## Pull Requests

Please include:

- what changed
- why it changed
- how it was tested
- screenshots for UI changes when applicable

If your change affects keyboard shortcuts, paste behavior, or model handling, mention the target platform you tested on.

## Issues

Bug reports are most useful when they include:

- OS and desktop environment
- whether the app was run via `tauri dev` or a built package
- expected behavior
- actual behavior
- reproduction steps

For terminal paste issues, include the terminal application name.

## Release Process

See:
- [RELEASING.md](./RELEASING.md)
- [CHANGELOG.md](./CHANGELOG.md)
- [docs/release-checklist.md](./docs/release-checklist.md)
