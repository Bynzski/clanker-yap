# Build & Release Guide

How to build, package, and release Clanker Yap.

## Build Overview

```
Source Code ──► Rust Compilation ──► Tauri Bundle ──► Package
                    │                      │
                    ▼                      ▼
               Debug/Release        .deb/.app/.exe
```

## Prerequisites

### All Platforms

- Rust stable toolchain
- Node.js and npm
- Tauri CLI (installed via `npm install`)

### Platform-Specific

**Linux:**
```sh
sudo pacman -S --needed base-devel cmake webkit2gtk-4.1 \
  libappindicator-gtk3 librsvg
```

**macOS:**
```sh
xcode-select --install
```

**Windows:**
- Visual Studio Build Tools
- WebView2 Runtime

## Build Commands

### Debug Build

Quick build for development testing:

```sh
npm run tauri build -- --debug
```

**Output:** `src-tauri/target/debug/bundle/`

### Release Build

Production-ready build:

```sh
npm run tauri build
```

**Output:** `src-tauri/target/release/bundle/`

## Build Output

### Linux (.deb)

```
src-tauri/target/release/bundle/deb/
└── Clanker Yap_0.1.0_amd64.deb
```

Install with:
```sh
sudo dpkg -i "Clanker Yap_0.1.0_amd64.deb"
```

### macOS (.app)

```
src-tauri/target/release/bundle/macos/
└── Clanker Yap.app
```

Install by copying to Applications:
```sh
cp -R "Clanker Yap.app" /Applications/
```

### Windows (.exe/.msi)

```
src-tauri/target/release/bundle/
├── msi/Clanker Yap_0.1.0_x64_en-US.msi
└── exe/Clanker Yap_0.1.0_x64-setup.exe
```

## Build Configuration

### Bundle Targets

Configure in `src-tauri/tauri.conf.json`:

```json
{
  "bundle": {
    "targets": "all"
  }
}
```

**Options:**
- `"all"` - Build for current platform
- `"deb"` - Linux only
- `["deb", "app", "msi"]` - Specific targets

### App Identifier

Unique identifier for the app:

```json
{
  "identifier": "dev.jay.voice-transcribe"
}
```

**Format:** `domain.reversed.app-name`

### Window Configuration

```json
{
  "app": {
    "windows": [{
      "label": "main",
      "title": "Clanker Yap",
      "width": 480,
      "height": 196,
      "resizable": false,
      "decorations": false,
      "center": true
    }]
  }
}
```

### App Metadata

```json
{
  "productName": "Clanker Yap",
  "version": "0.1.0",
  "copyright": "Copyright 2024",
  "category": "Productivity"
}
```

## Release Workflow

### 1. Update Version

Update `version` in:
- `src-tauri/tauri.conf.json`
- `src-tauri/Cargo.toml`
- Optionally `package.json`

### 2. Test Everything

```sh
# Run all tests
cargo test --manifest-path src-tauri/Cargo.toml

# Build release
npm run tauri build

# Test installed app
./Clanker\ Yap
```

### 3. Build Release

```sh
npm run tauri build
```

### 4. Verify Artifacts

Check all output files:
- Sizes are reasonable
- Binaries are executable
- Package installs correctly

### 5. Create Release

For GitHub releases:

1. Create git tag: `git tag v0.1.0`
2. Push tag: `git push origin v0.1.0`
3. Create GitHub release with artifacts

## Continuous Integration

### GitHub Actions Example

```yaml
name: Build and Release

on:
  push:
    tags:
      - 'v*'

jobs:
  build:
    strategy:
      matrix:
        include:
          - platform: 'ubuntu-22.04'
            artifact: '.deb'
          - platform: 'macos-13'
            artifact: '.app'
          - platform: 'windows-2022'
            artifact: '.exe'

    runs-on: ${{ matrix.platform }}

    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        
      - name: Install dependencies
        run: |
          npm install
          # Platform-specific deps
      
      - name: Build
        run: npm run tauri build
        
      - name: Upload artifact
        uses: actions/upload-artifact@v4
        with:
          name: clanker-yap-${{ matrix.platform }}
          path: src-tauri/target/release/bundle/**
```

## Code Signing

### macOS

1. Obtain Apple Developer certificate
2. Sign app bundle:

```sh
codesign --force --deep --sign "Developer ID: Name" "Clanker Yap.app"
```

3. Notarize:

```sh
xcrun notarytool submit "Clanker Yap.app" --apple-api-key ...
```

### Windows

1. Obtain code signing certificate
2. Sign executable:

```sh
signtool sign /f certificate.pfx /p password app.exe
```

## Uninstall

### Linux (.deb)

```sh
sudo dpkg -r clanker-yap
```

### macOS

Remove from Applications:
```sh
rm -rf /Applications/Clanker\ Yap.app
```

Also remove supporting files:
```sh
rm -rf ~/Library/Application\ Support/dev.jay.voice-transcribe
rm -rf ~/Library/Preferences/dev.jay.voice-transcribe.plist
```

### Windows

Use "Add or Remove Programs" or:

```sh
winget uninstall clanker-yap
```

## Troubleshooting Build Issues

### Missing WebKit

**Error:** `error: failed to run custom build command for 'webkit2gtk'`

**Solution:** Install webkit development libraries:
```sh
# Debian/Ubuntu
sudo apt install libwebkit2gtk-4.1-dev

# Fedora
sudo dnf install webkit2gtk4.1-devel
```

### Missing CMake

**Error:** `Could not find CMake`

**Solution:**
```sh
# Install CMake
sudo pacman -S cmake  # Linux
brew install cmake     # macOS
```

### Linker Errors

**Error:** `linker gcc not found`

**Solution:** Install build tools:
```sh
# Linux
sudo pacman -S base-devel

# macOS
xcode-select --install
```

### Target Triple Mismatch

**Error:** `package target mismatch`

**Solution:** Clean and rebuild:
```sh
cargo clean --manifest-path src-tauri/Cargo.toml
npm run tauri build
```

## Artifact Sizes

Typical release build sizes:

| Component | Size |
|-----------|------|
| Rust binary | ~15-20 MB |
| WebView (Linux) | Bundled |
| Total .deb | ~50-60 MB |
| Total .app | ~60-80 MB |

## Performance Optimization

### Release Profile

Configured in `src-tauri/Cargo.toml`:

```toml
[profile.release]
lto = true
codegen-units = 1
panic = "abort"
strip = true
```

### Build Time Optimization

```toml
[profile.dev]
opt-level = 0  # Faster debug builds
debug = false
```

## Validation Script

Create `validate-build.sh`:

```bash
#!/bin/bash
set -e

echo "Running tests..."
cargo test --manifest-path src-tauri/Cargo.toml

echo "Building release..."
npm run tauri build

echo "Checking artifacts..."
ls -la src-tauri/target/release/bundle/deb/

echo "Build complete!"
```

Run with:
```sh
chmod +x validate-build.sh
./validate-build.sh
```