# Phase 2 Research: Persistent Data

**Gathered:** 2026-05-11
**Status:** Complete

---

## User Constraints

> Copied verbatim from 02-CONTEXT.md. Planner MUST honor these.

### Locked Decisions

- **D-01:** Port the gtk_ref schema as the starting point — it's battle-tested and covers all required tables. Do NOT redesign from scratch.
- **D-02:** Evolve the schema where the frontend types reveal gaps — specifically, add `title` and `last_message_preview` columns to `conversations`.
- **D-03:** Flatten the message model for Phase 2 — simplified `messages` table with `id, conversation_id, role, content, sort_order, timestamp`. `branch_id` and `message_branches` deferred to Phase 8.
- **D-04:** Keep FTS5 virtual tables from gtk_ref (e.g., `characters_fts`) — needed for CONV-03 search.
- **D-05:** Use a dedicated `std::sync::mpsc` channel-based background worker for DB operations, following the gtk_ref `DbService` pattern — not a simple `Mutex<Connection>`.
- **D-06:** In Tauri, implement the worker as: a `std::thread::spawn` that owns the `rusqlite::Connection` and listens on a `mpsc::Receiver<DbCommand>`. Tauri commands send `DbCommand` variants via the sender half and await results via `oneshot` channels.
- **D-07:** WAL mode + busy_timeout=5000 + foreign_keys=ON — carried forward from gtk_ref migration 1.
- **D-08:** Tauri commands return `serde_json::Value` or typed structs that match the frontend TypeScript interfaces. Rust domain models are the source of truth.
- **D-09:** Timestamps use ISO 8601 strings (`String`) in both Rust and TypeScript. Replace gtk_ref `NaiveDateTime` with `chrono::DateTime<Utc>` serialized as ISO strings.
- **D-10:** UUIDs as text strings (`TEXT PRIMARY KEY`) — carried forward from gtk_ref.
- **D-11:** Do NOT ship built-in characters or personas in the database — the app starts with an empty library.
- **D-12:** Ship built-in generation presets (Creative, Balanced, Precise) — same as gtk_ref's `seed_builtin_presets`.
- **D-13:** Add `title TEXT NOT NULL DEFAULT ''` and `last_message_preview TEXT NOT NULL DEFAULT ''` columns to `conversations` table. Updated on every new message save.
- **D-14:** Compute `message_count` dynamically via `SELECT COUNT(*)` — not as a denormalized column.
- **D-15:** One Tauri command per CRUD operation. Follow the `DbInput`/`DbOutput` enum pattern from gtk_ref internally but expose flat commands to the frontend.
- **D-16:** Commands are async and return `Result<T, String>` — Tauri's standard error convention.
- **D-17:** Replace `hydrate()` in `+layout.svelte` with Tauri `invoke()` calls. Keep `AppState` class structure — just change the data source.
- **D-18:** Add `is_loading = $state<boolean>(true)` to `AppState`. UI shows skeleton during initial fetch.
- **D-19:** Remove `src/lib/data/mock.ts` after wiring is complete.

### Claude's Discretion

None — all decisions were auto-selected in yolo mode.

### Deferred Ideas

None — discussion stayed within phase scope.

---

## Standard Stack

### Rust Crates (add to `src-tauri/Cargo.toml`)

| Crate                | Version                   | Purpose                                     | Source                                               |
| -------------------- | ------------------------- | ------------------------------------------- | ---------------------------------------------------- |
| `rusqlite`           | `0.39`                    | SQLite bindings                             | `[VERIFIED: cargo search]` — matches gtk_ref version |
| `rusqlite_migration` | `2.5`                     | Schema migration runner                     | `[VERIFIED: cargo search]` — matches gtk_ref version |
| `chrono`             | `0.4` (features: `serde`) | Timestamp handling, ISO 8601 serialization  | `[VERIFIED: cargo search]` — matches gtk_ref version |
| `uuid`               | `1.x` (features: `v4`)    | UUID generation for primary keys            | `[VERIFIED: cargo search]` — matches gtk_ref version |
| `dirs`               | `6.0`                     | Platform-specific data directory resolution | `[VERIFIED: cargo search]` — matches gtk_ref version |
| `anyhow`             | `1.0`                     | Application error handling                  | `[ASSUMED]` — standard Rust error crate              |
| `thiserror`          | `2.0`                     | Domain error types                          | `[ASSUMED]` — standard Rust error derive crate       |
| `tracing`            | `0.1`                     | Structured logging                          | `[ASSUMED]` — standard Rust tracing crate            |

**rusqlite features to enable:**

- `bundled` — bundles SQLite so users don't need it installed. Essential for cross-platform distribution. `[VERIFIED: cargo search]`
- `functions` — needed if we use custom SQL functions (not critical for Phase 2, but gtk_ref enables it) `[ASSUMED]`
- `window` — needed for window functions (not critical for Phase 2) `[ASSUMED]`

**Minimal Phase 2 feature set:** `bundled` is the only hard requirement. `functions` and `window` can be added later if needed.

### TypeScript / Frontend

| Package           | Purpose                  | Source                                                     |
| ----------------- | ------------------------ | ---------------------------------------------------------- |
| `@tauri-apps/api` | `invoke()` for Tauri IPC | `[VERIFIED: codebase grep]` — already installed in Phase 1 |

No new npm dependencies needed for Phase 2.

---

## Architecture Patterns

### 1. Background-Thread DB Worker (D-05, D-06)

**Pattern:** `std::thread::spawn` owns `rusqlite::Connection`. Tauri commands send `DbCommand` via `mpsc::Sender`, receive results via `tokio::sync::oneshot`. `[CITED: gtk_ref/src/db/mod.rs L137-428]`

```
┌─────────────────┐     mpsc::Sender<DbCommand>     ┌──────────────────┐
│  Tauri Command  │ ──────────────────────────────→  │  DB Worker      │
│  (async fn)     │                                  │  (std::thread)   │
│                 │  ←──── oneshot::Receiver<T> ──── │  owns Connection│
└─────────────────┘                                  └──────────────────┘
```

**Implementation structure:**

```rust
// src-tauri/src/db/mod.rs

pub enum DbCommand {
    LoadCharacters { reply: oneshot::Sender<Result<Vec<CharacterCardV2>, String>> },
    SaveCharacter { character: CharacterCardV2, reply: oneshot::Sender<Result<CharacterCardV2, String>> },
    DeleteCharacter { id: String, reply: oneshot::Sender<Result<String, String>> },
    // ... other commands
}

pub struct DbWorker {
    sender: mpsc::Sender<DbCommand>,
}

impl DbWorker {
    pub fn start(db_path: PathBuf) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel::<DbCommand>();

        std::thread::spawn(move || {
            let mut conn = Connection::open(&db_path)
                .unwrap_or_else(|e| panic!("DB open failed: {}", e));
            migrations::run_migrations(&mut conn)
                .unwrap_or_else(|e| panic!("Migrations failed: {}", e));
            generation::seed_builtin_presets(&conn)
                .unwrap_or_else(|e| panic!("Preset seed failed: {}", e));

            while let Ok(cmd) = receiver.recv() {
                match cmd {
                    DbCommand::LoadCharacters { reply } => {
                        let result = characters::load_characters(&conn)
                            .map_err(|e| e.to_string());
                        let _ = reply.send(result);
                    }
                    // ... other matches
                }
            }
        });

        Ok(DbWorker { sender })
    }

    pub async fn send<T: Send + 'static>(
        &self,
        cmd_factory: impl FnOnce(oneshot::Sender<Result<T, String>>) -> DbCommand,
    ) -> Result<T, String> {
        let (reply_tx, reply_rx) = oneshot::channel();
        self.sender.send(cmd_factory(reply_tx))
            .map_err(|e| format!("DB worker channel error: {}", e))?;
        reply_rx.await
            .map_err(|e| format!("DB worker oneshot error: {}", e))?
    }
}
```

**Why `tokio::sync::oneshot` (not `std::sync::mpsc` for replies):** Tauri commands run on tokio runtime. The oneshot future integrates naturally with `.await`. Using `std::sync::mpsc` for replies would require blocking waits or polling. `[ASSUMED]`

**Why `std::sync::mpsc` for commands (not `tokio::sync::mpsc`):** The DB worker thread is a plain `std::thread`, not a tokio task. `std::sync::mpsc::Receiver::recv()` is blocking, which is correct for a thread that blocks on SQLite calls. `tokio::sync::mpsc` would require a tokio runtime on the worker thread, adding unnecessary complexity. `[ASSUMED]`

### 2. Tauri State Management for DB Worker

**Pattern:** Store `DbWorker` in Tauri's managed state via `app.manage()`. Commands access it via `State<'_, DbWorker>`. `[CITED: Tauri v2 docs — tauri::Manager::manage]`

```rust
// src-tauri/src/lib.rs

pub fn run() {
    let db_path = dirs::data_dir()
        .expect("Could not determine data directory")
        .join("com.revelry.desktop")
        .join("revelry.db");

    let db_worker = DbWorker::start(db_path)
        .expect("Failed to start database worker");

    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(db_worker)
        .invoke_handler(tauri::generate_handler![
            commands::get_characters,
            commands::save_character,
            commands::delete_character,
            commands::search_characters,
            commands::get_personas,
            commands::save_persona,
            commands::delete_persona,
            commands::get_conversations,
            commands::get_messages,
            commands::start_conversation,
            commands::save_message,
            commands::delete_conversation,
            commands::get_presets,
            commands::init_db,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Key detail:** `DbWorker` must be `Send + Sync` because Tauri's `State` requires it. The `mpsc::Sender` is `Send + Sync`, and the `oneshot` channels are created per-request. The `Connection` stays on the worker thread and is never shared. `[ASSUMED]`

### 3. Tauri Command Handlers (D-15, D-16)

**Pattern:** Each command is an async function that:

1. Gets `DbWorker` from Tauri state
2. Creates a oneshot channel
3. Sends a `DbCommand` variant with the oneshot sender
4. Awaits the oneshot result
5. Returns `Result<T, String>` to the frontend

```rust
#[tauri::command]
async fn get_characters(
    db: State<'_, DbWorker>,
) -> Result<Vec<CharacterCardV2>, String> {
    db.send(|reply| DbCommand::LoadCharacters { reply }).await
}

#[tauri::command]
async fn save_character(
    db: State<'_, DbWorker>,
    character: CharacterCardV2,
) -> Result<CharacterCardV2, String> {
    db.send(|reply| DbCommand::SaveCharacter { character, reply }).await
}
```

**Error convention:** All commands return `Result<T, String>`. Tauri serializes `Err(String)` as a rejected promise on the frontend, caught via `invoke().catch()`. `[CITED: Tauri v2 docs — command error handling]`

### 4. Schema Migration Strategy (D-01, D-02, D-03, D-04, D-07, D-13)

**Pattern:** Port gtk_ref's 9 migrations, then add a Phase 2 evolution migration. Use `rusqlite_migration` crate. `[CITED: gtk_ref/src/db/migrations.rs L5-200]`

**Migration sequence for Phase 2:**

| #      | Source          | Description                                                                                                                                                                                                                                        |
| ------ | --------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1      | gtk_ref M1      | providers table + WAL/busy_timeout/foreign_keys pragmas                                                                                                                                                                                            |
| 2      | gtk_ref M2      | app_settings table                                                                                                                                                                                                                                 |
| 3      | gtk_ref M3      | characters table + FTS5 + triggers                                                                                                                                                                                                                 |
| 4      | gtk_ref M4      | creator_notes + character_book columns on characters                                                                                                                                                                                               |
| 5      | gtk_ref M5      | conversations, message_branches, messages tables + indexes                                                                                                                                                                                         |
| 6      | gtk_ref M6      | model_name on providers                                                                                                                                                                                                                            |
| 7      | gtk_ref M7      | personas table + default_persona_id on characters                                                                                                                                                                                                  |
| 8      | gtk_ref M8      | context_window_size on providers, lorebook_entries table, lorebook fields on characters                                                                                                                                                            |
| 9      | gtk_ref M9      | generation_presets + regex_scripts tables + default_preset_id on characters + generation_preset_id on conversations                                                                                                                                |
| **10** | **Phase 2 NEW** | **Add `title` + `last_message_preview` to conversations (D-02, D-13). Drop `branch_id` from messages and `message_branches` table (D-03). Drop `provider_id` NOT NULL constraint on conversations (allow null until providers exist in Phase 3).** |

**Migration 10 specifics (D-02, D-03, D-13):**

```sql
-- Add denormalized sidebar fields to conversations
ALTER TABLE conversations ADD COLUMN title TEXT NOT NULL DEFAULT '';
ALTER TABLE conversations ADD COLUMN last_message_preview TEXT NOT NULL DEFAULT '';

-- Flatten message model for Phase 2 (branching deferred to Phase 8)
-- SQLite doesn't support DROP COLUMN before 3.35.0, but rusqlite bundled includes 3.39+
-- However, we can't easily drop columns referenced by FK constraints.
-- Strategy: Keep branch_id column as nullable (it already is), don't create
-- message_branches rows. The frontend won't use branch_id.
-- Phase 8 will add proper branching support.

-- Make provider_id nullable (no providers until Phase 3)
-- SQLite doesn't support ALTER COLUMN. Workaround: recreate table.
-- Actually, gtk_ref already has provider_id as `TEXT NOT NULL REFERENCES providers(id) ON DELETE SET NULL`
-- This is a problem because there are no providers in Phase 2.
-- Solution: Drop and recreate conversations table without NOT NULL on provider_id.
-- But this is destructive with existing data.
-- Better solution: Seed a default provider row OR make migration 5 use nullable provider_id from the start.
```

**Critical decision for migration 5 (conversations table):** The gtk_ref schema has `provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE SET NULL`. This is contradictory (NOT NULL + ON DELETE SET NULL). In Phase 2, there are no providers (Phase 3). **The planner must adjust migration 5 to make `provider_id` nullable** (`TEXT REFERENCES providers(id) ON DELETE SET NULL`) so conversations can be created without a provider. `[VERIFIED: codebase review of gtk_ref/src/db/migrations.rs L95-117]`

**Alternative approach for Migration 10:** Rather than complex ALTER TABLE gymnastics, since Phase 2 is the _first real use_ of this database (no production data exists yet), the planner should consider folding all 10 migrations into a cleaner set that reflects the Phase 2 schema from the start. The `rusqlite_migration` crate uses `user_version` to track applied migrations, and since no user has run any migrations yet, a clean rewrite is safe. `[ASSUMED — requires planner confirmation]`

### 5. Domain Model Port (D-08, D-09, D-10)

**Pattern:** Port gtk_ref domain structs to `src-tauri/src/domain/`. Key changes from gtk_ref: `[CITED: gtk_ref/src/domain/character.rs, conversation.rs, persona.rs, generation.rs]`

**Change: `NaiveDateTime` → `chrono::DateTime<Utc>` (D-09)**

gtk_ref uses `NaiveDateTime` and serializes as `"%Y-%m-%d %H:%M:%S%.f"`. For Tauri IPC:

- Replace with `DateTime<Utc>` (timezone-aware)
- Serialize as ISO 8601: `"2026-05-11T14:30:00Z"`
- Frontend TypeScript uses `string` for timestamps — no change needed
- `chrono::DateTime<Utc>` has built-in serde support via `chrono/serde` feature

```rust
// Before (gtk_ref):
pub created_at: NaiveDateTime,
// After (src-tauri):
pub created_at: DateTime<Utc>,
```

**Change: Add `title` and `last_message_preview` to `Conversation` (D-02, D-13)**

```rust
pub struct Conversation {
    pub id: String,
    pub character_id: String,
    pub provider_id: Option<String>,  // nullable — no providers in Phase 2
    pub creator_persona_id: Option<String>,
    pub generation_preset_id: Option<String>,
    pub title: String,                    // NEW — D-13
    pub last_message_preview: String,     // NEW — D-13
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Change: Flatten `Message` model (D-03)**

```rust
// Before (gtk_ref):
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub branch_id: Option<String>,  // deferred to Phase 8
    pub role: MessageRole,
    pub content: String,
    pub timestamp: NaiveDateTime,
    pub sort_order: i32,
}

// After (src-tauri):
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sort_order: i32,
    pub timestamp: DateTime<Utc>,
}
```

**Type alignment: Rust ↔ TypeScript mapping (D-08)**

| Rust field             | Rust type            | SQLite type   | TypeScript type           | Notes            |
| ---------------------- | -------------------- | ------------- | ------------------------- | ---------------- |
| `id`                   | `String`             | `TEXT PK`     | `string`                  | UUID v4 text     |
| `created_at`           | `DateTime<Utc>`      | `TEXT`        | `string`                  | ISO 8601         |
| `tags`                 | `Vec<String>`        | `TEXT` (JSON) | `string[]`                | serde_json       |
| `extensions`           | `serde_json::Value`  | `TEXT` (JSON) | `Record<string, unknown>` | serde_json       |
| `alternate_greetings`  | `Vec<String>`        | `TEXT` (JSON) | `string[]`                | serde_json       |
| `role`                 | `MessageRole` (enum) | `TEXT`        | `MessageRole` (type)      | lowercase string |
| `is_builtin`           | `bool`               | `INTEGER`     | —                         | 0/1              |
| `title`                | `String`             | `TEXT`        | `string`                  | NEW              |
| `last_message_preview` | `String`             | `TEXT`        | `string`                  | NEW              |

**Frontend type `MessageRole` mismatch:** Frontend uses `'character'` while Rust/db uses `'assistant'`. This must be reconciled. Two options:

1. Change the Rust `MessageRole::Assistant` to serialize as `"character"` — matches the frontend, deviates from AI API convention
2. Change the frontend `MessageRole` to use `'assistant'` — matches Rust and AI APIs, requires updating all frontend message rendering

**Recommendation:** Option 2 — use `'assistant'` in both places. It aligns with AI API conventions and is more correct. The frontend currently uses `'character'` as a roleplay-specific alias, but this creates a translation layer that will cause bugs. The planner should update the TypeScript type and all references. `[ASSUMED]`

### 6. Row Mapper Pattern

**Pattern:** Port gtk*ref's `row_to*\*`functions from`models.rs`. Adapt timestamp parsing from `NaiveDateTime`format string to`DateTime<Utc>`. `[CITED: gtk_ref/src/db/models.rs]`

```rust
// Before (gtk_ref):
let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")?;

// After (src-tauri):
let created_at: DateTime<Utc> = created_at_str.parse()
    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
        0, rusqlite::types::Type::Text, Box::new(e),
    ))?;
```

`DateTime<Utc>` implements `FromStr` for ISO 8601 strings, so `.parse()` works directly. `[CITED: docs.rs/chrono — DateTime::parse_from_rfc3339]`

### 7. Frontend Hydration Flow (D-17, D-18)

**Pattern:** Replace mock imports with parallel `invoke()` calls in `onMount`. `[CITED: src/routes/+layout.svelte]`

```typescript
// +layout.svelte — Phase 2
onMount(async () => {
  app.is_loading = true;

  try {
    // Parallel fetch — all independent
    const [characters, personas, conversations, presets] = await Promise.all([
      invoke<CharacterCard[]>("get_characters"),
      invoke<UserPersona[]>("get_personas"),
      invoke<Conversation[]>("get_conversations"),
      invoke<PromptPreset[]>("get_presets"),
    ]);

    app.hydrate({
      characters,
      personas,
      conversations,
      providers: [],
      prompt_presets: presets,
    });
  } catch (e) {
    app.init_error = String(e);
  } finally {
    app.is_loading = false;
  }
});
```

**Key detail:** `providers` array is empty in Phase 2 (providers are Phase 3). The `hydrate()` signature needs adjustment to make `providers` optional or accept an empty array. `[CITED: src/lib/stores/app.svelte.ts]`

**`AppState` additions (D-18):**

```typescript
is_loading = $state<boolean>(true);
init_error = $state<string | null>(null);
```

### 8. Conversation Creation and Message Save (D-13)

**Pattern:** When creating a conversation, set `title` from character name (e.g., "Chat with Eleanor Vance") and `last_message_preview` from the character's `first_mes`. When saving a message, update the conversation's `last_message_preview` and `updated_at`. `[CITED: gtk_ref/src/db/conversations.rs L9-131]`

```rust
// start_conversation — adapted from gtk_ref
pub fn start_conversation(conn: &Connection, character: &CharacterCardV2) -> Result<Conversation> {
    let mut conversation = Conversation::new(character.id.clone());
    conversation.title = format!("Chat with {}", character.name);  // D-13
    conversation.last_message_preview = character.first_mes.clone();  // D-13

    // Insert conversation...
    // Create initial assistant message from first_mes...
}
```

```rust
// save_message — update denormalized fields
pub fn save_message(conn: &Connection, msg: &Message) -> Result<()> {
    // Insert message...

    // Update conversation denormalized fields (D-13)
    let now = Utc::now();
    conn.prepare_cached(
        "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3"
    )?.execute(params![
        now.to_rfc3339(),
        // Truncate preview to ~100 chars
        msg.content.chars().take(100).collect::<String>(),
        &msg.conversation_id,
    ])?;

    Ok(())
}
```

### 9. Database File Location

**Pattern:** Use `dirs::data_dir()` for platform-appropriate path. `[CITED: dirs crate docs]`

| Platform | Path                                                             |
| -------- | ---------------------------------------------------------------- |
| macOS    | `~/Library/Application Support/com.revelry.desktop/revelry.db`   |
| Linux    | `~/.local/share/com.revelry.desktop/revelry.db`                  |
| Windows  | `C:\Users\{user}\AppData\Roaming\com.revelry.desktop\revelry.db` |

**Directory creation:** Must create the parent directory if it doesn't exist before opening the database. `std::fs::create_dir_all()` before `Connection::open()`. `[ASSUMED]`

### 10. Empty Library Experience (D-11)

Since no built-in characters/personas are shipped, the app's first-run experience is empty. The UI-SPEC defines empty state patterns for all views. The planner must ensure:

- Empty state components are rendered when `app.characters.length === 0`, etc.
- The "New character" / "Import card" CTAs exist (even if import isn't wired until Phase 6)
- Generation presets ARE populated (D-12) — `get_presets` always returns at least 3 built-in presets

`[CITED: .planning/phases/02-persistent-data/02-UI-SPEC.md L101-201]`

---

## Don't Hand-Roll

| Problem                                  | Use This Instead                                                                 | Source                                           |
| ---------------------------------------- | -------------------------------------------------------------------------------- | ------------------------------------------------ |
| SQLite connection management             | `rusqlite::Connection` with `bundled` feature                                    | `[VERIFIED: cargo search]`                       |
| Schema versioning                        | `rusqlite_migration` crate — auto-runs on startup, uses `user_version`           | `[VERIFIED: cargo search]`                       |
| UUID generation                          | `uuid::Uuid::new_v4()` — not manual string generation                            | `[CITED: gtk_ref/src/domain/character.rs L184]`  |
| Platform data directory                  | `dirs::data_dir()` — not platform-specific `if` blocks                           | `[CITED: dirs crate docs]`                       |
| JSON serialization of list/object fields | `serde_json::to_string()` / `serde_json::from_str()` — not manual string joining | `[CITED: gtk_ref/src/db/characters.rs L12-14]`   |
| FTS5 query sanitization                  | `sanitize_fts_query()` from gtk_ref — wraps in double quotes after escaping      | `[CITED: gtk_ref/src/db/characters.rs L113-120]` |
| Timestamp serialization                  | `chrono::DateTime<Utc>.to_rfc3339()` — not manual format strings                 | `[CITED: chrono docs]`                           |
| DB worker channel pattern                | `std::sync::mpsc` + `tokio::sync::oneshot` — not `Mutex<Connection>`             | `[CITED: gtk_ref/src/db/mod.rs]`                 |
| Tauri command error handling             | `Result<T, String>` — not custom error types serialized manually                 | `[CITED: Tauri v2 docs]`                         |

---

## Common Pitfalls

### 1. `rusqlite::Connection` is `!Send`

`Connection` cannot be moved across threads. The background worker thread must own it exclusively. Never try to share it via `Arc<Mutex<Connection>>` — this technically compiles but violates rusqlite's safety contract for certain operations. `[CITED: rusqlite docs — "Connection is not Send"]`

### 2. SQLite `NOT NULL` + `ON DELETE SET NULL` Contradiction

The gtk_ref migration 5 has `provider_id TEXT NOT NULL REFERENCES providers(id) ON DELETE SET NULL`. This is contradictory — `SET NULL` can't work on a `NOT NULL` column. In Phase 2, conversations need to be created without a provider (providers are Phase 3). **The planner must fix this in the initial migration set.** `[VERIFIED: codebase review of gtk_ref/src/db/migrations.rs L95]`

### 3. FTS5 Triggers Must Stay in Sync

FTS5 content-sync triggers (`characters_ai`, `_ad`, `_au`) must be created alongside the `characters` table. If they're missing, searches silently return no results. Port them verbatim from gtk_ref migration 3. `[CITED: gtk_ref/src/db/migrations.rs L56-85]`

### 4. Frontend `MessageRole` Mismatch: `'character'` vs `'assistant'`

The frontend TypeScript type uses `'character'` while the Rust domain and SQLite store `'assistant'`. If this isn't reconciled, messages saved as `'assistant'` won't match the frontend's `'character'` check, and the UI will render all AI messages with the wrong style. **The planner must decide which direction to align and apply it consistently.** `[VERIFIED: codebase review of src/lib/types/index.ts L62]`

### 5. Conversation `title` Population on Creation

The `title` column is `NOT NULL DEFAULT ''`. If `start_conversation()` doesn't set a title, the sidebar shows blank entries. The initial title should be derived from the character name (e.g., "Chat with Eleanor Vance"). `[CITED: 02-CONTEXT.md D-13]`

### 6. `start_conversation` Requires a Character Load

The gtk_ref `start_conversation()` loads the character first (to get `first_mes` for the initial assistant message). This means the DB worker must handle the character load + conversation insert + initial message insert in a single command to avoid race conditions. Do NOT split this into three separate `invoke()` calls from the frontend. `[CITED: gtk_ref/src/db/conversations.rs L9-45]`

### 7. Foreign Key Order Matters

Characters must exist before conversations referencing them. Migrations handle table creation order, but **runtime data** must also respect FK order: a conversation requires a character_id that exists in the characters table. Since we don't seed characters (D-11), the user must create or import a character before starting a conversation. The UI must enforce this ordering. `[ASSUMED]`

### 8. `Promise.all` Partial Failure

If one of the parallel `invoke()` calls in `onMount` fails, `Promise.all` rejects entirely and no data loads. Use `Promise.allSettled` instead, then handle partial failures per the UI-SPEC's "Individual Command Failure" pattern. `[ASSUMED]`

```typescript
const results = await Promise.allSettled([
  invoke<CharacterCard[]>("get_characters"),
  invoke<UserPersona[]>("get_personas"),
  invoke<Conversation[]>("get_conversations"),
  invoke<PromptPreset[]>("get_presets"),
]);
```

### 9. Migration 10 and Existing Data

If we port all 9 gtk_ref migrations and add migration 10 for the Phase 2 schema changes, we need migration 10 to handle existing data gracefully:

- `title` and `last_message_preview` on conversations should backfill from existing data
- Since Phase 2 is the first real use, there IS no existing data — so `DEFAULT ''` is sufficient
- But the planner should verify: are we running these migrations fresh or is there test data?

**Simpler alternative:** Since no user has ever run these migrations, collapse all migrations into a single clean migration set that produces the final Phase 2 schema directly. `rusqlite_migration` uses `user_version`, and a fresh database starts at 0. `[ASSUMED]`

### 10. `DbCommand` Enum Size

The `DbCommand` enum will have ~15+ variants. Each variant carries a `oneshot::Sender`. The enum itself is not large (senders are pointers internally), but the `match` in the worker thread must handle all variants. Missing a variant is a compile error in Rust, so this is safe — but the planner should organize the match into sub-modules (characters, personas, conversations) for readability. `[ASSUMED]`

---

## Code Examples

### A. `DbCommand` Enum (adapted from gtk_ref `DbInput`)

```rust
// src-tauri/src/db/mod.rs

use tokio::sync::oneshot;
use crate::domain::*;

pub enum DbCommand {
    // Characters
    LoadCharacters { reply: oneshot::Sender<anyhow::Result<Vec<CharacterCardV2>>> },
    SaveCharacter { character: CharacterCardV2, reply: oneshot::Sender<anyhow::Result<CharacterCardV2>> },
    DeleteCharacter { id: String, reply: oneshot::Sender<anyhow::Result<()>> },
    SearchCharacters { query: String, reply: oneshot::Sender<anyhow::Result<Vec<CharacterCardV2>>> },

    // Personas
    LoadPersonas { reply: oneshot::Sender<anyhow::Result<Vec<Persona>>> },
    SavePersona { persona: Persona, reply: oneshot::Sender<anyhow::Result<Persona>> },
    DeletePersona { id: String, reply: oneshot::Sender<anyhow::Result<()>> },

    // Conversations
    LoadConversations { reply: oneshot::Sender<anyhow::Result<Vec<Conversation>>> },
    StartConversation { character_id: String, reply: oneshot::Sender<anyhow::Result<Conversation>> },
    DeleteConversation { id: String, reply: oneshot::Sender<anyhow::Result<()>> },
    SearchConversations { query: String, reply: oneshot::Sender<anyhow::Result<Vec<Conversation>>> },

    // Messages
    LoadMessages { conversation_id: String, reply: oneshot::Sender<anyhow::Result<Vec<Message>>> },
    SaveMessage { message: Message, reply: oneshot::Sender<anyhow::Result<()>> },

    // Presets
    LoadPresets { reply: oneshot::Sender<anyhow::Result<Vec<GenerationPreset>>> },
}
```

**Note on error types:** The oneshot channels carry `anyhow::Result`. When Tauri commands receive the result, they convert `anyhow::Error` to `String` via `.map_err(|e| e.to_string())`. This is the simplest approach for Tauri's `Result<T, String>` convention. `[ASSUMED]`

### B. Tauri Command Handler

```rust
// src-tauri/src/commands/characters.rs

use tauri::State;
use crate::db::DbWorker;
use crate::domain::CharacterCardV2;

#[tauri::command]
pub async fn get_characters(
    db: State<'_, DbWorker>,
) -> Result<Vec<CharacterCardV2>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadCharacters { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
```

### C. Frontend `invoke()` with Type Safety

```typescript
// src/lib/data/api.ts — typed wrappers over invoke()

import { invoke } from "@tauri-apps/api/core";
import type {
  CharacterCard,
  UserPersona,
  Conversation,
  Message,
  PromptPreset,
} from "$lib/types";

export async function getCharacters(): Promise<CharacterCard[]> {
  return invoke<CharacterCard[]>("get_characters");
}

export async function getPersonas(): Promise<UserPersona[]> {
  return invoke<UserPersona[]>("get_personas");
}

export async function getConversations(): Promise<Conversation[]> {
  return invoke<Conversation[]>("get_conversations");
}

export async function getMessages(conversationId: string): Promise<Message[]> {
  return invoke<Message[]>("get_messages", { conversationId });
}

export async function getPresets(): Promise<PromptPreset[]> {
  return invoke<PromptPreset[]>("get_presets");
}

export async function startConversation(
  characterId: string,
): Promise<Conversation> {
  return invoke<Conversation>("start_conversation", { characterId });
}

export async function saveMessage(message: Omit<Message, "id">): Promise<void> {
  return invoke("save_message", { message });
}

export async function deleteConversation(id: string): Promise<void> {
  return invoke("delete_conversation", { id });
}
```

**Creating a typed API layer** (`src/lib/data/api.ts`) instead of scattering `invoke()` calls throughout components has these benefits:

- Single place to update if command signatures change
- Type narrowing at the boundary
- Easier to mock for testing
- Consistent error handling

`[ASSUMED]` — planner should decide whether to use an API layer or call `invoke()` directly.

### D. SQLite Schema — Final Phase 2 State

The planner should produce this schema from the migrations (either 1 big migration or 10 sequential):

```sql
-- providers (kept for Phase 3, but table exists from M1)
CREATE TABLE providers (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    provider_type TEXT NOT NULL,
    base_url TEXT,
    model_name TEXT NOT NULL DEFAULT '',
    context_window_size INTEGER,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- app_settings
CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- characters + FTS5 (from M3, M4, M7, M8)
CREATE TABLE characters (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT NOT NULL DEFAULT '',
    personality TEXT NOT NULL DEFAULT '',
    scenario TEXT NOT NULL DEFAULT '',
    first_mes TEXT NOT NULL DEFAULT '',
    mes_example TEXT NOT NULL DEFAULT '',
    system_prompt TEXT NOT NULL DEFAULT '',
    post_history_instructions TEXT NOT NULL DEFAULT '',
    alternate_greetings TEXT NOT NULL DEFAULT '[]',
    tags TEXT NOT NULL DEFAULT '[]',
    creator TEXT NOT NULL DEFAULT 'Unknown',
    character_version TEXT NOT NULL DEFAULT '2.0',
    avatar_path TEXT,
    extensions TEXT NOT NULL DEFAULT 'null',
    creator_notes TEXT NOT NULL DEFAULT '',
    character_book TEXT NOT NULL DEFAULT 'null',
    default_persona_id TEXT,
    default_preset_id TEXT,
    lorebook_scan_depth INTEGER NOT NULL DEFAULT 2,
    lorebook_recursive_depth INTEGER NOT NULL DEFAULT 2,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE VIRTUAL TABLE characters_fts USING fts5(
    name, description, tags,
    content=characters,
    content_rowid=rowid
);
-- + INSERT, DELETE, UPDATE triggers (from gtk_ref M3)

-- conversations (from M5, M9, + Phase 2 additions)
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    provider_id TEXT REFERENCES providers(id) ON DELETE SET NULL,  -- NULLABLE for Phase 2
    creator_persona_id TEXT,
    generation_preset_id TEXT,
    title TEXT NOT NULL DEFAULT '',              -- Phase 2 addition (D-13)
    last_message_preview TEXT NOT NULL DEFAULT '', -- Phase 2 addition (D-13)
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);

-- messages (from M5, simplified for Phase 2 — D-03)
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('user','assistant','system')),
    content TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT NOT NULL
    -- branch_id omitted for Phase 2 (D-03)
);

CREATE INDEX idx_messages_conv_order ON messages(conversation_id, sort_order);

-- message_branches table DEFERRED to Phase 8 (D-03)

-- personas (from M7)
CREATE TABLE personas (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    appearance TEXT NOT NULL DEFAULT '',
    personality TEXT NOT NULL DEFAULT '',
    background TEXT NOT NULL DEFAULT '',
    role_description TEXT NOT NULL DEFAULT '',
    relationships TEXT NOT NULL DEFAULT '',
    speech_style TEXT NOT NULL DEFAULT '',
    tags TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- generation_presets (from M9)
CREATE TABLE generation_presets (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    is_builtin INTEGER NOT NULL DEFAULT 0,
    temperature REAL NOT NULL DEFAULT 0.8,
    top_p REAL NOT NULL DEFAULT 0.9,
    max_tokens INTEGER NOT NULL DEFAULT 0,
    frequency_penalty REAL NOT NULL DEFAULT 0.0,
    presence_penalty REAL NOT NULL DEFAULT 0.0,
    stop_sequences TEXT NOT NULL DEFAULT '[]',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- lorebook_entries (from M8, unused until Phase 9)
CREATE TABLE lorebook_entries (
    id TEXT PRIMARY KEY,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    keywords TEXT NOT NULL DEFAULT '[]',
    secondary_terms TEXT NOT NULL DEFAULT '[]',
    content TEXT NOT NULL DEFAULT '',
    insertion_position TEXT NOT NULL DEFAULT 'after_char',
    depth_from_bottom INTEGER NOT NULL DEFAULT 4,
    enabled INTEGER NOT NULL DEFAULT 1,
    token_budget INTEGER,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- regex_scripts (from M9, unused until Phase 9)
CREATE TABLE regex_scripts (
    id TEXT PRIMARY KEY,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    name TEXT NOT NULL DEFAULT '',
    find_pattern TEXT NOT NULL DEFAULT '',
    replace_pattern TEXT NOT NULL DEFAULT '',
    flags TEXT NOT NULL DEFAULT 'gi',
    sort_order INTEGER NOT NULL DEFAULT 0,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

-- Pragmas (from M1)
PRAGMA journal_mode=WAL;
PRAGMA busy_timeout=5000;
PRAGMA foreign_keys=ON;
```

### E. Builtin Preset Seeding

Directly ported from gtk_ref. Fixed UUIDs ensure stability. `[CITED: gtk_ref/src/db/generation.rs L7-32]`

```rust
// Called after migrations in DB worker init
pub fn seed_builtin_presets(conn: &Connection) -> Result<()> {
    let presets = [
        GenerationPreset::creative(),  // id: "builtin-creative", temp: 1.2
        GenerationPreset::balanced(),   // id: "builtin-balanced", temp: 0.8
        GenerationPreset::precise(),    // id: "builtin-precise", temp: 0.3
    ];
    for preset in &presets {
        let stop_str = serde_json::to_string(&preset.stop_sequences)?;
        conn.prepare_cached(
            "INSERT OR IGNORE INTO generation_presets
             (id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty,
              presence_penalty, stop_sequences, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        )?.execute(params![
            preset.id, preset.name, preset.is_builtin as i32,
            preset.temperature, preset.top_p, preset.max_tokens,
            preset.frequency_penalty, preset.presence_penalty,
            stop_str, preset.created_at.to_rfc3339(), preset.updated_at.to_rfc3339(),
        ])?;
    }
    Ok(())
}
```

### F. Conversation Search (CONV-03)

The gtk_ref schema doesn't have FTS5 on conversations, but the requirements need `CONV-03: User can search conversations by text`. Two approaches:

**Option A: FTS5 on conversations** — Add `conversations_fts` virtual table matching on `title`. Simple, but doesn't search message content.

**Option B: SQL LIKE query** — `WHERE title LIKE ?1 OR last_message_preview LIKE ?1`. Simpler, works for Phase 2's sidebar search, no extra tables. Sufficient until Phase 8 adds full FTS.

**Recommendation:** Option B for Phase 2. It covers the sidebar search use case without additional FTS complexity. Phase 8 will add full FTS across conversations and messages (CONV-08). `[ASSUMED]`

---

## Open Questions for Planner

These are NOT locked decisions — the planner should resolve them:

1. **Migration strategy:** Port all 9 gtk_ref migrations + add migration 10, OR collapse into a clean single migration set? (No existing data to preserve.)
2. **`MessageRole` alignment:** Change frontend `'character'` → `'assistant'`, or add translation layer in Tauri commands?
3. **Frontend API layer:** Create `src/lib/data/api.ts` with typed `invoke()` wrappers, or call `invoke()` directly from components?
4. **`Promise.all` vs `Promise.allSettled`:** Use `allSettled` for partial failure resilience, or `all` for simpler code (fail fast)?
5. **`DbCommand` error type:** Use `anyhow::Result` inside oneshot channels (convert to String at command boundary), or use `String` directly in channels?
6. **`start_conversation` without provider:** How should the conversation's `provider_id` be handled? Set to `NULL` and let Phase 3 add provider selection?

---

_Phase: 02-Persistent Data_
_Research gathered: 2026-05-11_
