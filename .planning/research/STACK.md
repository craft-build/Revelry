# Technology Stack

**Project:** Persona - AI Roleplay Desktop Application  
**Researched:** April 27, 2025  
**Research Mode:** Ecosystem

## Recommended Stack

### Core Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Tauri | 2.4.0 | Desktop app framework | Native webview (WebKit/Chromium), 5-10MB binaries, secure IPC, plugin ecosystem. Stable release Oct 2024, proven in production. |
| Rust | 1.80+ | Backend language | Type safety, zero-cost abstractions, native performance, secure key storage via OS keychain APIs |
| React | 18.3 | Frontend framework | Concurrent features, automatic batching, Suspense for streaming UI, mature ecosystem |
| TypeScript | 5.8+ | Frontend language | Type-safe components, excellent DX, aligns with PROJECT.md constraints |

### Database

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tauri-plugin-sql | 2.4.0 | SQLite interface | Official Tauri plugin, sqlx-based, supports migrations, TypeScript bindings |
| SQLx (Rust) | 0.8 | SQL toolkit | Compile-time checked queries, async support, SQLite/MySQL/PostgreSQL |
| SQLite | 3.45+ | Local database | Single-file portable, FTS5 extension for search, excellent for desktop |

### HTTP & API

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tauri-plugin-http | 2.4.0 | HTTP client | Re-exports reqwest, Web API-compatible fetch, scoped permissions |
| reqwest | 0.12 (via plugin) | Rust HTTP client | Streaming support, async/await, SSE compatible |

### State Management

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| tauri-plugin-store | 2.4.0 | Persistent settings | Key-value store with JSON support, auto-save, Rust/JS interop |
| Zustand | 5.0+ | Frontend state | Minimal boilerplate, TypeScript-first, perfect for Tauri (no SSR complexity) |
| Rust State | tauri::State | Backend state | Managed via Tauri's Manager API, Mutex-wrapped for thread safety |

### UI Framework

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Radix UI | 1.1+ | Headless primitives | Accessibility-first, unstyled components, works with Tailwind |
| Tailwind CSS | 4.0+ | Styling | Utility-first, purges unused styles (keeps bundle small), GNOME palette compatible |
| Lucide React | 0.475+ | Icons | Lightweight, tree-shakeable, symbolic icon variants |

### Security

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| OS Keychain | Native | API key storage | Windows Credential Manager, macOS Keychain, Linux Secret Service |
| keyring | 3.6+ (Rust) | Keychain abstraction | Cross-platform keychain access, used in Rust backend |

### Build & Distribution

| Technology | Version | Purpose | Why |
|------------|---------|---------|-----|
| Vite | 6.0+ | Build tool | Fast HMR, optimized for Tauri, supports TAURI_DEV_HOST for mobile |
| @tauri-apps/cli | 2.4.0 | CLI tooling | Create, build, bundle, sign, and update Tauri apps |
| NSIS | 3.09+ | Windows installer | Standard .exe installer, customizable, auto-updater support |
| WiX | 4.0+ | Windows alternative | MSI installer for enterprise deployment |
| DMG | Built-in | macOS installer | Native disk image format, drag-drop install |
| AppImage | Built-in | Linux portable | Single-file executable, no root required |

## Alternatives Considered

| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Desktop Framework | Tauri 2.x | Electron | Electron bundles Chromium+Node (150MB+ vs 5-10MB), slower startup, larger attack surface |
| Frontend State | Zustand | Redux Toolkit | Redux adds boilerplate, Zustand is simpler for desktop apps without SSR needs |
| UI Library | Radix + Tailwind | Chakra UI | Chakra is heavier; Radix provides primitives with full Tailwind control |
| Database | tauri-plugin-sql | rusqlite directly | Plugin provides JS bindings and migration system, better Tauri integration |
| Key Storage | OS Keychain | tauri-plugin-stronghold | Stronghold is overkill for API keys; OS keychain is standard and simpler |
| Streaming HTTP | reqwest via plugin | native fetch | Tauri plugin provides scoped permissions and CSP compliance |

## Installation

### Prerequisites
```bash
# Rust (via rustup)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup target add x86_64-pc-windows-msvc  # Windows
echo 'rust-analyzer component add'       # Optional: IDE support

# Node.js 20+ (via fnm or nvm)
fnm install 20
fnm use 20
```

### Core Dependencies

**Rust (`src-tauri/Cargo.toml`):**
```toml
[package]
name = "persona"
version = "0.1.0"
edition = "2021"
rust-version = "1.80"

[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-sql = { version = "2", features = ["sqlite"] }
tauri-plugin-store = "2"
tauri-plugin-http = "2"
tauri-plugin-dialog = "2"
tauri-plugin-fs = "2"
serde = { version = "1", features = ["derive"] }
tokio = { version = "1", features = ["full"] }
keyring = "3"

[profile.release]
panic = "abort"
codegen-units = 1
lto = true
opt-level = 3
strip = true
```

**Frontend (`package.json`):**
```json
{
  "name": "persona",
  "version": "0.1.0",
  "type": "module",
  "dependencies": {
    "@tauri-apps/api": "^2.4.0",
    "@tauri-apps/plugin-sql": "^2.4.0",
    "@tauri-apps/plugin-store": "^2.4.0",
    "@tauri-apps/plugin-http": "^2.4.0",
    "@tauri-apps/plugin-dialog": "^2.4.0",
    "@tauri-apps/plugin-fs": "^2.4.0",
    "react": "^18.3.1",
    "react-dom": "^18.3.1",
    "zustand": "^5.0.0",
    "@radix-ui/react-dialog": "^1.1.0",
    "@radix-ui/react-dropdown-menu": "^2.1.0",
    "@radix-ui/react-scroll-area": "^1.1.0",
    "@radix-ui/react-slot": "^1.1.0",
    "@radix-ui/react-tabs": "^1.1.0",
    "lucide-react": "^0.475.0",
    "clsx": "^2.1.0",
    "tailwind-merge": "^2.6.0"
  },
  "devDependencies": {
    "@tauri-apps/cli": "^2.4.0",
    "@types/react": "^18.3.0",
    "@types/react-dom": "^18.3.0",
    "@vitejs/plugin-react": "^4.3.0",
    "autoprefixer": "^10.4.0",
    "postcss": "^8.5.0",
    "tailwindcss": "^4.0.0",
    "typescript": "^5.8.0",
    "vite": "^6.0.0"
  }
}
```

## Tauri 2.0 Specific Features & Migration Notes

### Key Changes from Tauri 1.x

1. **Plugin Architecture**: All non-core APIs moved to plugins (http, fs, dialog, etc.)
   - Import from `@tauri-apps/plugin-*` instead of `@tauri-apps/api`
   - Rust: `tauri-plugin-*` crates instead of built-in `tauri::api`

2. **Permission System**: New capabilities-based security model
   - Define permissions in `src-tauri/capabilities/default.json`
   - Core pseudo-plugins use `core:` prefix (e.g., `core:path:default`)
   - Use `core:default` for basic permissions

3. **Mobile Support**: Shared library architecture for iOS/Android
   - `src-tauri/src/lib.rs` instead of `main.rs` for mobile
   - Desktop entry point in `src-tauri/src/main.rs`

4. **HTTP Origin**: Windows uses `http://tauri.localhost` instead of `https://`
   - Set `useHttpsScheme: true` in config to keep HTTPS behavior
   - Affects IndexedDB/LocalStorage migration from v1

5. **State Management**: `tauri::State` is now `Arc`-wrapped automatically
   - No need for `Arc<Mutex<T>>` in managed state
   - Use `std::sync::Mutex` or `tokio::sync::Mutex` directly

### Vite Configuration for Tauri

```typescript
// vite.config.ts
import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

const host = process.env.TAURI_DEV_HOST

export default defineConfig({
  plugins: [react()],
  clearScreen: false,
  server: {
    port: 5173,
    strictPort: true,
    host: host || false,
    hmr: host ? { protocol: 'ws', host, port: 1430 } : undefined,
    watch: { ignored: ['**/src-tauri/**'] }
  },
  envPrefix: ['VITE_', 'TAURI_ENV_*'],
  build: {
    target: process.env.TAURI_ENV_PLATFORM === 'windows' 
      ? 'chrome105' 
      : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_ENV_DEBUG
  }
})
```

## Rust Backend Patterns

### SQLite with tauri-plugin-sql

```rust
// src-tauri/src/lib.rs
use tauri_plugin_sql::{Builder, Migration, MigrationKind};

fn main() {
    let migrations = vec![
        Migration {
            version: 1,
            description: "create_initial_tables",
            sql: include_str!("../migrations/001_initial.sql"),
            kind: MigrationKind::Up,
        }
    ];

    tauri::Builder::default()
        .plugin(
            tauri_plugin_sql::Builder::default()
                .add_migrations("sqlite:persona.db", migrations)
                .build()
        )
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### HTTP Client with reqwest

```rust
use tauri_plugin_http::reqwest;

#[tauri::command]
async fn stream_chat(messages: Vec<Message>) -> Result<StreamResponse, String> {
    let client = reqwest::Client::new();
    let res = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": "gpt-4",
            "messages": messages,
            "stream": true
        }))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    
    // Handle SSE stream
    Ok(StreamResponse::from_response(res).await)
}
```

### OS Keychain for API Keys

```rust
use keyring::Entry;

#[tauri::command]
fn store_api_key(provider: String, key: String) -> Result<(), String> {
    let entry = Entry::new("persona", &format!("api_key_{}", provider))
        .map_err(|e| e.to_string())?;
    entry.set_password(&key).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_api_key(provider: String) -> Result<Option<String>, String> {
    let entry = Entry::new("persona", &format!("api_key_{}", provider))
        .map_err(|e| e.to_string())?;
    match entry.get_password() {
        Ok(key) => Ok(Some(key)),
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(e.to_string())
    }
}
```

### State Management

```rust
use std::sync::Mutex;
use tauri::State;

#[derive(Default)]
struct AppState {
    current_conversation_id: Option<i64>,
    streaming: bool,
}

type ManagedState = Mutex<AppState>;

#[tauri::command]
fn start_conversation(state: State<ManagedState>) -> Result<(), String> {
    let mut state = state.lock().unwrap();
    state.current_conversation_id = Some(generate_id());
    Ok(())
}
```

## Frontend Patterns

### Zustand Store Structure

```typescript
// src/stores/chatStore.ts
import { create } from 'zustand'
import { invoke } from '@tauri-apps/api/core'

interface ChatStore {
  messages: Message[]
  streaming: boolean
  sendMessage: (content: string) => Promise<void>
  stopStreaming: () => void
}

export const useChatStore = create<ChatStore>((set, get) => ({
  messages: [],
  streaming: false,
  
  sendMessage: async (content) => {
    set({ streaming: true })
    
    // Stream handling via Tauri command
    await invoke('stream_chat', { 
      messages: [...get().messages, { role: 'user', content }]
    })
    
    set({ streaming: false })
  },
  
  stopStreaming: () => {
    invoke('abort_stream').catch(console.error)
    set({ streaming: false })
  }
}))
```

### SSE Streaming in React

```typescript
// src/hooks/useStreaming.ts
import { useEffect, useRef } from 'react'
import { listen } from '@tauri-apps/api/event'

export function useStreaming() {
  const abortRef = useRef<AbortController | null>(null)

  useEffect(() => {
    const unlisten = listen('chat:chunk', (event) => {
      // Append chunk to current message
      const chunk = event.payload as string
      // Update store...
    })

    return () => {
      unlisten.then(fn => fn())
    }
  }, [])

  const abort = () => {
    abortRef.current?.abort()
  }

  return { abort }
}
```

### Capabilities Configuration

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "description": "Default permissions for Persona",
  "local": true,
  "windows": ["main"],
  "permissions": [
    "core:default",
    "core:window:default",
    "core:app:default",
    "core:path:default",
    "sql:default",
    "sql:allow-execute",
    "store:default",
    "http:default",
    {
      "identifier": "http:default",
      "allow": [{ "url": "https://*.openai.com" }, { "url": "https://*.anthropic.com" }]
    },
    "dialog:default",
    "fs:default",
    {
      "identifier": "fs:allow-write-file",
      "allow": [{ "path": "$APPDATA/*" }, { "path": "$DOCUMENT/*" }]
    }
  ]
}
```

## Build & Distribution

### Platform Targets

| Platform | Bundle Format | Notes |
|----------|--------------|-------|
| Windows | .msi (WiX), .exe (NSIS) | Code signing required for SmartScreen |
| macOS | .dmg, .app | Apple Developer ID required, notarization for 10.15+ |
| Linux | .AppImage, .deb | AppImage for portable, .deb for Debian/Ubuntu |

### CI/CD (GitHub Actions)

```yaml
# .github/workflows/release.yml
name: Release
on:
  push:
    tags: ['v*']

jobs:
  build:
    strategy:
      matrix:
        platform: [ubuntu-latest, windows-latest, macos-latest]
    runs-on: ${{ matrix.platform }}
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: 20
      - uses: dtolnay/rust-action@stable
      - name: Install dependencies
        run: npm ci
      - name: Build Tauri
        run: npm run tauri build
      - name: Upload artifacts
        uses: actions/upload-artifact@v4
        with:
          name: persona-${{ matrix.platform }}
          path: src-tauri/target/release/bundle
```

## Version Confidence

| Component | Current | Stability | Notes |
|-----------|---------|-----------|-------|
| Tauri Core | 2.4.0 | HIGH | Stable since Oct 2024 |
| Tauri Plugins | 2.4.0 | HIGH | Tracked with core |
| React | 18.3.1 | HIGH | Stable, React 19 available but 18.x sufficient |
| TypeScript | 5.8.2 | HIGH | Latest stable |
| Zustand | 5.0.3 | HIGH | Stable, v5 is latest |
| Vite | 6.2.0 | HIGH | Latest major |

## Sources

### Official Documentation
- Tauri v2 Migration Guide: https://tauri.app/start/migrate/from-tauri-1/
- Tauri 2.0 Release Candidate: https://tauri.app/blog/tauri-2-0-0-release-candidate/
- Tauri Frontend Configuration: https://v2.tauri.app/start/frontend/
- Tauri SQL Plugin: https://v2.tauri.app/plugin/sql/
- Tauri Store Plugin: https://v2.tauri.app/plugin/store/
- Tauri HTTP Plugin: https://v2.tauri.app/plugin/http-client/
- Tauri State Management: https://v2.tauri.app/develop/state-management/

### Crate Registries
- crates.io tauri-plugin-sql: 2.4.0 (verified April 2025)
- crates.io tauri-plugin-store: 2.4.0 (verified April 2025)
- crates.io rusqlite: 0.39.0

### NPM Registries
- @tauri-apps/api: 2.4.0
- @tauri-apps/cli: 2.4.0
- Verified via npm registry API

---

**Confidence Level: HIGH**

All versions verified against official Tauri documentation and package registries as of April 2025. Tauri 2.x is stable and production-ready. All recommended libraries are actively maintained with stable major versions.
