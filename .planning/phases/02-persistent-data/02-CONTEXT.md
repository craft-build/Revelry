# Phase 2: Persistent Data - Context

**Gathered:** 2026-05-11
**Status:** Ready for planning

<domain>
## Phase Boundary

Characters, personas, and conversations load from and save to a real SQLite database instead of mock data. The app initializes SQLite on first launch, runs migrations, and all CRUD operations go through Tauri IPC commands to the Rust backend. The frontend replaces all mock data imports with `invoke()` calls. Database operations never freeze the UI.

This phase delivers: SQLite initialization + migration, domain model port, DB service with background-thread worker, Tauri command layer for character/persona/conversation CRUD, and frontend wiring to replace mock data.

NOT in scope: AI provider integration (Phase 3), streaming (Phase 4), prompt assembly (Phase 5), editors (Phase 7), branching (Phase 8), lorebook (Phase 9).

</domain>

<decisions>
## Implementation Decisions

### Schema Evolution
- **D-01:** Port the gtk_ref schema as the starting point — it's battle-tested and covers all required tables (providers, characters, personas, conversations, messages, message_branches, lorebook_entries, generation_presets, regex_scripts, app_settings). Do NOT redesign from scratch.
- **D-02:** Evolve the schema where the frontend types reveal gaps — specifically, add `title` and `last_message_preview` columns to `conversations` (the frontend `Conversation` type requires these for the sidebar). The gtk_ref schema lacked these denormalizations.
- **D-03:** Flatten the message model for Phase 2 — the frontend `Message` type uses `parent_id` + `branch_index` while gtk_ref uses `branch_id` + `sort_order`. Since branching is Phase 8, use a simplified flat model now: `messages` table with `id, conversation_id, role, content, sort_order, timestamp`. The `branch_id` and `message_branches` tables are deferred to Phase 8 (they're only needed when branching is implemented).
- **D-04:** Keep FTS5 virtual tables from gtk_ref (e.g., `characters_fts`) — they'll be needed for CONV-03 search, even though full-text search UI comes in Phase 8. Schema is simpler with them in from the start.

### DB Threading Model
- **D-05:** Use a dedicated `std::sync::mpsc` channel-based background worker for DB operations, following the gtk_ref `DbService` pattern — not a simple `Mutex<Connection>`. SQLite is inherently single-writer and the background worker ensures: (a) all DB ops are off-main-thread, (b) writes are naturally serialized, (c) the connection stays on one thread (required by rusqlite's `!Send` contract).
- **D-06:** In Tauri, implement the worker as: a `std::thread::spawn` that owns the `rusqlite::Connection` and listens on a `mpsc::Receiver<DbCommand>`. Tauri commands send `DbCommand` variants via the sender half and await results via `oneshot` channels. This is the proven pattern from gtk_ref adapted to Tauri's architecture.
- **D-07:** WAL mode + busy_timeout=5000 + foreign_keys=ON — carried forward from gtk_ref migration 1. These are essential for concurrent read access and data integrity.

### Type Alignment (Rust ↔ TypeScript)
- **D-08:** Tauri commands return `serde_json::Value` or typed structs that match the frontend TypeScript interfaces. The Rust domain models are the source of truth; the TypeScript types are updated to match what the Tauri commands actually return.
- **D-09:** Timestamps use ISO 8601 strings (`String`) in both Rust and TypeScript — the frontend already uses `string` for timestamps. The gtk_ref `NaiveDateTime` approach is replaced with `chrono::DateTime<Utc>` serialized as ISO strings, which is more portable across the IPC boundary.
- **D-10:** UUIDs as text strings (`TEXT PRIMARY KEY`) — carried forward from gtk_ref. No integer auto-increment PKs. This matches both the existing schema and the frontend type expectations.

### Initial Data Seeding
- **D-11:** Do NOT ship built-in characters or personas in the database — the app starts with an empty library. The mock data (Eleanor, KAI-7, Marisol, Cartographer + Wren, Aris Vell, Theo) served the UI scaffold only. Users will create or import their own characters.
- **D-12:** Ship built-in generation presets (Creative, Balanced, Precise) — same as gtk_ref's `seed_builtin_presets`. These are utility defaults that should exist from day one.

### Conversation List Denormalization
- **D-13:** Add `title TEXT NOT NULL DEFAULT ''` and `last_message_preview TEXT NOT NULL DEFAULT ''` columns to the `conversations` table. These are updated on every new message save. The sidebar needs them for display without joining the messages table.
- **D-14:** Compute `message_count` dynamically (via `SELECT COUNT(*)` query) rather than storing it as a denormalized column — it's only needed when the sidebar renders and is cheap to compute. Avoids stale-count bugs.

### Tauri Command Design
- **D-15:** One Tauri command per CRUD operation — e.g., `get_characters`, `save_character`, `delete_character`, `search_characters`, `get_personas`, `save_persona`, `delete_persona`, `get_conversations`, `get_messages`, `start_conversation`, `save_message`, `delete_conversation`. Follow the `DbInput`/`DbOutput` enum pattern from gtk_ref but expose flat commands to the frontend (the enum is internal routing only).
- **D-16:** Commands are async and return `Result<T, String>` — Tauri's standard error convention. The DB worker handles the actual synchronous rusqlite calls; the Tauri command awaits the oneshot result.

### Frontend Integration
- **D-17:** Replace the `hydrate()` call in `+layout.svelte` with Tauri `invoke()` calls that load real data from the backend. Keep the `AppState` class structure — just change the data source from `mock.ts` imports to `invoke()` results.
- **D-18:** Add a loading state to `AppState` — `is_loading = $state<boolean>(true)` — that's true during initial data fetch and false once data arrives. The UI shows a loading indicator during hydration. This replaces the instant mock-data hydration.
- **D-19:** Remove `src/lib/data/mock.ts` after wiring is complete — it's no longer needed once real data flows through.

### Claude's Discretion
None — all decisions were auto-selected in yolo mode (recommended defaults).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Planning
- `.planning/ROADMAP.md` — Phase 2 definition, plans list (02-01 through 02-04), success criteria
- `.planning/REQUIREMENTS.md` — DB-01, DB-02, DB-03, CHAR-01, PERS-01, PERS-05, CONV-01, CONV-02, CONV-03, CONV-04, CONV-05 requirements
- `.planning/PROJECT.md` — Project context, constraints, tech stack decisions, key decisions table

### Archival Reference (MUST read — patterns to port)
- `gtk_ref/src/db/migrations.rs` — Full SQL migration definitions (9 migrations), WAL/busy_timeout/foreign_keys pragmas, FTS5 setup
- `gtk_ref/src/db/mod.rs` — `DbInput`/`DbOutput` enum design, `DbService` worker pattern with background thread
- `gtk_ref/src/db/models.rs` — Row→domain type mapper functions (row_to_character, row_to_persona, row_to_conversation, row_to_message, etc.)
- `gtk_ref/src/db/characters.rs` — Character CRUD operations
- `gtk_ref/src/db/personas.rs` — Persona CRUD operations
- `gtk_ref/src/db/conversations.rs` — Conversation and message CRUD operations
- `gtk_ref/src/db/generation.rs` — Generation preset CRUD and seed_builtin_presets
- `gtk_ref/src/domain/character.rs` — CharacterCardV2 domain model, CharaCardV2Spec import format
- `gtk_ref/src/domain/conversation.rs` — Conversation, Message, MessageBranch, MessageRole domain models
- `gtk_ref/src/domain/persona.rs` — Persona domain model

### Active Frontend (current state — will be modified)
- `src/lib/types/index.ts` — TypeScript type definitions (will need updates to match Tauri command return types)
- `src/lib/data/mock.ts` — Mock data (will be removed after wiring)
- `src/lib/stores/app.svelte.ts` — AppState (will change hydration from mock to invoke)
- `src/routes/+layout.svelte` — App shell (will add loading state, invoke-based hydration)

### Tauri Backend (current state — will be extended)
- `src-tauri/src/lib.rs` — Tauri plugin registration (will add DB worker initialization + commands)
- `src-tauri/Cargo.toml` — Current minimal dependencies (will add rusqlite, chrono, uuid, etc.)

### Architecture Documentation
- `.planning/codebase/ARCHITECTURE.md` — System overview, layers, data flow, integration points
- `.planning/codebase/STRUCTURE.md` — Directory layout, where to add new Rust modules

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **gtk_ref DB layer**: Complete, working SQLite implementation with 9 migrations, CRUD for all entities, background-thread worker. Direct porting target — don't redesign, just adapt to Tauri's architecture.
- **gtk_ref domain models**: Rust structs with serde derives, validation, constructors. Port to `src-tauri/src/domain/`.
- **Frontend TypeScript types**: Close to the Rust models but with structural differences (title/preview on Conversation, parent_id/branch_index on Message vs branch_id/sort_order). Will be aligned during implementation.
- **AppState**: Clean reactive store with `hydrate()` method — minimal change needed, just swap the data source.

### Established Patterns
- **UUID text PKs**: Both gtk_ref and the frontend use string UUIDs. No integer auto-increment.
- **JSON in TEXT columns**: Lists (tags, alternate_greetings, keywords) and complex objects (extensions, character_book) stored as JSON TEXT in SQLite. Serde deserialization in row mappers. Same pattern should continue.
- **WAL mode**: Required for concurrent reads without blocking writes. Already proven in gtk_ref.
- **Tauri IPC via invoke()**: Phase 1 established the pattern. Commands registered in `lib.rs`, called from Svelte via `@tauri-apps/api/core`.

### Integration Points
- **`src-tauri/src/lib.rs`**: Add DB worker initialization before `.run()`, register command handlers via `.invoke_handler()`
- **`src-tauri/Cargo.toml`**: Add `rusqlite` (with `bundled` feature), `chrono`, `uuid`, `thiserror`, `rusqlite_migration`, `dirs`
- **`src/lib/stores/app.svelte.ts`**: Change `hydrate()` to call `invoke()` commands instead of accepting mock data
- **`src/routes/+layout.svelte`**: Add `onMount` data loading with loading indicator
- **New `src-tauri/src/domain/`**: Port domain models from gtk_ref
- **New `src-tauri/src/db/`**: Port DB service, migrations, CRUD modules
- **New `src-tauri/src/commands/`**: Tauri IPC command handlers

</code_context>

<specifics>
## Specific Ideas

- Database file location: Use `dirs::data_dir()` for platform-appropriate path (`~/Library/Application Support/` on macOS, `~/.local/share/` on Linux, `%APPDATA%` on Windows)
- Database filename: `revelry.db`
- Migration strategy: `rusqlite_migration` crate (same as gtk_ref) — auto-runs on startup
- Loading indicator: Skeleton/placeholder UI while initial data loads (not a spinner — content-first philosophy)
- Error handling on DB init failure: App should show an error dialog and quit gracefully, not silently fail

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 02-Persistent Data*
*Context gathered: 2026-05-11*
