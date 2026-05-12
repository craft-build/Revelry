# Architecture Patterns

**Domain:** Tauri desktop AI roleplay app with SvelteKit frontend

## Recommended Architecture

### Component Boundaries

```
┌─────────────────────────────────────────────────────┐
│                   Tauri Desktop Shell                 │
│                                                      │
│  ┌──────────────────────┐  ┌──────────────────────┐  │
│  │   SvelteKit Frontend │  │   Rust Backend        │  │
│  │   (Webview)          │  │   (src-tauri/)        │  │
│  │                      │  │                        │  │
│  │  ┌────────────────┐  │  │  ┌──────────────────┐  │  │
│  │  │  AppState       │  │  │  │  Tauri Commands  │  │  │
│  │  │  (Svelte 5      │◄─┼──┼─►│  (#[tauri::cmd]) │  │  │
│  │  │   runes)        │  │  │  │                  │  │  │
│  │  └────────────────┘  │  │  └────────┬─────────┘  │  │
│  │        │              │  │           │            │  │
│  │  ┌─────▼──────┐       │  │  ┌────────▼─────────┐  │  │
│  │  │  Components │      │  │  │  Service Layer    │  │  │
│  │  │  (Bits.ui)  │      │  │  │                    │  │  │
│  │  └────────────┘       │  │  │  ┌──────────────┐│  │  │
│  │                      │  │  │  │  AiService    ││  │  │
│  │  ┌────────────────┐  │  │  │  │  (rig-core)   ││  │  │
│  │  │  Tauri IPC     │  │  │  │  └──────────────┘│  │  │
│  │  │  invoke()      │──┼──┼─►│  ┌──────────────┐│  │  │
│  │  │  Channel<>     │  │  │  │  │  PromptEngine ││  │  │
│  │  │  listen()      │◄─┼──┼──┤  └──────────────┘│  │  │
│  │  └────────────────┘  │  │  │  ┌──────────────┐│  │  │
│  │                      │  │  │  │  DbService    ││  │  │
│  └──────────────────────┘  │  │  │  (SQLite)     ││  │  │
│                             │  │  └──────────────┘│  │  │
│                             │  │  ┌──────────────┐│  │  │
│                             │  │  │  KeyringStore ││  │  │
│                             │  │  └──────────────┘│  │  │
│                             │  │  ┌──────────────┐│  │  │
│                             │  │  │  TokenService ││  │  │
│                             │  │  └──────────────┘│  │  │
│                             │  │  ┌──────────────┐│  │  │
│                             │  │  │  MarkdownSvc ││  │  │
│                             │  │  └──────────────┘│  │  │
│                             │  │  ┌──────────────┐│  │  │
│                             │  │  │  CardIO       ││  │  │
│                             │  │  └──────────────┘│  │  │
│                             │  │  ┌──────────────┐│  │  │
│                             │  │  │  LorebookSvc ││  │  │
│                             │  │  └──────────────┘│  │  │
│                             │  │  ┌──────────────┐│  │  │
│                             │  │  │  RegexService ││  │  │
│                             │  │  └──────────────┘│  │  │
│                             │  └────────────────────┘  │
│                             │                          │
│                             │  ┌────────────────────┐  │
│                             │  │  Domain Models      │  │
│                             │  │  (Character, Conv,  │  │
│                             │  │   Persona, etc.)    │  │
│                             │  └────────────────────┘  │
│                             │                          │
│                             │  ┌────────────────────┐  │
│                             │  │  App State          │  │
│                             │  │  (tauri::State)     │  │
│                             │  └────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### Data Flow

**1. Chat Message Flow (the critical path)**

```
User types message → MessageInput.svelte
  → invoke('send_message', { conversationId, content })
    → Tauri command: send_message()
      → PromptEngine: assemble prompt (character + persona + lorebook + history)
      → TokenService: count tokens, check budget
      → AiService: stream_chat() via rig-core
        → For each token chunk:
          → Channel<StreamEvent>.send(TokenChunk { text })
        → On completion:
          → DbService: save user message + assistant message
          → Channel<StreamEvent>.send(Complete { messageId })
  → Frontend Channel.onmessage:
    → Append tokens to current assistant message
    → On Complete: finalize message in AppState
```

**2. Data Loading Flow**

```
Page load → +page.ts load function
  → invoke('list_conversations') or invoke('get_conversation', { id })
    → Tauri command → DbService → SQLite query
    → Return serialized domain models
  → Populate AppState
```

**3. Settings Flow**

```
Settings page → invoke('save_provider', { provider })
  → Tauri command: save_provider()
    → KeyringStore: save API key to OS keychain
    → DbService: save provider config (without key) to SQLite
  → Return success/error
```

### Key Architectural Decisions

**Tauri Commands vs Events:**

- **Commands (`invoke()`)**: Request-response — data loading, CRUD operations, settings
- **Channels (`Channel<T>`)**: Streaming — AI token streaming, long-running operations
- **Events (`emit()`/`listen()`)**: Unsolicited notifications — theme change, database changed, background task completed

**State Split:**

- **Frontend (AppState in Svelte 5 runes)**: UI state — current view, active conversation, sidebar open, theme, form state
- **Backend (tauri::State)**: Service instances — DbService, AiService, KeyringStore. NOT domain data (that's in SQLite)
- **SQLite**: Source of truth for all persistent domain data

**Service Layer Pattern (from gtk_ref/):**

- Each service is a struct with methods (not static functions)
- Services are instantiated in `main.rs` and managed via `tauri::State`
- Services access each other through `tauri::State` references, not direct injection
- Async services use tokio; sync services (rusqlite) use a background thread with mpsc channels

### Patterns to Follow

**Pattern 1: Tauri Command + Channel for Streaming**

The canonical Tauri v2 streaming pattern (verified via Context7):

```rust
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "event", content = "data")]
enum StreamEvent {
  Token { text: String },
  Complete { message_id: String },
  Error { message: String },
}

#[tauri::command]
async fn send_message(
  conversation_id: String,
  content: String,
  on_event: Channel<StreamEvent>,
  state: State<'_, AppState>,
) -> Result<(), String> {
  // ... stream tokens via on_event.send()
}
```

Frontend:

```typescript
const onEvent = new Channel<StreamEvent>();
onEvent.onmessage = (msg) => {
  if (msg.event === "token") appState.appendToken(msg.data.text);
  if (msg.event === "complete") appState.finalizeMessage(msg.data.messageId);
};
await invoke("send_message", { conversationId, content, onEvent });
```

**Pattern 2: Background DB Worker**

rusqlite is synchronous. To avoid blocking the Tauri main thread:

- Spawn a dedicated tokio task for DB operations
- Use `mpsc` channels to send DB requests and receive responses
- Or use `tokio::task::spawn_blocking` for individual queries
- The `gtk_ref/` `DbService` pattern (background-thread worker with `DbInput`/`DbOutput`) maps directly to this

**Pattern 3: Error Propagation Across IPC**

All Tauri commands return `Result<T, String>` (or a serializable error type). Map domain errors to user-friendly strings at the command boundary:

```rust
#[tauri::command]
async fn get_conversation(id: String, state: State<'_, AppServices>) -> Result<Conversation, String> {
  state.db.get_conversation(&id).await.map_err(|e| e.to_string())
}
```

### Anti-Patterns to Avoid

**Anti-Pattern 1: Synchronous DB in Command Handlers**

Never call rusqlite directly from a `#[tauri::command]`. The command runs on the Tauri main thread. Use `spawn_blocking` or a background worker.

**Anti-Pattern 2: Storing Domain State in tauri::State**

`tauri::State` should hold service instances, not domain data. Domain data lives in SQLite and is loaded on demand. Don't try to mirror the Svelte AppState in Rust.

**Anti-Pattern 3: Direct API Keys on Disk**

Never store API keys in SQLite, config files, or localStorage. Always use the keyring crate. This is a hard constraint from the design spec.

**Anti-Pattern 4: Prop Drilling State Through Components**

The existing SvelteKit app already avoids this by using the centralized `AppState` with Svelte 5 runes. Continue this pattern — don't pass callbacks through 5 layers of components.

### Suggested Build Order

Based on dependencies:

1. **Tauri shell + SvelteKit adapter** — Get the app running in a Tauri window first (no backend logic yet)
2. **Domain models + DB layer** — Foundation for everything else
3. **Keyring + Provider config** — Enables AI service
4. **AI service + Streaming** — The core value proposition
5. **Prompt engine + Token counting** — Makes AI conversations work correctly
6. **Markdown rendering** — Polish AI output
7. **Card I/O** — Character import/export
8. **Editors** — UI for creating/editing characters, personas
9. **Lorebook** — Advanced context injection
10. **Regex scripting** — Output post-processing
11. **Branching + FTS** — Advanced conversation features
12. **Theme toggle** — Polish
13. **Design doc update** — Can happen in parallel; not blocking any code

## Sources

- Tauri v2 official docs (Context7) — Commands, channels, state management, SvelteKit integration
- rig-core API docs (Context7) — Provider support, streaming API
- Existing codebase: `.planning/codebase/ARCHITECTURE.md`
- Archival reference: `gtk_ref/` architecture patterns (DbService, AiService, KeyringStore)
