---
phase: 02-persistent-data
plan: phase-summary
subsystem: database
tags: [sqlite, rusqlite, tauri-ipc, svelte, serde, datetime-utc, db-worker]

requires:
  - phase: 01-native-desktop-shell
    provides: Tauri v2 app shell with SvelteKit frontend, plugin registration
provides:
  - SQLite database initialization with WAL mode, migrations, and seed data
  - Domain models: CharacterCardV2, Conversation, Message, Persona, GenerationPreset
  - Background-thread DbWorker with mpsc channel pattern
  - Full CRUD for characters, personas, conversations, messages, presets
  - 16 Tauri IPC commands registered in invoke_handler
  - Typed TypeScript API layer with invoke() wrappers
  - Frontend loading/error states with skeleton UI
  - TypeScript types aligned with Rust serde output
affects:
  [
    03-provider-setup,
    04-live-conversations,
    05-smart-prompts,
    06-rich-output,
    07-editors,
    08-advanced-conversations,
    09-power-features,
  ]

tech-stack:
  added:
    [
      rusqlite 0.31 bundled,
      rusqlite-migration 1.0,
      chrono 0.4,
      uuid 1,
      thiserror 2,
      dirs 6,
      anyhow 1,
      tokio 1 sync,
      tracing 0.1,
    ]
  patterns:
    [
      background-thread DB worker,
      mpsc channel DbCommand pattern,
      oneshot reply channels,
      Tauri State for DbWorker,
      snake_case Tauri v2 command names,
      DateTime<Utc> with rfc3339 serialization,
      FTS5 with sanitize_fts_query,
      Promise.allSettled for partial failure,
    ]

key-files:
  created:
    - src-tauri/src/domain/mod.rs
    - src-tauri/src/domain/character.rs
    - src-tauri/src/domain/conversation.rs
    - src-tauri/src/domain/persona.rs
    - src-tauri/src/domain/generation.rs
    - src-tauri/src/db/mod.rs
    - src-tauri/src/db/migrations.rs
    - src-tauri/src/db/models.rs
    - src-tauri/src/db/characters.rs
    - src-tauri/src/db/personas.rs
    - src-tauri/src/db/conversations.rs
    - src-tauri/src/db/generation.rs
    - src-tauri/src/commands/mod.rs
    - src-tauri/src/commands/characters.rs
    - src-tauri/src/commands/personas.rs
    - src-tauri/src/commands/conversations.rs
    - src-tauri/src/commands/presets.rs
    - src/lib/data/api.ts
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs
    - src/lib/types/index.ts
    - src/lib/stores/app.svelte.ts
    - src/routes/+layout.svelte
    - src/routes/chat/[id]/+page.svelte
    - src/lib/components/chat/MessageBubble.svelte
    - src/lib/components/chat/ChatHeader.svelte
    - src/lib/components/chat/PersonaSwitcher.svelte
    - src/lib/components/layout/ConversationSidebar.svelte
    - src/lib/components/characters/CharacterCard.svelte
    - src/lib/components/personas/PersonaCard.svelte
    - src/routes/+page.svelte
  deleted:
    - src/lib/data/mock.ts

key-decisions:
  - "DateTime<Utc> with .to_rfc3339() serialization instead of NaiveDateTime — more portable across IPC"
  - "Single collapsed migration producing complete Phase 2 schema — no data to preserve, cleaner than porting 9 sequential migrations"
  - "PRAGMAs moved out of migration SQL into configure_connection() — WAL mode cannot run inside transaction (rusqlite_migration wraps in transaction)"
  - "conversations.provider_id nullable — no providers until Phase 3"
  - "messages table has no branch_id — deferred to Phase 8"
  - "MessageRole Assistant (not Character) — aligned with rig-core convention"
  - "avatar_url → avatar_path — matches Rust field name from gtk_ref"
  - "PromptPreset → GenerationPreset — matches Rust domain model, backward compat alias kept"
  - "Promise.allSettled for frontend hydration — partial failure resilience"

patterns-established:
  - "Background-thread DB Worker: std::sync::mpsc channel + std::thread::spawn, DbCommand enum with oneshot reply channels"
  - "Tauri command pattern: #[tauri::command] async fn, State<'_, DbWorker>, create-oneshot/send/await-reply"
  - "Row mapper pattern: pub fn row_to_X(row: &Row) -> rusqlite::Result<T> with DateTime<Utc> parse and JSON deserialization"
  - "FTS5 pattern: sanitize_fts_query() escapes double quotes and wraps in phrase, prevents operator injection"
  - "Type alignment: Rust serde output is source of truth, TypeScript types match exactly"
  - "Frontend API layer: typed invoke() wrappers in src/lib/data/api.ts, snake_case command names"

requirements-completed:
  [
    DB-01,
    DB-02,
    DB-03,
    CHAR-01,
    PERS-01,
    PERS-05,
    CONV-01,
    CONV-02,
    CONV-03,
    CONV-04,
    CONV-05,
  ]

duration: ~80min
completed: 2026-05-11
---

# Phase 2: Persistent Data Summary

**SQLite persistence with background-thread worker, 16 Tauri IPC commands, and frontend invoke() wiring replacing all mock data**

## Performance

- **Duration:** ~80 min (4 plans × ~20 min each)
- **Started:** 2026-05-11T03:00:00Z
- **Completed:** 2026-05-11T03:55:00Z
- **Tasks:** 23 tasks across 4 plans
- **Files modified:** ~25 created/modified, 1 deleted

## Accomplishments

- Complete SQLite persistence layer ported from gtk_ref with DateTime<Utc> timestamps
- 16 Tauri IPC commands enabling full CRUD for characters, personas, conversations, messages, presets
- Frontend wired to real backend — mock.ts deleted, all data flows through invoke()
- Loading skeleton and error overlay for robust UX during data fetch
- 80 Rust unit tests covering domain models, migrations, CRUD, and DbWorker integration

## Plan Summaries

### Plan 02-01: Domain models + schema + DbWorker

- 5 domain models ported with DateTime<Utc>
- Single collapsed migration producing Phase 2 schema
- DbWorker with mpsc channel background-thread pattern
- 42 tests passing

### Plan 02-02: CRUD functions + row mappers

- 5 row mapper functions (character, persona, conversation, message, generation_preset)
- Full CRUD for characters (FTS5 search), personas (LIKE search), conversations (with denormalization), messages, presets
- 80 tests passing

### Plan 02-03: Tauri IPC commands

- 16 command handlers in 4 module files
- DbWorker initialization in lib.rs with dirs::data_dir() path resolution
- App exits loudly on DB init failure
- PRAGMAs moved out of migration (WAL can't run in transaction)

### Plan 02-04: Frontend wiring

- TypeScript types aligned with Rust serde output
- Typed API layer (api.ts) with 16 invoke() wrappers
- AppState with is_loading/init_error, async switchPersona with optimistic update
- Skeleton loading UI and DB error overlay
- mock.ts deleted — no mock data remains

## Decisions Made

- DateTime<Utc> with .to_rfc3339() instead of NaiveDateTime — better IPC portability
- Single collapsed migration — no existing data to preserve
- PRAGMAs moved to configure_connection() — WAL can't run inside rusqlite_migration transaction
- conversations.provider_id nullable — Phase 3 adds providers
- messages table has no branch_id — deferred to Phase 8
- MessageRole uses 'assistant' not 'character' — rig-core convention

## Deviations from Plan

### Auto-fixed Issues

**1. PRAGMA placement — WAL mode cannot run inside transaction**

- **Found during:** Plan 02-03 (lib.rs wiring)
- **Issue:** rusqlite_migration wraps migrations in a transaction; PRAGMA journal_mode=WAL fails inside a transaction
- **Fix:** Moved PRAGMAs out of migration SQL into a `configure_connection()` function that runs before migrations
- **Files modified:** src-tauri/src/db/migrations.rs, src-tauri/src/db/mod.rs
- **Verification:** cargo test --lib passes, WAL mode confirmed set

**2. DbCommand fire-and-forget → request/response pattern**

- **Found during:** Plan 02-03 (command handler implementation)
- **Issue:** Original DbCommand variants lacked reply channels — command handlers had no way to receive results
- **Fix:** Added `reply: oneshot::Sender<anyhow::Result<T>>` to every DbCommand variant
- **Files modified:** src-tauri/src/db/mod.rs, all command files
- **Verification:** All 16 commands work end-to-end

---

**Total deviations:** 2 auto-fixed
**Impact on plan:** Both necessary for correctness. No scope creep.

## Issues Encountered

None beyond the auto-fixed deviations above.

## User Setup Required

None — database is created automatically on first launch.

## Next Phase Readiness

- SQLite persistence layer complete and tested
- All 16 Tauri IPC commands available for Phase 3 (provider CRUD will follow same pattern)
- Frontend API layer established — Phase 3 commands will add provider wrappers to api.ts
- TypeScript type alignment pattern established for future phases
- Phase 3 (Provider Setup) can proceed immediately

---

_Phase: 02-persistent-data_
_Completed: 2026-05-11_
