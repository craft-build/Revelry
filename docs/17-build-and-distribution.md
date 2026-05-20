# Build and Distribution

## Prerequisites

### macOS

- Xcode Command Line Tools: `xcode-select --install`
- Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- Bun: `curl -fsSL https://bun.sh/install | bash`

### Linux (Ubuntu/Debian)

- System dependencies:
  ```bash
  sudo apt install build-essential pkg-config libssl-dev \
    libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev \
    librsvg2-dev file
  ```
- Rust and Bun (same as macOS)

### Windows

- Visual Studio Build Tools 2022 (with C++ build tools workload)
- WebView2 (bundled by Tauri; pre-installed on Windows 11)
- Rust: `winget install Rustlang.Rustup`
- Bun: `powershell -c "irm bun.sh/install.ps1 | iex"`

## Building Locally

```bash
# Install frontend dependencies
bun install

# Development mode
bun run tauri:dev

# Production build (platform-native output)
bun run tauri:build
```

Build outputs:

- **macOS:** `src-tauri/target/release/bundle/dmg/Revelry_0.1.0_x64.dmg`
- **Linux:** `src-tauri/target/release/bundle/appimage/Revelry_0.1.0_amd64.AppImage`
- **Windows:** `src-tauri/target/release/bundle/nsis/Revelry_0.1.0_x64-setup.exe`

## CI/CD Pipeline

Builds for Linux (AppImage) and Windows (NSIS) run on GitLab CI when a `v*.*.*` tag is pushed. macOS DMG is built locally.

### Triggering a Release

```bash
git tag v0.1.0
git push origin v0.1.0
```

This triggers:

1. Linux AppImage build (Ubuntu Docker executor)
2. Windows NSIS build (Windows shared runner)
3. GitLab Release creation with download links

Upload the macOS DMG manually to the GitLab Release after building locally.

### Manual Build Trigger

Navigate to CI/CD → Pipelines in GitLab and click "Run pipeline" for manual builds.

## Distribution Notes

### Code Signing (Not Implemented in v1)

This release is unsigned. Users will see OS warnings on first launch:

- **macOS:** Right-click the app → Open → Open (bypasses Gatekeeper)
  Alternatively: System Preferences → Privacy & Security → Open Anyway
- **Windows:** Click "More info" → "Run anyway" to bypass SmartScreen
- **Linux:** `chmod +x Revelry_*.AppImage && ./Revelry_*.AppImage`

### Platform Notes

- **SQLite:** Statically compiled via rusqlite `bundled` feature — no external dependency
- **WebView:** Tauri uses the system WebView (WebKit on macOS/Linux, WebView2 on Windows)
- **Icons:** All icon formats (PNG, ICO, ICNS) bundled in `src-tauri/icons/`
