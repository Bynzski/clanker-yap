# Releasing Clanker Yap

Clanker Yap ships one release artifact: a Linux x86_64 AppImage built on Linux.

## Versioning

The project uses SemVer-style `MAJOR.MINOR.PATCH` versions:

- Patch: bug fixes and small compatible improvements.
- Minor: new features, meaningful workflow changes, or packaging-policy changes.
- Major: incompatible settings, storage, or user-workflow changes.

Keep noteworthy changes in [`CHANGELOG.md`](./CHANGELOG.md) under `Unreleased`. Before a release, move them into a dated version section and start a fresh `Unreleased` section.

## Release procedure

1. Update the version in `package.json`, `src-tauri/Cargo.toml`, and `src-tauri/tauri.conf.json`.
2. Update `CHANGELOG.md` and release-facing documentation.
3. Run the complete release command:

   ```sh
   npm ci
   npm run release:verify
   ```

   `release:verify` runs all Rust checks and builds only the AppImage. The build script includes `NO_STRIP=1`, required on Arch/CachyOS.

4. Smoke-test:

   ```sh
   chmod +x "src-tauri/target/release/bundle/appimage/Clanker Yap_X.Y.Z_amd64.AppImage"
   ./src-tauri/target/release/bundle/appimage/Clanker\ Yap_X.Y.Z_amd64.AppImage
   ```

   Verify launch, hotkey registration, record → transcribe → paste, editor and terminal paste, overlay behavior, settings, history, and word-count persistence.

5. Generate the checksum:

   ```sh
   cd src-tauri/target/release/bundle/appimage
   sha256sum "Clanker Yap_X.Y.Z_amd64.AppImage" > SHA256SUMS.txt
   ```

6. Commit, tag, and push:

   ```sh
   git commit -m "chore(release): prepare vX.Y.Z"
   git tag vX.Y.Z
   git push origin main --tags
   ```

7. If publishing a GitHub release, upload the AppImage and `SHA256SUMS.txt`, then include highlights, smoke-test coverage, and known limitations.

Use [`docs/release-checklist.md`](./docs/release-checklist.md) for the concrete checklist.

## Git hooks

Install the repository hooks with:

```sh
npm run hooks:install
```

The `commit-msg` hook enforces Conventional Commits and `pre-push` runs the Rust verification gate.
