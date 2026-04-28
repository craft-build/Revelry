# Phase 1: Foundation - Research

**Phase:** 01-foundation
**Date:** 2025-04-28
**Status:** Complete

---

## Stack Research

### Verified Versions

| Technology | Version | Source | Notes |
|------------|---------|--------|-------|
| Tauri | 2.4.0 | crates.io | Production-stable since Oct 2024 |
| React | 18.3 | npm | Concurrent features, Suspense |
| TypeScript | 5.8+ | npm | Strict mode enabled |
| Rust | 1.80+ | rustup | Edition 2021 |
| SQLite | 3.45+ | rusqlite | Bundled via libsqlite3-sys |

### Essential Dependencies

| Category | Crate/Library | Version | Purpose |
|----------|---------------|---------|---------|
| Database | tauri-plugin-sql | 2.4.0 | SQLite bindings + migrations (TS) |
| Database | rusqlite | 0.32+ | Direct SQLite access (Rust) |
| Database | rusqlite_migration | 1.2+ | Versioned SQL migrations |
| Security | keyring | 3.6+ | OS keychain abstraction |
| Type Gen | ts-rs | 10.0+ | Rust→TypeScript type generation |
| Errors | snafu | 0.8+ | Structured error types with context |
| State | tauri-plugin-store | 2.4.0 | Persistent JSON settings |
| State | zustand | 5.0+ | Frontend UI state |
| HTTP | tauri-plugin-http | 2.4.0 | HTTP client (scoped) |
| UI | @radix-ui/react | 1.1+ | Accessible primitives |
| UI | tailwindcss | 4.0+ | Utility-first CSS |
| Icons | lucide-react | 0.475+ | Tree-shakeable icons |
| Build | vite | 6.0+ | Fast HMR for Tauri |

---

## Architecture Patterns

### Tauri 2.0 IPC Pattern

**Commands** (request/response, CRUD):
```rust
#[tauri::command]
async fn get_personas(state: tauri::State<'_, AppState>) -> Result<Vec<Persona>, AppError> {
    let db = state.db.lock().await;
    // ...
}
```

**Frontend invocation:**
```typescript
import { invoke } from '@tauri-apps/api/core';
const personas = await invoke<Persona[]>('get_personas');
```

**Channels** (streaming, low latency) — Phase 2 for AI tokens:
```rust
#[tauri::command]
async fn stream_chat(on_chunk: Channel<StreamEvent>) -> Result<(), AppError> {
    on_chunk.send(StreamEvent::Token("hello".into())).await?;
    Ok(())
}
```

### State Management

Three-layer architecture:
1. **Zustand** — Ephemeral UI state (active view, sidebar state, editor state)
2. **TanStack Query** — Server state (personas, providers, settings) — cache + invalidation
3. **SQLite** — Persistence (all data stored locally)

Pattern: Frontend calls `invoke()` → Rust command → SQLite → returns typed data → TanStack Query caches → Zustand reads from cache.

### Database Access (rusqlite)

```rust
use std::sync::Mutex;
use rusqlite::Connection;

pub struct DbState {
    pub conn: Mutex<Connection>,
}

// In main.rs:
tauri::Builder::default()
    .manage(DbState {
        conn: Mutex::new(Connection::open(&db_path)?),
    })
```

**CRITICAL:** SQLite connections are NOT `Send`. Wrap in `Mutex<Connection>`, never share across async boundaries. Use `tokio::task::spawn_blocking` for sync rusqlite operations inside async commands.

### Migrations (rusqlite_migration)

```rust
use rusqlite_migration::{Migrations, M};

let migrations = Migrations::new(vec![
    M::up(include_str!("../migrations/001_create_personas.sql")),
    M::down("DROP TABLE IF EXISTS personas;"),
]);

// On app startup:
migrations.to_latest(&mut conn)?;
```

SQL files in `src-tauri/migrations/`:
```sql
-- 001_create_personas.sql
CREATE TABLE IF NOT EXISTS personas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    appearance TEXT,
    personality TEXT,
    background TEXT,
    role_description TEXT,
    relationships TEXT,
    speech_style TEXT,
    notes TEXT,
    tags TEXT, -- JSON array
    avatar_filename TEXT,
    is_default BOOLEAN NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
```

### Type Sharing (ts-rs)

```rust
#[derive(ts_rs::TS, serde::Serialize)]
#[ts(export_to = "../src/types/generated/")]
pub struct Persona {
    pub id: String,
    pub name: String,
    pub appearance: Option<String>,
    // ... all fields
}
```

Build script auto-generates `.ts` files on `cargo build`. Frontend imports from `src/types/generated/`.

### Keychain Integration (keyring crate)

```rust
use keyring::Entry;

fn store_api_key(service: &str, key_name: &str, value: &str) -> Result<(), keyring::Error> {
    let entry = Entry::new(service, key_name)?;
    entry.set_password(value)?;
    Ok(())
}

fn get_api_key(service: &str, key_name: &str) -> Result<String, keyring::Error> {
    let entry = Entry::new(service, key_name)?;
    entry.get_password()
}

fn delete_api_key(service: &str, key_name: &str) -> Result<(), keyring::Error> {
    let entry = Entry::new(service, key_name)?;
    entry.delete_credential()
}
```

**Service naming:** `persona-dev` in dev mode, `persona` in production (per D-37).

### Error Handling (snafu)

```rust
use snafu::prelude::*;

#[derive(Debug, Snafu)]
pub enum AppError {
    #[snafu(display("Database error: {source}"))]
    Database { source: rusqlite::Error },

    #[snafu display("Validation error: {message}")]
    Validation { message: String },

    #[snafu display("Not found: {entity} with id {id}")]
    NotFound { entity: String, id: String },

    #[snafu display("Keychain error: {source}"))]
    Keychain { source: keyring::Error },

    #[snafu display("IO error: {source}"))]
    Io { source: std::io::Error },
}

// All Tauri commands return Result<T, AppError>
// AppError implements serde::Serialize for frontend consumption
```

### Tauri Capabilities (v2 Security Model)

```json
// src-tauri/capabilities/default.json
{
  "identifier": "default",
  "description": "Default capability for Persona main window",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "sql:default",
    "dialog:default",
    "store:default"
  ]
}
```

**Key points:**
- No blanket permissions — explicit grants only
- FS access scoped to AppData directory
- `dialog:open` for file picker (avatar upload) bypasses FS scope
- Keychain: Rust-only, no Tauri plugin needed (per D-31)

---

## UI Architecture (GNOME HIG)

### Adwaita Color Palette

```css
/* Light variant */
--adwaita-bg: #ffffff;
--adwaita-surface: #f6f5f4;
--adwaita-card: #ffffff;
--adwaita-text: #2e3436;
--adwaita-text-secondary: #77767b;
--adwaita-accent: #3584e4;
--adwaita-accent-hover: #2a76d6;
--adwaita-border: #c0bfbc;
--adwaita-headerbar: #303030;
--adwaita-sidebar: #f6f5f4;

/* Dark variant */
--adwaita-bg-dark: #242424;
--adwaita-surface-dark: #303030;
--adwaita-card-dark: #383838;
--adwaita-text-dark: #ffffff;
--adwaita-text-secondary-dark: #9a9996;
--adwaita-accent-dark: #3584e4;
--adwaita-border-dark: #505050;
--adwaita-headerbar-dark: #303030;
--adwaita-sidebar-dark: #1e1e1e;
```

### Layout Structure

```
┌─────────────────────────────────────────────────┐
│ Header Bar: [Logo] [Chat|Chars|Settings] [👤Name]│
├──────────┬──────────────────────────────────────┤
│ Sidebar  │                                       │
│ 240px    │    Content Area                       │
│          │    (Chat / Characters / Settings)     │
│ • Conv 1 │                                       │
│ • Conv 2 │                                       │
│          │                                       │
│  [≡]     │                                       │
├──────────┴──────────────────────────────────────┤
```

**Responsive breakpoints (D-17):**
- >=1024px: Full layout (sidebar + content)
- <1024px: Sidebar collapses to overlay

### Component Patterns

- **Header Bar:** Dark background, view switcher as tabs, persona avatar+name on right
- **Sidebar:** 240px expanded, 48px collapsed (icon-only), Ctrl+B toggle
- **Persona Editor:** Sliding panel from right, progressive disclosure sections
- **Avatar Picker:** GNOME Contacts pattern — circular click-to-upload area
- **Toast Notifications:** Non-modal, auto-dismiss (Adwaita pattern)

---

## Critical Pitfalls (Phase 1 Specific)

| Pitfall | Prevention | Verification |
|---------|------------|--------------|
| SQLite thread safety | `Mutex<Connection>`, never share across async | Code review: no `conn` across await points |
| Tauri 2.0 permissions | Explicit capability config, deny-by-default | Test: commands fail without capability grant |
| API key in plaintext | `keyring` crate only, never DB/config | Grep: no `api_key` column in schema |
| IPC payload limits | Return all personas (small set Phase 1), defer pagination | Count: <20 personas typical |
| Windows WebView storage | `useHttpsScheme: true` in tauri.conf.json | Config check present |
| Memory leaks | `unlisten()` in useEffect cleanup | Lint: useTauriEvent custom hook |
| Type drift | ts-rs build script auto-generates | CI: cargo build regenerates types |

---

## Don't Hand-Roll

| Concern | Use This | Not This |
|---------|----------|----------|
| DB migrations | `rusqlite_migration` crate | Manual CREATE TABLE checks |
| Type sharing | `ts-rs` derive macros | Manual TypeScript type definitions |
| API key storage | `keyring` crate | SQLite column, .env file, localStorage |
| Error types | `snafu` with context selectors | String errors, unwrap() |
| UI state | Zustand stores | React context + useReducer |
| Server state | TanStack Query | useEffect + useState manual fetch |

---

## Validation Architecture

### Dimension 1: Compilation
- `cargo build` passes for Rust backend
- `npm run build` (Vite) passes for frontend
- Generated TypeScript types compile without errors

### Dimension 2: Runtime Smoke
- `cargo tauri dev` launches application window
- Header bar, sidebar, content area render
- SQLite database initializes and migrations run

### Dimension 3: Contract Tests
- Each Tauri command has typed Result<Success, AppError>
- Frontend invoke calls return typed data or typed error
- Keyring set/get/delete roundtrip succeeds

### Dimension 4: Integration
- Persona CRUD cycle: create → read → update → delete
- Default persona: set → verify only one is_default
- Avatar: upload → file exists in app data → thumbnail generated

### Dimension 5: Boundary
- SQL injection: parameterized queries (rusqlite prepared statements)
- Input validation: name required, tags JSON format validated
- FS scope: avatar files only in AppData/persona/avatars/

### Dimension 6: Performance
- Persona list loads <100ms for 20 items
- UI renders 60fps during persona CRUD operations
- Memory baseline <50MB at idle (target <100MB)

### Dimension 7: Resilience
- Database migration failure: display error, don't crash
- Keychain unavailable: graceful error, not panic
- Missing avatar file: show placeholder, don't crash

### Dimension 8: Extensibility
- Migration system supports adding new tables in Phase 2
- Type generation auto-updates on Rust model changes
- Capabilities can be extended for new plugins

---

*Research complete: 2025-04-28*
