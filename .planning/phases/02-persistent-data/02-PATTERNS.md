# Phase 2: Persistent Data — Pattern Mapping

**Phase:** 02-Persistent Data
**Generated:** 2026-05-11

This document maps each file to be created/modified in Phase 2 to its closest existing analog in the codebase, extracts concrete code excerpts showing the pattern to follow, and classifies each by role and data flow.

---

## File Inventory

| # | File | Role | Data Flow | Analog Source |
|---|------|------|-----------|---------------|
| 1 | `src-tauri/src/domain/mod.rs` | Module registry | — | `gtk_ref/src/domain/mod.rs` |
| 2 | `src-tauri/src/domain/character.rs` | Domain model | Rust ↔ DB ↔ TS | `gtk_ref/src/domain/character.rs` |
| 3 | `src-tauri/src/domain/conversation.rs` | Domain model | Rust ↔ DB ↔ TS | `gtk_ref/src/domain/conversation.rs` |
| 4 | `src-tauri/src/domain/persona.rs` | Domain model | Rust ↔ DB ↔ TS | `gtk_ref/src/domain/persona.rs` |
| 5 | `src-tauri/src/domain/generation.rs` | Domain model | Rust ↔ DB ↔ TS | `gtk_ref/src/domain/generation.rs` |
| 6 | `src-tauri/src/db/mod.rs` | DB worker + command enum | Rust ↔ Tauri IPC | `gtk_ref/src/db/mod.rs` |
| 7 | `src-tauri/src/db/migrations.rs` | Schema migrations | App startup → SQLite | `gtk_ref/src/db/migrations.rs` |
| 8 | `src-tauri/src/db/models.rs` | Row mappers | SQLite → Rust domain | `gtk_ref/src/db/models.rs` |
| 9 | `src-tauri/src/db/characters.rs` | Character CRUD | Rust domain ↔ SQLite | `gtk_ref/src/db/characters.rs` |
| 10 | `src-tauri/src/db/personas.rs` | Persona CRUD | Rust domain ↔ SQLite | `gtk_ref/src/db/personas.rs` |
| 11 | `src-tauri/src/db/conversations.rs` | Conversation/Message CRUD | Rust domain ↔ SQLite | `gtk_ref/src/db/conversations.rs` |
| 12 | `src-tauri/src/db/generation.rs` | Preset CRUD + seeding | Rust domain ↔ SQLite | `gtk_ref/src/db/generation.rs` |
| 13 | `src-tauri/src/commands/mod.rs` | Command registry | Tauri IPC → DB worker | — (new) |
| 14 | `src-tauri/src/commands/characters.rs` | Tauri command handlers | TS invoke → Rust → DB | — (new, pattern from RESEARCH) |
| 15 | `src-tauri/src/commands/conversations.rs` | Tauri command handlers | TS invoke → Rust → DB | — (new, pattern from RESEARCH) |
| 16 | `src-tauri/src/commands/personas.rs` | Tauri command handlers | TS invoke → Rust → DB | — (new, pattern from RESEARCH) |
| 17 | `src-tauri/src/commands/presets.rs` | Tauri command handlers | TS invoke → Rust → DB | — (new, pattern from RESEARCH) |
| 18 | `src-tauri/Cargo.toml` | Dependency manifest | — | `src-tauri/Cargo.toml` (extend) |
| 19 | `src-tauri/src/lib.rs` | App entry point | Startup → DB init + commands | `src-tauri/src/lib.rs` (extend) |
| 20 | `src/lib/stores/app.svelte.ts` | Frontend state | invoke() → AppState | `src/lib/stores/app.svelte.ts` (modify) |
| 21 | `src/lib/types/index.ts` | TypeScript types | Rust serde → TS interface | `src/lib/types/index.ts` (modify) |
| 22 | `src/routes/+layout.svelte` | App shell | onMount → invoke → hydrate | `src/routes/+layout.svelte` (modify) |
| 23 | `src/lib/data/mock.ts` | Remove after wiring | — | `src/lib/data/mock.ts` (delete) |
| 24 | `src/lib/utils/index.ts` | Utilities | Possibly update estimateTokens | `src/lib/utils/index.ts` (review) |

---

## Role Classifications

### Domain Models (Files 1–5)
**Role:** Pure data structs with serde serialization. Source of truth for the type contract between Rust and TypeScript. No business logic — just constructors, defaults, and serde derives.

**Data Flow:**
```
SQLite TEXT rows → models.rs row_to_*() → Domain struct → serde_json → Tauri IPC → TypeScript
TypeScript → Tauri IPC → serde_json → Domain struct → CRUD functions → SQLite TEXT
```

### DB Layer (Files 6–12)
**Role:** SQLite persistence. The background-thread worker owns the connection and processes commands sequentially. CRUD functions are pure functions taking `&Connection`. Row mappers translate between SQLite rows and domain structs.

**Data Flow:**
```
Tauri command → DbCommand enum → mpsc::Sender → worker thread → CRUD fn → Connection → SQLite
SQLite → Row → models.rs mapper → Domain struct → oneshot::Sender → Tauri command → Frontend
```

### Command Handlers (Files 13–17)
**Role:** Thin async wrappers that bridge Tauri IPC to the DB worker. Each handler creates a oneshot channel, sends a `DbCommand`, and awaits the result. No business logic.

**Data Flow:**
```
Frontend invoke('get_characters') → #[tauri::command] async fn → DbWorker::send() → DbCommand → oneshot result
```

### App Wiring (Files 18–19)
**Role:** Startup initialization. `Cargo.toml` adds the dependency graph. `lib.rs` creates the DB worker, manages it as Tauri state, and registers all command handlers.

**Data Flow:**
```
App launch → lib.rs::run() → DbWorker::start(db_path) → app.manage(db_worker) → invoke_handler([...])
```

### Frontend Integration (Files 20–24)
**Role:** Replace mock data with real IPC. Add loading state. Align TypeScript types with Rust serde output. Remove mock.ts.

**Data Flow:**
```
onMount → invoke('get_characters'), invoke('get_personas'), ... → Promise.allSettled → app.hydrate(data)
```

---

## Pattern Mapping — Detailed

### 1. Domain Models: `src-tauri/src/domain/`

#### Analog: `gtk_ref/src/domain/character.rs`

The gtk_ref `CharacterCardV2` struct is the direct porting target. Two key changes for the Tauri version:
- Replace `NaiveDateTime` with `DateTime<Utc>` (D-09)
- Keep all serde derives and `#[serde(default)]` annotations

**Excerpt — gtk_ref CharacterCardV2 (to port):**

```gtk_ref/src/domain/character.rs#L9-64
pub struct CharacterCardV2 {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub scenario: String,
    #[serde(default)]
    pub first_mes: String,
    #[serde(default)]
    pub mes_example: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub post_history_instructions: String,
    #[serde(default)]
    pub alternate_greetings: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_creator")]
    pub creator: String,
    #[serde(default = "default_character_version")]
    pub character_version: String,
    #[serde(default)]
    pub avatar_path: Option<String>,
    #[serde(default = "default_extensions")]
    pub extensions: serde_json::Value,
    /// Notes from the creator about the card (not used in prompt generation).
    /// Maps to V2 spec's `creator_notes` field.
    #[serde(default)]
    pub creator_notes: String,
    /// Character lorebook / world info (JSON, not used in prompt by default).
    /// Maps to V2 spec's `character_book` field.
    #[serde(default = "default_character_book")]
    pub character_book: serde_json::Value,
    /// Per-character default persona ID for persona assignment hierarchy.
    /// Maps to the `default_persona_id` column added in Migration 007.
    #[serde(default)]
    pub default_persona_id: Option<String>,
    /// Per-character default preset ID for generation parameter hierarchy per D-03.
    /// Maps to the `default_preset_id` column added in Migration 009.
    #[serde(default)]
    pub default_preset_id: Option<String>,
    /// Number of recent messages to scan for keyword matching when activating lorebook entries.
    /// Per D-10: per-character scan depth (default 2).
    #[serde(default = "default_lorebook_scan_depth")]
    pub lorebook_scan_depth: i32,
    /// Maximum depth of recursive lorebook activation (how many times activated entries can trigger other entries).
    /// Per D-12: recursive activation depth limit (default 2).
    #[serde(default = "default_lorebook_recursive_depth")]
    pub lorebook_recursive_depth: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

**Excerpt — gtk_ref CharacterCardV2::new() (to port):**

```gtk_ref/src/domain/character.rs#L181-212
impl CharacterCardV2 {
    /// Creates a new character with a generated UUID v4 id, current timestamps,
    /// character_version "2.0", and all other fields at their defaults.
    pub fn new(name: String) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_mes: String::new(),
            mes_example: String::new(),
            system_prompt: String::new(),
            post_history_instructions: String::new(),
            alternate_greetings: Vec::new(),
            tags: Vec::new(),
            creator: "Unknown".to_string(),
            character_version: "2.0".to_string(),
            avatar_path: None,
            extensions: serde_json::Value::Null,
            creator_notes: String::new(),
            character_book: serde_json::Value::Null,
            default_persona_id: None,
            default_preset_id: None,
            lorebook_scan_depth: 2,
            lorebook_recursive_depth: 2,
            created_at: now,
            updated_at: now,
        }
    }
}
```

**Changes for Tauri version:**
- `chrono::NaiveDateTime` → `chrono::DateTime<Utc>` for all timestamp fields
- Constructor uses `Utc::now()` instead of `Local::now().naive_local()`
- Add `#[serde(with = "chrono::serde::ts_seconds")]` or rely on `DateTime<Utc>`'s default ISO 8601 serde

---

#### Analog: `gtk_ref/src/domain/conversation.rs`

**Excerpt — gtk_ref Conversation (to port with modifications):**

```gtk_ref/src/domain/conversation.rs#L11-23
pub struct Conversation {
    pub id: String,
    pub character_id: String,
    pub provider_id: String,
    pub creator_persona_id: Option<String>,
    /// Per-conversation preset override per D-03. Resolved via hierarchy:
    /// conversation → character → global → none (provider defaults).
    pub generation_preset_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

**Excerpt — gtk_ref Message (to port, simplified per D-03):**

```gtk_ref/src/domain/conversation.rs#L53-63
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub branch_id: Option<String>,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: NaiveDateTime,
    pub sort_order: i32,
}
```

**Excerpt — gtk_ref MessageRole (to port):**

```gtk_ref/src/domain/conversation.rs#L27-52
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

impl FromStr for MessageRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            _ => Err(format!("Unknown message role: {}", s)),
        }
    }
}
```

**Changes for Tauri version:**
- `Conversation`: Change `provider_id: String` → `Option<String>` (nullable until Phase 3)
- `Conversation`: Add `title: String` and `last_message_preview: String` (D-13)
- `Message`: Remove `branch_id` (D-03 — branching deferred to Phase 8)
- All timestamps: `NaiveDateTime` → `DateTime<Utc>`

**Phase 2 Conversation struct:**
```rust
pub struct Conversation {
    pub id: String,
    pub character_id: String,
    pub provider_id: Option<String>,
    pub creator_persona_id: Option<String>,
    pub generation_preset_id: Option<String>,
    pub title: String,
    pub last_message_preview: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
```

**Phase 2 Message struct:**
```rust
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sort_order: i32,
    pub timestamp: DateTime<Utc>,
}
```

---

#### Analog: `gtk_ref/src/domain/persona.rs`

**Excerpt — gtk_ref Persona (to port with minimal changes):**

```gtk_ref/src/domain/persona.rs#L7-26
pub struct Persona {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub appearance: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub role_description: String,
    #[serde(default)]
    pub relationships: String,
    #[serde(default)]
    pub speech_style: String,
    #[serde(default)]
    pub tags: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

**Changes for Tauri version:** Only `NaiveDateTime` → `DateTime<Utc>` on the two timestamp fields.

---

#### Analog: `gtk_ref/src/domain/generation.rs`

**Excerpt — gtk_ref GenerationPreset (to port with minimal changes):**

```gtk_ref/src/domain/generation.rs#L7-25
pub struct GenerationPreset {
    pub id: String,
    pub name: String,
    /// True for built-in presets that cannot be edited or deleted (D-14).
    pub is_builtin: bool,
    pub temperature: f64,
    pub top_p: f64,
    /// 0 = no limit
    pub max_tokens: i64,
    pub frequency_penalty: f64,
    pub presence_penalty: f64,
    #[serde(default)]
    pub stop_sequences: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}
```

**Excerpt — built-in preset constructors (to port verbatim):**

```gtk_ref/src/domain/generation.rs#L37-97
impl GenerationPreset {
    pub fn creative() -> Self { /* id: "builtin-creative", temp: 1.2 */ }
    pub fn balanced() -> Self { /* id: "builtin-balanced", temp: 0.8 */ }
    pub fn precise() -> Self { /* id: "builtin-precise", temp: 0.3 */ }
}
```

**Changes for Tauri version:** Only `NaiveDateTime` → `DateTime<Utc>`.

---

### 2. DB Worker: `src-tauri/src/db/mod.rs`

#### Analog: `gtk_ref/src/db/mod.rs`

The gtk_ref uses a Relm4 `Worker` trait with `ComponentSender`. Tauri doesn't have this — we adapt to a raw `std::thread::spawn` + `mpsc` channel pattern.

**Excerpt — gtk_ref DbInput enum (adapt to DbCommand with oneshot replies):**

```gtk_ref/src/db/mod.rs#L25-73
pub enum DbInput {
    LoadProviders,
    SaveProvider(ProviderConfig),
    DeleteProvider(String),
    LoadAppSetting(String),
    SaveAppSetting { key: String, value: String },
    LoadCharacters,
    SaveCharacter(CharacterCardV2),
    DeleteCharacter(String),
    SearchCharacters(String),
    LoadPersonas,
    SavePersona(Persona),
    DeletePersona(String),
    SetGlobalDefaultPersona(Option<String>),
    StartConversation { character_id: String, provider_id: String, creator_persona_id: Option<String> },
    LoadConversations,
    LoadMessages { conversation_id: String, branch_id: Option<String> },
    SaveMessage(Message),
    // ... more variants
}
```

**Excerpt — gtk_ref DbService::init (adapt to std::thread::spawn):**

```gtk_ref/src/db/mod.rs#L147-157
fn init(db_path: Self::Init, _sender: ComponentSender<Self>) -> Self {
    let mut conn = Connection::open(&db_path)
        .unwrap_or_else(|e| panic!("Failed to open database at {:?}: {}", db_path, e));
    migrations::run_migrations(&mut conn)
        .unwrap_or_else(|e| panic!("Failed to run migrations: {}", e));
    generation::seed_builtin_presets(&conn)
        .unwrap_or_else(|e| panic!("Failed to seed built-in presets: {}", e));
    tracing::info!("Database initialized at {:?}", db_path);
    DbService { conn, db_path: db_path.clone() }
}
```

**Excerpt — gtk_ref DbService::update (adapt to match on DbCommand with oneshot replies):**

```gtk_ref/src/db/mod.rs#L159-428
fn update(&mut self, msg: DbInput, sender: ComponentSender<Self>) {
    let result = match msg {
        DbInput::LoadCharacters => {
            characters::load_characters(&self.conn)
                .map(DbOutput::CharactersLoaded)
        }
        DbInput::SaveCharacter(c) => {
            let existing = characters::load_character(&self.conn, &c.id);
            match existing {
                Ok(Some(_)) => {
                    characters::update_character(&self.conn, &c)
                        .map(|_| DbOutput::CharacterSaved(c))
                }
                Ok(None) => {
                    characters::insert_character(&self.conn, &c)
                        .map(|_| DbOutput::CharacterSaved(c))
                }
                Err(e) => Err(e),
            }
        }
        // ... more matches
    };
}
```

**Tauri adaptation pattern — DbCommand with oneshot replies:**

Instead of gtk_ref's `DbInput`/`DbOutput` pair with Relm4 sender, the Tauri version embeds a `oneshot::Sender` in each `DbCommand` variant. This eliminates the `DbOutput` enum entirely — each command's result type is determined by the oneshot channel.

```rust
// Phase 2 DbCommand (replaces DbInput + DbOutput)
pub enum DbCommand {
    LoadCharacters { reply: oneshot::Sender<Result<Vec<CharacterCardV2>, String>> },
    SaveCharacter { character: CharacterCardV2, reply: oneshot::Sender<Result<CharacterCardV2, String>> },
    DeleteCharacter { id: String, reply: oneshot::Sender<Result<String, String>> },
    SearchCharacters { query: String, reply: oneshot::Sender<Result<Vec<CharacterCardV2>, String>> },
    LoadPersonas { reply: oneshot::Sender<Result<Vec<Persona>, String>> },
    SavePersona { persona: Persona, reply: oneshot::Sender<Result<Persona, String>> },
    DeletePersona { id: String, reply: oneshot::Sender<Result<String, String>> },
    LoadConversations { reply: oneshot::Sender<Result<Vec<Conversation>, String>> },
    StartConversation { character_id: String, reply: oneshot::Sender<Result<Conversation, String>> },
    DeleteConversation { id: String, reply: oneshot::Sender<Result<String, String>> },
    LoadMessages { conversation_id: String, reply: oneshot::Sender<Result<Vec<Message>, String>> },
    SaveMessage { message: Message, reply: oneshot::Sender<Result<(), String>> },
    LoadPresets { reply: oneshot::Sender<Result<Vec<GenerationPreset>, String>> },
}
```

**Tauri DbWorker struct and start:**

```rust
pub struct DbWorker {
    sender: mpsc::Sender<DbCommand>,
}

impl DbWorker {
    pub fn start(db_path: PathBuf) -> Result<Self, String> {
        let (sender, receiver) = mpsc::channel::<DbCommand>();

        std::thread::spawn(move || {
            // Create parent directory
            if let Some(parent) = std::path::Path::new(&db_path).parent() {
                std::fs::create_dir_all(parent).expect("Failed to create data directory");
            }

            let mut conn = Connection::open(&db_path)
                .unwrap_or_else(|e| panic!("DB open failed: {}", e));
            migrations::run_migrations(&mut conn)
                .unwrap_or_else(|e| panic!("Migrations failed: {}", e));
            generation::seed_builtin_presets(&conn)
                .unwrap_or_else(|e| panic!("Preset seed failed: {}", e));

            while let Ok(cmd) = receiver.recv() {
                match cmd {
                    DbCommand::LoadCharacters { reply } => {
                        let _ = reply.send(
                            characters::load_characters(&conn).map_err(|e| e.to_string())
                        );
                    }
                    DbCommand::SaveCharacter { character, reply } => {
                        let result = match characters::load_character(&conn, &character.id) {
                            Ok(Some(_)) => characters::update_character(&conn, &character),
                            Ok(None) => characters::insert_character(&conn, &character),
                            Err(e) => Err(e),
                        }.map(|_| character).map_err(|e| e.to_string());
                        let _ = reply.send(result);
                    }
                    // ... other matches
                }
            }
        });

        Ok(DbWorker { sender })
    }

    pub fn send(&self, cmd: DbCommand) -> Result<(), String> {
        self.sender.send(cmd).map_err(|e| format!("DB worker channel error: {}", e))
    }
}
```

**Key differences from gtk_ref:**
- `DbOutput` enum eliminated — reply type is encoded in the oneshot channel
- `tokio::sync::oneshot` for replies (async-compatible with Tauri's tokio runtime)
- `std::sync::mpsc` for commands (blocking recv on the worker thread)
- `anyhow::Result` in CRUD functions → converted to `Result<T, String>` at the oneshot boundary
- `DbWorker` is `Send + Sync` (only contains `mpsc::Sender`, which is `Send + Sync`)
- Worker thread owns `Connection` exclusively — no `Arc<Mutex>`

---

### 3. Schema Migrations: `src-tauri/src/db/migrations.rs`

#### Analog: `gtk_ref/src/db/migrations.rs`

**Excerpt — gtk_ref migration set (to port with Phase 2 evolution):**

```gtk_ref/src/db/migrations.rs#L5-200
pub fn get_migrations() -> Migrations<'static> {
    Migrations::new(vec![
        // Migration 1: providers table + pragmas
        M::up("CREATE TABLE providers (...) ; PRAGMA journal_mode=WAL; ...")
          .down("DROP TABLE providers"),
        // Migration 2: app_settings
        // Migration 3: characters + FTS5 + triggers
        // Migration 4: creator_notes, character_book
        // Migration 5: conversations, message_branches, messages
        // Migration 6: model_name on providers
        // Migration 7: personas, default_persona_id
        // Migration 8: lorebook_entries, context_window_size, lorebook fields
        // Migration 9: generation_presets, regex_scripts, preset_id columns
    ])
}
```

**Phase 2 strategy (from RESEARCH):** Since no user has ever run these migrations, collapse the 9 gtk_ref migrations + Phase 2 evolution into a single clean migration set that produces the final Phase 2 schema directly. The `rusqlite_migration` crate uses `user_version` and a fresh DB starts at 0.

**Phase 2 final schema (one migration, or a small clean set):**

```sql
-- Pragmas
PRAGMA journal_mode=WAL;
PRAGMA busy_timeout=5000;
PRAGMA foreign_keys=ON;

-- providers (exists for Phase 3 FK target, nullable in conversations)
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

-- characters + FTS5
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
-- + INSERT, DELETE, UPDATE triggers (verbatim from gtk_ref M3)

-- conversations (nullable provider_id, added title + last_message_preview)
CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE,
    provider_id TEXT REFERENCES providers(id) ON DELETE SET NULL,  -- NULLABLE
    creator_persona_id TEXT,
    generation_preset_id TEXT,
    title TEXT NOT NULL DEFAULT '',
    last_message_preview TEXT NOT NULL DEFAULT '',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
CREATE INDEX idx_conversations_updated ON conversations(updated_at DESC);

-- messages (flattened — no branch_id, no message_branches table)
CREATE TABLE messages (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    role TEXT NOT NULL CHECK(role IN ('user','assistant','system')),
    content TEXT NOT NULL DEFAULT '',
    sort_order INTEGER NOT NULL DEFAULT 0,
    timestamp TEXT NOT NULL
);
CREATE INDEX idx_messages_conv_order ON messages(conversation_id, sort_order);

-- personas
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

-- generation_presets
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

-- lorebook_entries (unused until Phase 9, but schema exists)
CREATE TABLE lorebook_entries ( ... ); -- verbatim from gtk_ref M8

-- regex_scripts (unused until Phase 9, but schema exists)
CREATE TABLE regex_scripts ( ... ); -- verbatim from gtk_ref M9
```

**Key differences from gtk_ref:**
- `conversations.provider_id` is nullable (no `NOT NULL`) — allows conversations without providers in Phase 2
- `conversations` has `title` and `last_message_preview` columns (D-13)
- `messages` has no `branch_id` column (D-03)
- `message_branches` table is NOT created (D-03, deferred to Phase 8)
- Index `idx_messages_conv_order` replaces `idx_messages_conv_branch`

---

### 4. Row Mappers: `src-tauri/src/db/models.rs`

#### Analog: `gtk_ref/src/db/models.rs`

**Excerpt — gtk_ref row_to_character (to port with timestamp change):**

```gtk_ref/src/db/models.rs#L39-98
pub fn row_to_character(row: &Row) -> rusqlite::Result<CharacterCardV2> {
    let created_at_str: String = row.get("created_at")?;
    let updated_at_str: String = row.get("updated_at")?;

    let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0, rusqlite::types::Type::Text, Box::new(e),
        ))?;
    // ...

    let alt_greetings_str: String = row.get("alternate_greetings")?;
    let alternate_greetings: Vec<String> = serde_json::from_str(&alt_greetings_str)
        .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
            0, rusqlite::types::Type::Text, Box::new(e),
        ))?;
    // ... same for tags, extensions, character_book

    Ok(CharacterCardV2 { /* all fields */ })
}
```

**Change for Tauri version:** Replace `NaiveDateTime::parse_from_str` with `DateTime<Utc>` parsing:

```rust
// Before (gtk_ref):
let created_at = NaiveDateTime::parse_from_str(&created_at_str, "%Y-%m-%d %H:%M:%S%.f")
    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(...))?;

// After (src-tauri):
let created_at: DateTime<Utc> = created_at_str.parse()
    .map_err(|e| rusqlite::Error::FromSqlConversionFailure(
        0, rusqlite::types::Type::Text, Box::new(e),
    ))?;
```

`DateTime<Utc>` implements `FromStr` for ISO 8601 strings, so `.parse()` works directly. The same pattern applies to all timestamp fields in all row mappers.

**New mapper needed:** `row_to_conversation` must now extract `title` and `last_message_preview` fields (these don't exist in the gtk_ref version).

---

### 5. CRUD Functions: `src-tauri/src/db/characters.rs`, `personas.rs`, `conversations.rs`, `generation.rs`

#### Analog: `gtk_ref/src/db/characters.rs`

**Excerpt — gtk_ref insert_character (to port with timestamp change):**

```gtk_ref/src/db/characters.rs#L8-41
pub fn insert_character(conn: &Connection, character: &CharacterCardV2) -> Result<()> {
    let alternate_greetings_str = serde_json::to_string(&character.alternate_greetings)?;
    let tags_str = serde_json::to_string(&character.tags)?;
    let extensions_str = serde_json::to_string(&character.extensions)?;

    conn.prepare_cached(
        "INSERT INTO characters (id, name, description, personality, scenario, first_mes,
         mes_example, system_prompt, post_history_instructions, alternate_greetings, tags,
         creator, character_version, avatar_path, extensions, creator_notes, character_book,
         default_persona_id, lorebook_scan_depth, lorebook_recursive_depth, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19, ?20, ?21, ?22)"
    )?
    .execute(params![
        character.id, character.name, character.description,
        // ... all fields
        character.created_at.to_string(),
        character.updated_at.to_string(),
    ])?;
    Ok(())
}
```

**Change for Tauri version:** Replace `character.created_at.to_string()` with `character.created_at.to_rfc3339()`.

The same pattern applies to `update_character`, `load_characters`, `load_character`, `delete_character`, and `search_characters`. Port verbatim with the timestamp serialization change and the SQL column additions (`default_preset_id` was already in gtk_ref M9).

#### Analog: `gtk_ref/src/db/personas.rs`

Port verbatim with timestamp change (`to_rfc3339()` / `.parse()`).

#### Analog: `gtk_ref/src/db/conversations.rs`

**Excerpt — gtk_ref start_conversation (to port with modifications):**

```gtk_ref/src/db/conversations.rs#L9-45
pub fn start_conversation(
    conn: &Connection,
    character: &CharacterCardV2,
    provider_id: &str,
    creator_persona_id: Option<&str>,
) -> Result<Conversation> {
    let mut conversation = Conversation::new(character.id.clone(), provider_id.to_string());
    conversation.creator_persona_id = creator_persona_id.map(|s| s.to_string());

    conn.prepare_cached(
        "INSERT INTO conversations (id, character_id, provider_id, creator_persona_id, generation_preset_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )?
    .execute(params![
        &conversation.id, &conversation.character_id, &conversation.provider_id,
        &conversation.creator_persona_id, &conversation.generation_preset_id,
        conversation.created_at.to_string(), conversation.updated_at.to_string(),
    ])?;

    if !character.first_mes.is_empty() {
        let initial_message = Message::new(
            conversation.id.clone(), None,
            MessageRole::Assistant, character.first_mes.clone(),
        );
        save_message(conn, &initial_message)?;
    }

    Ok(conversation)
}
```

**Changes for Tauri version:**
- `provider_id: &str` → `provider_id: Option<&str>` (nullable)
- Set `conversation.title = format!("Chat with {}", character.name)` (D-13)
- Set `conversation.last_message_preview = character.first_mes.clone()` (D-13)
- INSERT SQL must include `title` and `last_message_preview` columns
- Timestamps use `.to_rfc3339()`

**Excerpt — gtk_ref save_message (to port with denormalization update):**

```gtk_ref/src/db/conversations.rs#L108-131
pub fn save_message(conn: &Connection, msg: &Message) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO messages (id, conversation_id, branch_id, role, content, timestamp, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )?
    .execute(params![
        &msg.id, &msg.conversation_id, &msg.branch_id,
        msg.role.to_string(), &msg.content,
        msg.timestamp.to_string(), msg.sort_order,
    ])?;

    let now = chrono::Local::now().naive_local();
    conn.prepare_cached("UPDATE conversations SET updated_at = ?1 WHERE id = ?2")?
        .execute(params![now.to_string(), &msg.conversation_id])?;

    Ok(())
}
```

**Changes for Tauri version:**
- Remove `branch_id` from INSERT (D-03)
- Also update `last_message_preview` on the conversation (D-13):
  ```rust
  let preview: String = msg.content.chars().take(100).collect();
  conn.prepare_cached(
      "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3"
  )?.execute(params![now.to_rfc3339(), &preview, &msg.conversation_id])?;
  ```
- Timestamps use `.to_rfc3339()`

**Excerpt — gtk_ref load_messages (to port without branch_id):**

```gtk_ref/src/db/conversations.rs#L76-104
pub fn load_messages(
    conn: &Connection,
    conversation_id: &str,
    branch_id: Option<&str>,
) -> Result<Vec<Message>> {
    // ... branch-aware query
}
```

**Change:** Simplify to just `conversation_id` parameter (no branch_id). Query becomes:
```sql
SELECT id, conversation_id, role, content, sort_order, timestamp
FROM messages WHERE conversation_id = ?1 ORDER BY sort_order ASC
```

#### Analog: `gtk_ref/src/db/generation.rs`

Port verbatim with timestamp change. The `seed_builtin_presets`, `insert_preset`, `load_presets`, `update_preset`, `delete_preset` functions all follow the same pattern.

**Excerpt — gtk_ref seed_builtin_presets (to port verbatim with timestamp change):**

```gtk_ref/src/db/generation.rs#L7-32
pub fn seed_builtin_presets(conn: &Connection) -> Result<()> {
    let presets = [
        GenerationPreset::creative(),
        GenerationPreset::balanced(),
        GenerationPreset::precise(),
    ];
    for preset in &presets {
        let stop_sequences_str = serde_json::to_string(&preset.stop_sequences)?;
        conn.prepare_cached(
            "INSERT OR IGNORE INTO generation_presets
             (id, name, is_builtin, temperature, top_p, max_tokens, frequency_penalty,
              presence_penalty, stop_sequences, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)"
        )?
        .execute(params![
            preset.id, preset.name, preset.is_builtin as i32,
            preset.temperature, preset.top_p, preset.max_tokens,
            preset.frequency_penalty, preset.presence_penalty,
            stop_sequences_str, preset.created_at.to_string(), preset.updated_at.to_string(),
        ])?;
    }
    Ok(())
}
```

**Change:** `preset.created_at.to_string()` → `preset.created_at.to_rfc3339()`.

---

### 6. Tauri Command Handlers: `src-tauri/src/commands/`

No existing analog in the codebase — this is a new pattern for the Tauri app. The RESEARCH.md provides the canonical pattern.

**Pattern — single command handler:**

```rust
// src-tauri/src/commands/characters.rs

use tauri::State;
use crate::db::{DbCommand, DbWorker};
use crate::domain::CharacterCardV2;
use tokio::sync::oneshot;

#[tauri::command]
pub async fn get_characters(
    db: State<'_, DbWorker>,
) -> Result<Vec<CharacterCardV2>, String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::LoadCharacters { reply: reply_tx })?;
    reply_rx.await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_character(
    db: State<'_, DbWorker>,
    character: CharacterCardV2,
) -> Result<CharacterCardV2, String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::SaveCharacter { character, reply: reply_tx })?;
    reply_rx.await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_character(
    db: State<'_, DbWorker>,
    id: String,
) -> Result<String, String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::DeleteCharacter { id, reply: reply_tx })?;
    reply_rx.await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_characters(
    db: State<'_, DbWorker>,
    query: String,
) -> Result<Vec<CharacterCardV2>, String> {
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::SearchCharacters { query, reply: reply_tx })?;
    reply_rx.await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
```

**Handler modules to create:**
- `commands/characters.rs` — `get_characters`, `save_character`, `delete_character`, `search_characters`
- `commands/personas.rs` — `get_personas`, `save_persona`, `delete_persona`
- `commands/conversations.rs` — `get_conversations`, `start_conversation`, `delete_conversation`, `get_messages`, `save_message`
- `commands/presets.rs` — `get_presets`

All follow the identical pattern: get `DbWorker` from state, create oneshot, send command, await result.

---

### 7. App Wiring: `src-tauri/Cargo.toml` + `src-tauri/src/lib.rs`

#### Current Cargo.toml:

```src-tauri/Cargo.toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-fs = "2"
tauri-plugin-dialog = "2"
tauri-plugin-shell = "2"
tauri-plugin-http = "2"
tauri-plugin-store = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

**Additions needed:**
```toml
rusqlite = { version = "0.31", features = ["bundled"] }
rusqlite_migration = "1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4"] }
thiserror = "1"
dirs = "5"
tokio = { version = "1", features = ["sync"] }
anyhow = "1"
```

#### Current lib.rs:

```src-tauri/src/lib.rs
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_http::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

**Modified lib.rs:**

```rust
mod db;
mod domain;
mod commands;

use db::DbWorker;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
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
            commands::start_conversation,
            commands::delete_conversation,
            commands::get_messages,
            commands::save_message,
            commands::get_presets,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

---

### 8. Frontend State: `src/lib/stores/app.svelte.ts`

#### Current AppState:

```src/lib/stores/app.svelte.ts
class AppState {
    characters = $state<CharacterCard[]>([]);
    personas = $state<UserPersona[]>([]);
    conversations = $state<Conversation[]>([]);
    providers = $state<Provider[]>([]);
    prompt_presets = $state<PromptPreset[]>([]);
    active_conversation_id = $state<string | null>(null);
    messages = $state<Message[]>([]);
    is_generating = $state<boolean>(false);
    sidebar_open = $state<boolean>(true);
    character_search_query = $state<string>('');

    hydrate(data: {
        characters: CharacterCard[];
        personas: UserPersona[];
        conversations: Conversation[];
        providers: Provider[];
        prompt_presets: PromptPreset[];
    }) {
        this.characters = data.characters;
        this.personas = data.personas;
        this.conversations = data.conversations;
        this.providers = data.providers;
        this.prompt_presets = data.prompt_presets;
    }
    // ... rest of class
}
```

**Changes:**
1. Add `is_loading` and `init_error` states (D-18)
2. `hydrate()` data source changes from mock to invoke (D-17)
3. Keep the `hydrate()` method signature — it will be called with real data from `+layout.svelte`

```typescript
class AppState {
    // ... existing fields ...
    is_loading = $state<boolean>(true);
    init_error = $state<string | null>(null);

    // hydrate() method unchanged — data source changes in +layout.svelte
}
```

---

### 9. Frontend Types: `src/lib/types/index.ts`

#### Current TypeScript types to align:

**MessageRole mismatch (D-08, Pitfall #4):**

```src/lib/types/index.ts
export type MessageRole = 'user' | 'character' | 'system';
```

**Change to:**
```typescript
export type MessageRole = 'user' | 'assistant' | 'system';
```

This aligns with Rust's `MessageRole::Assistant` which serializes as `"assistant"`. All frontend components that check `role === 'character'` must be updated to `role === 'assistant'`.

**Message type alignment (D-03):**

```src/lib/types/index.ts
export interface Message {
    id: string;
    conversation_id: string;
    parent_id: string | null;
    branch_index: number;
    role: MessageRole;
    content: string;
    token_count?: number;
    metadata?: MessageMetadata;
    created_at: string;
    is_streaming?: boolean;
}
```

**Change to:**
```typescript
export interface Message {
    id: string;
    conversation_id: string;
    role: MessageRole;
    content: string;
    sort_order: number;
    timestamp: string;
    token_count?: number;
    metadata?: MessageMetadata;
    /** UI-only: true while tokens are still streaming in */
    is_streaming?: boolean;
}
```

Key changes: `parent_id` + `branch_index` → `sort_order` + `timestamp`. The `created_at` → `timestamp` rename matches the Rust/DB column name.

**Conversation type alignment (D-13):**

```src/lib/types/index.ts
export interface Conversation {
    id: string;
    character_id: string;
    user_persona_id?: string;
    title: string;
    created_at: string;
    updated_at: string;
    active_branch_tip?: string;
    last_message_preview?: string;
    message_count?: number;
}
```

**Change to:**
```typescript
export interface Conversation {
    id: string;
    character_id: string;
    provider_id?: string;
    creator_persona_id?: string;
    generation_preset_id?: string;
    title: string;
    last_message_preview: string;
    created_at: string;
    updated_at: string;
}
```

Key changes: `user_persona_id` → `creator_persona_id`, add `provider_id`, `generation_preset_id`, `last_message_preview` becomes required. Remove `active_branch_tip` and `message_count` (computed dynamically per D-14).

**CharacterCard — minor alignment:**

```src/lib/types/index.ts
export interface CharacterCard {
    // ... existing fields ...
    avatar_url?: string;  // → avatar_path to match Rust
    extensions?: CharacterExtensions;  // → Record<string, unknown> to match serde_json::Value
}
```

**PromptPreset — align with GenerationPreset:**

```src/lib/types/index.ts
export interface PromptPreset {
    id: string;
    name: string;
    system_prompt?: string;
    temperature: number;
    top_p: number;
    max_tokens: number;
    frequency_penalty: number;
    presence_penalty: number;
    stop_sequences?: string[];
}
```

**Change to:**
```typescript
export interface GenerationPreset {
    id: string;
    name: string;
    is_builtin: boolean;
    temperature: number;
    top_p: number;
    max_tokens: number;
    frequency_penalty: number;
    presence_penalty: number;
    stop_sequences: string[];
    created_at: string;
    updated_at: string;
}
```

Rename `PromptPreset` → `GenerationPreset`, add `is_builtin`, `created_at`, `updated_at`, make `stop_sequences` required.

---

### 10. App Shell: `src/routes/+layout.svelte`

#### Current:

```src/routes/+layout.svelte
import { mockCharacters, mockPersonas, mockConversations, mockProviders, mockPresets } from '$lib/data/mock';

onMount(() => {
    app.hydrate({
        characters: mockCharacters,
        personas: mockPersonas,
        conversations: mockConversations,
        providers: mockProviders,
        prompt_presets: mockPresets
    });
});
```

**Change to:**
```svelte
<script lang="ts">
    import { invoke } from '@tauri-apps/api/core';
    import type { CharacterCard, UserPersona, Conversation, GenerationPreset } from '$lib/types';

    onMount(async () => {
        app.is_loading = true;

        try {
            const results = await Promise.allSettled([
                invoke<CharacterCard[]>('get_characters'),
                invoke<UserPersona[]>('get_personas'),
                invoke<Conversation[]>('get_conversations'),
                invoke<GenerationPreset[]>('get_presets'),
            ]);

            const characters = results[0].status === 'fulfilled' ? results[0].value : [];
            const personas = results[1].status === 'fulfilled' ? results[1].value : [];
            const conversations = results[2].status === 'fulfilled' ? results[2].value : [];
            const presets = results[3].status === 'fulfilled' ? results[3].value : [];

            // Collect errors from rejected promises
            const errors = results
                .filter((r): r is PromiseRejectedResult => r.status === 'rejected')
                .map((r) => String(r.reason));
            if (errors.length > 0) {
                app.init_error = errors.join('; ');
            }

            app.hydrate({
                characters,
                personas,
                conversations,
                providers: [],  // Empty until Phase 3
                prompt_presets: presets,
            });
        } catch (e) {
            app.init_error = String(e);
        } finally {
            app.is_loading = false;
        }
    });
</script>

{#if app.is_loading}
    <!-- Skeleton/placeholder loading UI -->
{:else if app.init_error}
    <!-- Error display -->
{:else}
    <!-- Normal app shell -->
{/if}
```

**Key patterns:**
- `Promise.allSettled` instead of `Promise.all` (handles partial failure — Pitfall #8)
- `invoke()` with type parameters for type-safe deserialization
- Empty `providers` array (Phase 3)
- Remove all mock data imports

---

### 11. Mock Data Removal: `src/lib/data/mock.ts`

**Delete this file entirely** after all components have been rewired to use `invoke()`. Before deletion, verify:
1. `+layout.svelte` no longer imports from `mock.ts`
2. No component still references `mockCharacters`, `mockPersonas`, etc.
3. The `getMessages()` stub function in `mock.ts` is replaced by `invoke('get_messages', { conversationId })`

---

### 12. Utilities: `src/lib/utils/index.ts`

#### Current estimateTokens:

```src/lib/utils/index.ts
export function estimateTokens(text: string): number {
    if (!text) return 0;
    return Math.ceil(text.length / 4);
}
```

No changes required in Phase 2. The comment says "The real app will swap this for tiktoken-rs via the backend bridge" — that's a future phase concern. The function is still useful for UI display purposes even with real data.

---

## Cross-Cutting Patterns

### Timestamp Serialization Contract

All timestamps in the system follow this contract:

| Layer | Type | Serialization |
|-------|------|---------------|
| Rust domain | `chrono::DateTime<Utc>` | `#[serde]` → ISO 8601 `"2026-05-11T14:30:00Z"` |
| Rust → SQLite | `String` via `.to_rfc3339()` | `"2026-05-11T14:30:00Z"` |
| SQLite → Rust | `.parse::<DateTime<Utc>>()` | ISO 8601 parse |
| Tauri IPC | JSON string | `"2026-05-11T14:30:00Z"` |
| TypeScript | `string` | No parsing needed — pass through |

### JSON Column Contract

Lists and objects stored as JSON TEXT in SQLite:

| Column | Rust Type | SQLite Storage | Serialization |
|--------|-----------|----------------|---------------|
| `tags` | `Vec<String>` | `TEXT` | `serde_json::to_string()` |
| `alternate_greetings` | `Vec<String>` | `TEXT` | `serde_json::to_string()` |
| `extensions` | `serde_json::Value` | `TEXT` | `serde_json::to_string()` |
| `character_book` | `serde_json::Value` | `TEXT` | `serde_json::to_string()` |
| `stop_sequences` | `Vec<String>` | `TEXT` | `serde_json::to_string()` |

### Error Handling Contract

| Layer | Error Type | Conversion |
|-------|-----------|------------|
| CRUD functions | `anyhow::Result<T>` | `?` propagation |
| DB worker match | `Result<T, String>` | `.map_err(\|e\| e.to_string())` |
| Tauri commands | `Result<T, String>` | `?` propagation |
| Frontend invoke | Rejected Promise | `.catch()` or `allSettled` |

### Upsert Pattern

All `Save*` commands follow the upsert pattern from gtk_ref:

```rust
// In the DB worker match arm:
DbCommand::SaveCharacter { character, reply } => {
    let result = match characters::load_character(&conn, &character.id) {
        Ok(Some(_)) => characters::update_character(&conn, &character),
        Ok(None) => characters::insert_character(&conn, &character),
        Err(e) => Err(e),
    }.map(|_| character).map_err(|e| e.to_string());
    let _ = reply.send(result);
}
```

This is consistent across characters, personas, and presets.

---

## Implementation Order Guidance

The pattern dependencies suggest this implementation order:

1. **`Cargo.toml`** — Add all dependencies first (everything else depends on this)
2. **`src-tauri/src/domain/`** — Port all domain models (DB layer + commands depend on these)
3. **`src-tauri/src/db/migrations.rs`** — Define the schema (CRUD functions depend on tables existing)
4. **`src-tauri/src/db/models.rs`** — Row mappers (CRUD functions depend on these)
5. **`src-tauri/src/db/characters.rs`** — Character CRUD (simplest, test first)
6. **`src-tauri/src/db/personas.rs`** — Persona CRUD (same pattern as characters)
7. **`src-tauri/src/db/conversations.rs`** — Conversation/Message CRUD (most complex, depends on characters)
8. **`src-tauri/src/db/generation.rs`** — Preset CRUD + seeding (independent, called during init)
9. **`src-tauri/src/db/mod.rs`** — DbWorker + DbCommand (wires all CRUD modules together)
10. **`src-tauri/src/commands/`** — Tauri command handlers (thin wrappers over DbWorker)
11. **`src-tauri/src/lib.rs`** — App wiring (registers worker + commands)
12. **`src/lib/types/index.ts`** — Update TypeScript types (must match Rust serde output)
13. **`src/lib/stores/app.svelte.ts`** — Add loading state
14. **`src/routes/+layout.svelte`** — Replace mock hydration with invoke
15. **`src/lib/data/mock.ts`** — Delete after wiring verified
16. **Frontend component updates** — Fix `'character'` → `'assistant'` role checks, `parent_id` → `sort_order` references

---

*Phase: 02-Persistent Data — Pattern Mapping*
