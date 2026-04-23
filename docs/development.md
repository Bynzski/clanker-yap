# Development Guide

How to set up your local development environment and contribute to Clanker Yap.

## Prerequisites

### System Requirements

| Tool | Version | Purpose |
|------|---------|---------|
| Rust | Stable | Backend compilation |
| Node.js | 16+ | Frontend and Tauri CLI |
| npm | 8+ | Package management |
| CMake | 3.10+ | C dependencies |
| Git | Any | Version control |

### Linux Dependencies

```sh
sudo pacman -S --needed \
  base-devel \
  cmake \
  webkit2gtk-4.1 \
  libappindicator-gtk3 \
  librsvg \
  nodejs \
  npm
```

### Verification

Check your toolchain:

```sh
rustc --version      # Should be 1.70+
node --version       # Should be 16+
npm --version        # Should be 8+
cmake --version      # Should be 3.10+
```

## Repository Setup

### Clone the Repository

```sh
git clone https://github.com/your-username/clanker-yap.git
cd clanker-yap
```

### Install Dependencies

```sh
npm install
```

This installs:
- Tauri CLI
- Build dependencies

### Verify Setup

```sh
# Rust compilation check
cargo check --manifest-path src-tauri/Cargo.toml

# Run tests
cargo test --manifest-path src-tauri/Cargo.toml
```

## Project Structure Overview

```
clanker-yap/
├── src/                    # Frontend (HTML/CSS/JS)
│   ├── index.html          # Main UI markup
│   ├── main.js             # Frontend logic
│   ├── style.css           # UI styles
│   └── vendor/             # Vendor files (tauri.js)
├── src-tauri/              # Rust backend
│   ├── src/
│   │   ├── domain/         # Core types
│   │   ├── application/    # Business logic
│   │   ├── infrastructure/ # External integrations
│   │   └── presentation/   # Tauri commands
│   ├── Cargo.toml
│   └── tauri.conf.json
├── docs/                   # Documentation
├── CONTRIBUTING.md
└── README.md
```

## Running Development Mode

### Start Development Server

```sh
npm run tauri dev
```

This command:
1. Compiles Rust backend
2. Starts Tauri dev server
3. Opens the application window
4. Enables hot reload for frontend

### Hot Reload

Changes to frontend files (`src/`) are reflected immediately.
Changes to Rust code trigger recompilation automatically.

### Debug Mode

For verbose logging, set the `RUST_LOG` environment variable:

```sh
RUST_LOG=trace npm run tauri dev
```

## Rust Development

### Package Structure

```
src-tauri/src/
├── domain/      # Pure types (no I/O)
├── application/ # Use cases
├── infrastructure/ # External integrations
└── presentation/  # Tauri commands
```

### Adding a New Feature

#### 1. Domain Layer

If adding business logic:

```rust
// src-tauri/src/domain/my_feature.rs

/// Documentation
#[derive(Debug, Clone)]
pub struct MyFeature {
    pub value: String,
}

impl MyFeature {
    /// Creates a new instance with validation
    pub fn new(value: String) -> Result<Self> {
        if value.is_empty() {
            return Err(AppError::SettingsInvalid(
                "Value cannot be empty".into()
            ));
        }
        Ok(Self { value })
    }
}
```

#### 2. Application Layer

If adding business operations:

```rust
// src-tauri/src/application/use_cases/my_feature.rs

use crate::domain::{AppError, Result};

pub struct MyFeatureUseCase;

impl MyFeatureUseCase {
    pub fn do_something(&self, input: String) -> Result<String> {
        // Business logic here
        Ok(input.to_uppercase())
    }
}
```

#### 3. Infrastructure Layer

If integrating with external services:

```rust
// src-tauri/src/infrastructure/my_service/mod.rs

pub mod service;
```

#### 4. Presentation Layer

If exposing commands to frontend:

```rust
// src-tauri/src/presentation/commands/my_feature_cmds.rs

use tauri::command;

#[command]
pub fn my_command(value: String) -> Result<String, String> {
    // Call use case
    Ok("response".into())
}
```

### Module Registration

Add new modules to `mod.rs` files:

```rust
// domain/mod.rs
pub mod my_feature;

// application/mod.rs
pub mod use_cases;

// presentation/commands/mod.rs
pub mod my_feature_cmds;
```

## Frontend Development

### HTML Structure

```html
<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <title>Clanker Yap</title>
    <link rel="stylesheet" href="style.css" />
    <script src="vendor/tauri.js"></script>
  </head>
  <body>
    <div class="app-shell">
      <!-- Your UI here -->
    </div>
    <script src="main.js"></script>
  </body>
</html>
```

### Tauri API Usage

```javascript
// Import invoke from Tauri
const { invoke } = window.__TAURI__.core;

// Call Rust command
async function getSettings() {
  try {
    const settings = await invoke('get_settings');
    console.log(settings);
  } catch (error) {
    console.error('Failed:', error);
  }
}

// Listen for events
window.__TAURI__.event.listen('recording-state', (event) => {
  console.log('State changed:', event.payload);
});
```

### UI Styling

Styles are in `src/style.css`. Uses CSS custom properties:

```css
:root {
  --color-primary: #0078d4;
  --color-bg: #1a1a1a;
  --font-sans: system-ui, sans-serif;
}

.app-shell {
  background: var(--color-bg);
  color: white;
  font-family: var(--font-sans);
}
```

## Testing

### Rust Unit Tests

```sh
# Run all tests
cargo test --manifest-path src-tauri/Cargo.toml

# Run specific test
cargo test --manifest-path src-tauri/Cargo.toml transcription

# Run with output
cargo test --manifest-path src-tauri/Cargo.toml -- --nocapture
```

### Frontend Testing

Manual testing for frontend:
1. `npm run tauri dev`
2. Interact with the app
3. Check console for errors

### Test Structure

```
src-tauri/src/
└── domain/
    └── transcription.rs  # Tests at bottom
        #[cfg(test)]
        mod tests {
            #[test]
            fn rejects_text_longer_than_max_length() { ... }
        }
```

## Code Quality

### Formatting

```sh
# Format Rust code
cargo fmt --manifest-path src-tauri/Cargo.toml

# Check formatting (CI)
cargo fmt --manifest-path src-tauri/Cargo.toml -- --check
```

### Linting

```sh
# Clippy lints
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
```

### Type Checking

```sh
# Type check
cargo check --manifest-path src-tauri/Cargo.toml
```

## Debugging

### Rust Debugging

#### Logging

Use `tracing` for structured logging:

```rust
use tracing::{info, warn, error};

fn my_function() {
    info!("Starting operation");
    warn!("Something unexpected");
    error!("Operation failed");
}
```

#### Environment Variables

```sh
# Enable all logging
RUST_LOG=trace npm run tauri dev

# Enable specific module
RUST_LOG=voice_transcribe=debug npm run tauri dev
```

#### GDB/LLDB

```sh
# Debug with GDB
rust-gdb --args cargo run --manifest-path src-tauri/Cargo.toml

# Debug with LLDB
rust-lldb --args cargo run --manifest-path src-tauri/Cargo.toml
```

### Frontend Debugging

#### Console

Open DevTools (right-click → Inspect) to see console logs.

#### Tauri Inspector

Enable Tauri devtools in `tauri.conf.json`:

```json
{
  "build": {
    "devtools": true
  }
}
```

### Network Debugging

#### Recording Issues

Check audio device permissions:

```sh
# Linux: Check ALSA
arecord -l

# Linux: Check PulseAudio
pactl list sources

# Check microphone in system settings
```

#### Paste Issues

Debug paste mode:

1. Open app in foreground
2. Hold hotkey, speak, release
3. Watch console for errors
4. Try different paste modes

## Building for Different Platforms

### Linux

```sh
# Build .deb package
npm run tauri build

# Output location
src-tauri/target/release/bundle/deb/
```

### macOS

Install required tools:
```sh
xcode-select --install
brew install cmake
```

Build:
```sh
npm run tauri build
```

### Windows

Install Visual Studio Build Tools, then:

```sh
npm run tauri build
```

## Contributing

See [CONTRIBUTING.md](../CONTRIBUTING.md) for:
- Code style guidelines
- Pull request process
- Reporting issues
- Feature requests

## Dependency Management

### Updating Rust Dependencies

```sh
# Update Cargo.toml version
# Then update lockfile
cargo update --manifest-path src-tauri/Cargo.toml
```

### Adding New Dependencies

1. Add to `src-tauri/Cargo.toml`:
   ```toml
   [dependencies]
   my-new-crate = "1.0"
   ```

2. Run `cargo check --manifest-path src-tauri/Cargo.toml`

### Frontend Dependencies

```sh
# Add npm package
npm install package-name

# Update package-lock.json
npm install
```

## Performance Profiling

### Rust Profiling

```sh
# Profile with criterion
cargo bench --manifest-path src-tauri/Cargo.toml

# Check binary size
cargo bloaty --release --manifest-path src-tauri/Cargo.toml
```

### Memory Usage

```sh
# Check memory with massif
valgrind --tool=massif ./target/release/voice-transcribe
```

## Common Development Tasks

### Add a New Setting

1. Update `domain/settings.rs`:
   ```rust
   pub struct Settings {
       // Existing fields...
       pub new_setting: String,
   }
   ```

2. Update persistence in `infrastructure/persistence/`

3. Add Tauri command in `presentation/commands/`

4. Add UI in frontend

### Add a New Command

1. Create command handler in `presentation/commands/`
2. Register in `lib.rs`
3. Call from frontend via `invoke()`

### Add Transcriptions

1. Update `domain/transcription.rs`
2. Update repository in `infrastructure/persistence/`
3. Add command for frontend access