# Domain Pitfalls: Tauri 2.0 + React + Rust Desktop Applications

**Domain:** Native Desktop AI Roleplay Application (Persona)
**Tech Stack:** Tauri 2.0, React 18, TypeScript, Rust, SQLite (rusqlite)
**Researched:** 2025-04-27
**Confidence:** HIGH (based on official Tauri docs, rusqlite docs, and GitHub issue analysis)

---

## Critical Pitfalls

Mistakes that cause rewrites, data loss, security vulnerabilities, or application crashes.

### Pitfall 1: SQLite Connection Thread Safety Violations

**What goes wrong:** SQLite connections (`rusqlite::Connection`) are not `Send` or `Sync` by default. Sharing a connection across async boundaries or threads causes runtime panics or data corruption.

**Why it happens:**
- rusqlite's `Connection` type does not implement `Send` - it cannot be safely transferred between threads
- Attempting to store a `Connection` in Tauri's managed state and access it from multiple commands causes compilation errors or runtime crashes
- Using `CachedStatement` across thread boundaries fails because it's explicitly `!Send`

**Consequences:**
- Compilation failures with complex error messages about trait bounds
- Runtime panics when commands execute concurrently
- Silent data corruption if using unsafe workarounds

**Prevention:**
```rust
// WRONG: Connection cannot be shared
pub struct DbState {
    conn: rusqlite::Connection, // This won't work across commands
}

// CORRECT: Use a connection pool or wrap in Mutex with thread-local
use std::sync::Mutex;
use rusqlite::Connection;

pub struct DbState {
    conn: Mutex<Connection>, // Each command locks for exclusive access
}

// OR use r2d2_sqlite for connection pooling:
// [dependencies]
// r2d2 = "0.8"
// r2d2_sqlite = "0.25"
// rusqlite = { version = "0.31", features = ["bundled"] }
```

**Detection:**
- Compilation error: "`Connection` cannot be sent between threads safely"
- Runtime panic in logs about borrowing or thread safety

**Phase to address:** Phase 1 (Core Infrastructure - Database Layer)

---

### Pitfall 2: IPC Command Payload Size Limits

**What goes wrong:** Tauri uses JSON-RPC for IPC serialization. Large payloads (conversations with thousands of messages, large character cards) cause severe performance degradation or failures.

**Why it happens:**
- All Tauri command arguments and returns must be JSON-serializable
- Large binary data (images, blobs) gets base64-encoded, ballooning size
- No built-in streaming for large data transfers

**Consequences:**
- UI freezing during large data transfers
- Out-of-memory errors in webview
- Slow startup when loading conversation history

**Prevention:**
```rust
// WRONG: Returning large dataset directly
#[tauri::command]
fn get_all_messages() -> Vec<Message> {
    // This serializes ALL messages to JSON - expensive!
    db.get_all_messages()
}

// CORRECT: Use tauri::ipc::Response for binary data
use tauri::ipc::Response;

#[tauri::command]
fn export_conversation(conversation_id: &str) -> Response {
    let data = db.export_to_bytes(conversation_id);
    Response::new(data) // Bypasses JSON serialization
}

// CORRECT: Paginate large datasets
#[tauri::command]
fn get_messages_paginated(
    conversation_id: &str,
    page: usize,
    page_size: usize
) -> Vec<Message> {
    db.get_messages_paginated(conversation_id, page, page_size)
}

// CORRECT: Use streaming for large text
#[tauri::command]
async fn stream_messages(
    window: tauri::Window,
    conversation_id: String
) -> Result<(), String> {
    let (mut tx, mut rx) = tokio::sync::mpsc::channel(100);
    
    // Spawn task to stream messages
    tauri::async_runtime::spawn(async move {
        while let Some(chunk) = rx.recv().await {
            window.emit("message-chunk", chunk).unwrap();
        }
    });
    
    db.stream_messages(&conversation_id, tx).await;
    Ok(())
}
```

**Detection:**
- Chrome DevTools shows massive JSON payloads in Network tab
- UI becomes unresponsive when loading conversations
- Memory usage spikes during data operations

**Phase to address:** Phase 1 (Core Chat) - Design pagination from day one

---

### Pitfall 3: Tauri 2.0 Permissions/Capabilities Misconfiguration

**What goes wrong:** Tauri 2.0 replaced the v1 allowlist with a new capabilities system. Misconfiguration causes commands to fail silently or throw confusing errors.

**Why it happens:**
- v1's `tauri.conf.json > tauri > allowlist` is completely removed in v2
- New capabilities system uses separate files in `src-tauri/capabilities/`
- Each plugin command must be explicitly permitted
- Default behavior is deny-all for security

**Consequences:**
- Commands fail with "command not allowed" errors
- File system operations silently fail
- Confusion because migration tool doesn't catch all cases

**Prevention:**
```json
// src-tauri/capabilities/default.json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "description": "Default capabilities for Persona",
  "windows": ["main"],
  "permissions": [
    // Core functionality
    "core:default",
    
    // File system - scoped to app data
    {
      "identifier": "fs:allow-read",
      "allow": [{ "path": "$APPDATA/*" }]
    },
    {
      "identifier": "fs:allow-write",
      "allow": [{ "path": "$APPDATA/*" }]
    },
    
    // Dialog plugin
    "dialog:default",
    
    // Shell - only allow opening URLs
    {
      "identifier": "shell:allow-open",
      "allow": [{ "args": ["https://*", "http://*"], "cmd": "" }]
    },
    
    // Custom commands - must be explicitly listed
    "custom-commands:allow-get-conversations",
    "custom-commands:allow-send-message"
  ]
}
```

**Detection:**
- Error in console: "command [name] not allowed"
- Command returns `null` or undefined instead of expected data
- Check Tauri devtools for capability violations

**Phase to address:** Phase 1 (Project Setup) - Configure capabilities early

---

### Pitfall 4: Memory Leaks from Event Listeners

**What goes wrong:** Tauri event listeners (`listen`, `listen_any`) accumulate if not properly cleaned up, causing memory leaks in long-running applications.

**Why it happens:**
- React components register listeners in `useEffect` but don't always clean up
- Tauri events are global and persist beyond component lifecycle
- Each `listen()` call returns an `unlisten` function that must be called

**Consequences:**
- Memory usage grows over application lifetime
- Event handlers execute multiple times (duplicate listeners)
- Application slows down after extended use

**Prevention:**
```typescript
// WRONG: Missing cleanup
useEffect(() => {
  listen('message-received', (event) => {
    setMessages(prev => [...prev, event.payload]);
  });
}, []); // Leaks! Unlisten never called

// CORRECT: Clean up listeners
useEffect(() => {
  let unlisten: (() => void) | undefined;
  
  listen('message-received', (event) => {
    setMessages(prev => [...prev, event.payload]);
  }).then(fn => { unlisten = fn });
  
  return () => {
    unlisten?.(); // Critical cleanup
  };
}, []);

// CORRECT: Use custom hook
function useTauriEvent<T>(event: string, handler: (payload: T) => void) {
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    
    listen<T>(event, (e) => handler(e.payload)).then(fn => {
      unlisten = fn;
    });
    
    return () => unlisten?.();
  }, [event, handler]);
}

// Usage
useTauriEvent<Message>('message-received', (msg) => {
  setMessages(prev => [...prev, msg]);
});
```

**Detection:**
- Chrome DevTools Memory tab shows growing listener counts
- Events fire multiple times for single occurrence
- Memory profile shows retained closures accumulating

**Phase to address:** Phase 1 (Core Chat) - Implement proper listener patterns

---

### Pitfall 5: Async Command Handling in Rust

**What goes wrong:** Tauri commands can be async, but improper use of blocking operations or async runtime causes performance issues or deadlocks.

**Why it happens:**
- rusqlite operations are synchronous (blocking)
- Running blocking operations in async commands without `spawn_blocking` freezes the runtime
- Tauri uses tokio for async runtime

**Consequences:**
- UI becomes unresponsive during database operations
- Concurrent commands block each other
- Potential for deadlock with nested async calls

**Prevention:**
```rust
// WRONG: Blocking in async command
#[tauri::command]
async fn get_conversations(state: State<'_, DbState>) -> Result<Vec<Conversation>, String> {
    // This blocks the async runtime!
    let conn = state.conn.lock().unwrap();
    let conversations = conn.prepare("SELECT ...").unwrap()
        .query_map([], |row| { /* ... */ })
        .unwrap()
        .collect::<Result<Vec<_>, _>>()
        .unwrap();
    Ok(conversations)
}

// CORRECT: Use spawn_blocking for sync operations
use tokio::task;

#[tauri::command]
async fn get_conversations(state: State<'_, DbState>) -> Result<Vec<Conversation>, String> {
    let conn = state.conn.clone();
    
    task::spawn_blocking(move || {
        let conn = conn.lock().unwrap();
        conn.prepare("SELECT ...")
            .map_err(|e| e.to_string())?
            .query_map([], |row| { /* ... */ })
            .map_err(|e| e.to_string())?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| e.to_string())?
}

// CORRECT: Use a dedicated DB thread/pool
#[tauri::command]
async fn get_conversations(state: State<'_, DbPool>) -> Result<Vec<Conversation>, String> {
    let pool = state.pool.clone();
    
    pool.get()
        .map_err(|e| e.to_string())?
        .interact(|conn| {
            // Database operations here
        })
        .await
        .map_err(|e| e.to_string())
}
```

**Detection:**
- UI freezes during database queries
- DevTools shows long-running JavaScript promises
- Async commands don't return

**Phase to address:** Phase 1 (Core Infrastructure - Database Layer)

---

### Pitfall 6: API Key Storage Security

**What goes wrong:** Storing API keys in plaintext (config files, localStorage, SQLite) exposes user credentials to malware or other applications.

**Why it happens:**
- localStorage is accessible to any JavaScript on the page
- SQLite database is a file that can be read by other applications
- Config files are often world-readable

**Consequences:**
- API keys stolen by malware
- Keys visible in backups
- Violation of "data sovereignty" principle

**Prevention:**
```rust
// Use the keyring crate for OS keychain integration
// [dependencies]
// keyring = { version = "3", features = ["apple-native", "windows-native", "linux-native"] }

use keyring::Entry;

pub struct SecureStorage;

impl SecureStorage {
    const SERVICE: &'static str = "com.persona.app";
    
    pub fn store_api_key(provider: &str, key: &str) -> Result<(), Box<dyn std::error::Error>> {
        let entry = Entry::new(Self::SERVICE, &format!("api_key_{}", provider))?;
        entry.set_password(key)?;
        Ok(())
    }
    
    pub fn get_api_key(provider: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
        let entry = Entry::new(Self::SERVICE, &format!("api_key_{}", provider))?;
        match entry.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }
    
    pub fn delete_api_key(provider: &str) -> Result<(), Box<dyn std::error::Error>> {
        let entry = Entry::new(Self::SERVICE, &format!("api_key_{}", provider))?;
        entry.delete_password()?;
        Ok(())
    }
}

// Tauri command
#[tauri::command]
async fn save_api_key(
    provider: String,
    key: String
) -> Result<(), String> {
    SecureStorage::store_api_key(&provider, &key)
        .map_err(|e| e.to_string())
}
```

**Detection:**
- Code review: search for API key storage in config files or database
- Check for localStorage usage for sensitive data

**Phase to address:** Phase 1 (Core Chat - API Provider Support)

---

### Pitfall 7: WebView IndexedDB/LocalStorage Data Loss (Windows)

**What goes wrong:** Tauri 2.0 changed the default origin URL on Windows from `https://tauri.localhost` to `http://tauri.localhost`. This resets IndexedDB, LocalStorage, and Cookies.

**Why it happens:**
- Browsers treat HTTP and HTTPS origins as completely separate
- Tauri v1 used `https://tauri.localhost` on Windows
- Tauri v2 uses `http://tauri.localhost` by default

**Consequences:**
- User data in LocalStorage/IndexedDB appears to "disappear" after app update
- Settings reset to defaults
- Cache invalidation issues

**Prevention:**
```json
// tauri.conf.json - Preserve v1 behavior
{
  "app": {
    "windows": [
      {
        "useHttpsScheme": true  // Use https://tauri.localhost
      }
    ]
  }
}
```

Or in Rust:
```rust
use tauri::WebviewWindowBuilder;

WebviewWindowBuilder::new(app, "main", WebviewUrl::App("index.html".into()))
    .use_https_scheme(true)  // Preserve storage across updates
    .build()?;
```

**Detection:**
- User reports settings lost after app update
- LocalStorage returns `null` for previously stored keys

**Phase to address:** Phase 0 (Project Setup - Configuration)

---

### Pitfall 8: Build and Distribution Failures

**What goes wrong:** Code signing, notarization, and platform-specific build requirements cause CI/CD failures or user-facing security warnings.

**Why it happens:**
- macOS requires code signing + notarization for distribution outside App Store
- Windows SmartScreen warns about unsigned executables
- Cross-compilation requires specific toolchains
- Certificate management is complex

**Consequences:**
- macOS app shows "cannot be opened" error
- Windows shows SmartScreen warning
- CI/CD pipelines fail on signing steps

**Prevention:**

**macOS:**
```bash
# 1. Create signing certificate in Apple Developer portal
# 2. Download and install to keychain
# 3. Configure Tauri
```

```json
// tauri.conf.json
{
  "bundle": {
    "macOS": {
      "signingIdentity": "Developer ID Application: Your Name (TEAM_ID)",
      "notarization": {
        "provider": "apple-id",  // or "app-store-connect-api"
        "appleId": "your@email.com",
        "ascProvider": "TEAM_ID"
      }
    }
  }
}
```

**Windows:**
```json
// tauri.conf.json
{
  "bundle": {
    "windows": {
      "certificateThumbprint": "A1B2C3D4...",
      "digestAlgorithm": "sha256",
      "timestampUrl": "http://timestamp.digicert.com"
    }
  }
}
```

**Environment variables for CI/CD:**
```bash
# macOS
export APPLE_SIGNING_IDENTITY="..."
export APPLE_ID="..."
export APPLE_PASSWORD="..."  # App-specific password
export APPLE_TEAM_ID="..."

# Windows (from certificate)
export WINDOWS_CERTIFICATE="base64_encoded_pfx"
export WINDOWS_CERTIFICATE_PASSWORD="..."
```

**Phase to address:** Phase 4 (Build & Distribution) - but configure early

---

### Pitfall 9: React Component Re-rendering in WebView

**What goes wrong:** React components in Tauri's webview re-render excessively, causing performance issues due to incorrect dependency arrays or context usage.

**Why it happens:**
- Tauri invoke calls create new function references on each render
- Context providers re-render unnecessarily
- Large lists (conversations, messages) render without virtualization

**Consequences:**
- Janky UI performance
- High CPU usage in webview process
- Battery drain on laptops

**Prevention:**
```typescript
// WRONG: New function reference on every render
function ChatComponent() {
  const [messages, setMessages] = useState([]);
  
  const sendMessage = async (content: string) => {
    // This function is recreated on every render
    await invoke('send_message', { content });
  };
  
  return <ChatInput onSend={sendMessage} />; // Causes ChatInput to re-render
}

// CORRECT: Memoize callbacks
function ChatComponent() {
  const [messages, setMessages] = useState([]);
  
  const sendMessage = useCallback(async (content: string) => {
    await invoke('send_message', { content });
  }, []); // Stable reference
  
  return <ChatInput onSend={sendMessage} />;
}

// CORRECT: Virtualize long lists
import { FixedSizeList as List } from 'react-window';

function MessageList({ messages }: { messages: Message[] }) {
  return (
    <List
      height={600}
      itemCount={messages.length}
      itemSize={80}
      itemData={messages}
    >
      {({ index, style }) => (
        <MessageItem 
          message={messages[index]} 
          style={style}
        />
      )}
    </List>
  );
}

// CORRECT: Optimize Tauri invocations
function useConversations() {
  const [conversations, setConversations] = useState<Conversation[]>([]);
  
  useEffect(() => {
    // Fetch once on mount
    invoke<Conversation[]>('get_conversations')
      .then(setConversations)
      .catch(console.error);
  }, []); // Empty deps = run once
  
  // Use events for updates instead of polling
  useTauriEvent<Conversation>('conversation-updated', (conv) => {
    setConversations(prev => 
      prev.map(c => c.id === conv.id ? conv : c)
    );
  });
  
  return conversations;
}
```

**Detection:**
- React DevTools Profiler shows excessive renders
- Chrome DevTools Performance tab shows long frames
- User reports of sluggish UI

**Phase to address:** Phase 1 (Core Chat - UI implementation)

---

### Pitfall 10: Database Migration Failures

**What goes wrong:** Schema migrations fail silently or corrupt data when not handled transactionally.

**Why it happens:**
- SQLite doesn't support full ALTER TABLE operations
- Migration scripts error partway through
- No versioning of schema state

**Consequences:**
- Application crashes on startup
- Data loss during migration
- Inconsistent database state

**Prevention:**
```rust
// Use rusqlite_migration crate
// [dependencies]
// rusqlite_migration = { version = "1", features = ["rusqlite-bundled"] }

use rusqlite_migration::{Migrations, M};

pub fn setup_migrations(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    let migrations = Migrations::new(vec![
        M::up("CREATE TABLE characters (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            description TEXT,
            created_at INTEGER NOT NULL
        )"),
        M::up("CREATE TABLE conversations (
            id TEXT PRIMARY KEY,
            character_id TEXT NOT NULL,
            title TEXT,
            created_at INTEGER NOT NULL,
            FOREIGN KEY (character_id) REFERENCES characters(id)
        )"),
        M::up("CREATE INDEX idx_conversations_character ON conversations(character_id)"),
        // Always add migrations, never modify existing ones
    ]);
    
    migrations.to_latest(conn)?;
    Ok(())
}

// Critical: Backup before migration
pub fn migrate_with_backup(conn: &mut Connection) -> Result<(), Box<dyn std::error::Error>> {
    // Create backup before any migration
    let backup_path = format!("{}.backup.{}", 
        db_path, 
        chrono::Utc::now().timestamp()
    );
    conn.backup(rusqlite::DatabaseName::Main, &backup_path, None)?;
    
    // Now run migrations
    setup_migrations(conn)?;
    
    Ok(())
}
```

**Detection:**
- Application crashes on startup with SQL errors
- Schema mismatch errors in logs
- Missing columns or tables

**Phase to address:** Phase 1 (Core Infrastructure - Database Layer)

---

## Moderate Pitfalls

### Pitfall 11: Window/Menu Event Handling Changes in v2

**What goes wrong:** Tauri 2.0 changed the Window API (`Window` → `WebviewWindow`) and Menu API. Code using v1 patterns fails.

**Prevention:**
```rust
// v1 pattern - NO LONGER WORKS
use tauri::Window;
let window = app.get_window("main").unwrap();

// v2 pattern
use tauri::WebviewWindow;
let window = app.get_webview_window("main").unwrap();

// Menu changed completely - see migration guide
// SystemTray → TrayIcon
// MenuBuilder pattern introduced
```

**Phase to address:** Phase 1 (GNOME UI Shell)

---

### Pitfall 12: HTTP Client Plugin Required

**What goes wrong:** HTTP functionality moved to a plugin in v2. Using the old API causes compilation errors.

**Prevention:**
```toml
# Cargo.toml
tauri-plugin-http = "2"
```

```rust
// Enable plugin
.plugin(tauri_plugin_http::init())
```

```typescript
// JavaScript - now a plugin
import { fetch } from '@tauri-apps/plugin-http';
const response = await fetch('https://api.example.com');
```

**Phase to address:** Phase 1 (Core Chat - API calls)

---

### Pitfall 13: State Management Anti-Patterns

**What goes wrong:** Improper use of Tauri's state management causes deadlocks or data races.

**Prevention:**
```rust
// Use parking_lot for better Mutex performance
use parking_lot::Mutex; // Faster than std::sync::Mutex

// Avoid holding locks across await points
#[tauri::command]
async fn update_conversation(
    state: State<'_, DbState>,
    id: String,
    title: String
) -> Result<(), String> {
    // Don't hold lock across await!
    let result = {
        let mut db = state.db.lock();
        db.update_conversation(&id, &title)
    }; // Lock released here
    
    // Now safe to await
    if result.is_ok() {
        notify_ui_of_update(&id).await;
    }
    
    result
}
```

**Phase to address:** Phase 1 (Core Infrastructure)

---

## Minor Pitfalls

### Pitfall 14: Environment Variable Changes

Tauri 2.0 renamed many environment variables:
- `TAURI_PRIVATE_KEY` → `TAURI_SIGNING_PRIVATE_KEY`
- `TAURI_KEY_PASSWORD` → `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
- `TAURI_DEBUG` → `TAURI_ENV_DEBUG`

**Phase to address:** Phase 4 (CI/CD Setup)

---

### Pitfall 15: Tray Icon/Menu on Linux

System tray on Linux requires specific libraries and may not work consistently across distributions.

**Prevention:**
```bash
# Ubuntu/Debian
sudo apt install libayatana-appindicator3-dev

# Or use the environment variable
export TAURI_LINUX_AYATANA_APPINDICATOR=true
```

**Phase to address:** Phase 2 (System Tray feature)

---

## Phase-Specific Warnings

| Phase | Topic | Likely Pitfall | Mitigation |
|-------|-------|----------------|------------|
| 1 | Database Setup | SQLite connection sharing | Use `r2d2_sqlite` or per-command connections |
| 1 | Core Chat | Large conversation history | Implement pagination/virtualization |
| 1 | API Integration | API key storage | Use OS keychain via `keyring` crate |
| 1 | UI Shell | Menu/Window API changes | Use v2 `WebviewWindow` and `MenuBuilder` |
| 2 | Multi-provider | HTTP client | Add `tauri-plugin-http` |
| 2 | Character Cards | Large file handling | Use `tauri::ipc::Response` for binary data |
| 3 | Group Chat | Concurrent message handling | Proper async/await with tokio |
| 4 | Distribution | Code signing | Configure early, test in CI |
| All | Event handling | Memory leaks | Always call `unlisten()` |

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| SQLite Thread Safety | HIGH | Verified via rusqlite docs and `!Send` trait |
| IPC Limitations | HIGH | Official Tauri IPC documentation |
| Tauri 2.0 Changes | HIGH | Official migration guide |
| Memory Leaks | MEDIUM | GitHub issues + official event docs |
| Security | HIGH | keyring crate is standard solution |
| Build/Distribution | HIGH | Official Tauri signing docs |
| React Performance | MEDIUM | General React knowledge + webview constraints |

---

## Sources

- Tauri 2.0 Migration Guide: https://tauri.app/start/migrate/from-tauri-1/
- Tauri IPC Documentation: https://v2.tauri.app/concept/inter-process-communication/
- Tauri Security: https://v2.tauri.app/security/
- rusqlite Documentation: https://docs.rs/rusqlite/latest/rusqlite/
- rusqlite_migration: https://docs.rs/rusqlite_migration/
- Tauri State Management: https://v2.tauri.app/develop/state-management/
- Tauri macOS Signing: https://v2.tauri.app/distribute/sign/macos/
- Tauri Windows Signing: https://v2.tauri.app/distribute/sign/windows/
- GitHub Issues: tauri-apps/tauri (memory leaks, event handling)
