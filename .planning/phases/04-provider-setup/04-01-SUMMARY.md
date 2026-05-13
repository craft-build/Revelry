---
phase: 04-provider-setup
plan: 01
subsystem: backend
tags: [keyring, keyring-core, provider, tauri-commands, sqlite, spawn-blocking, serde]

# Dependency graph
requires:
  - phase: 02-persistent-data
    provides: DbWorker pattern, domain models, SQLite schema, Tauri command infrastructure
  - phase: 03-editor-features
    provides: Editor dialog pattern, KebabMenu, DeleteConfirmationDialog
provides:
  - KeyringStore with save_key, load_key, delete_key, is_available (OS keychain)
  - ProviderConfig domain model with 1:N ProviderModelEntry, GenerationParams
  - Provider CRUD via DbWorker (insert, load, load single, update, delete)
  - 8 Tauri commands for provider + keyring + conversation-provider operations
  - Migration #3 adding models, default_params columns to providers and model_name to conversations
  - ProviderWithKeyStatus response type (has_api_key boolean, never key value)
affects: [04-02-provider-ui, 05-inference-streaming]

# Tech tracking
tech-stack:
  added: [keyring 4.0, keyring-core 1.0]
  patterns: [spawn_blocking for sync keyring ops, explicit serde rename per variant, batch has_api_key check, best-effort keyring cleanup on delete]

key-files:
  created: [src-tauri/src/secrets/mod.rs, src-tauri/src/domain/provider.rs, src-tauri/src/db/providers.rs, src-tauri/src/commands/providers.rs]
  modified: [src-tauri/Cargo.toml, src-tauri/src/lib.rs, src-tauri/src/domain/mod.rs, src-tauri/src/domain/conversation.rs, src-tauri/src/db/mod.rs, src-tauri/src/db/models.rs, src-tauri/src/db/migrations.rs, src-tauri/src/db/conversations.rs, src-tauri/src/commands/mod.rs]

key-decisions:
  - "Used explicit #[serde(rename)] per variant instead of rename_all = snake_case because serde splits OpenAI into open_a_i not openai"
  - "keyring v4 requires use_native_store(true) init before Entry::new and release_store() on shutdown"
  - "KeyringStore tests call ensure_keyring_init() to initialize store before each test"
  - "get_providers returns ProviderWithKeyStatus with batch has_api_key to avoid N+1 frontend calls"
  - "delete_provider does best-effort keyring cleanup after successful DB deletion"

patterns-established:
  - "Keyring v4 init pattern: use_native_store(true) before Builder, release_store() after .run()"
  - "spawn_blocking for keyring ops: all KeyringStore methods are sync, wrap in tokio::task::spawn_blocking"
  - "ProviderWithKeyStatus: flatten ProviderConfig + has_api_key boolean for frontend"
  - "Upsert pattern: SaveProvider checks if provider exists → update or insert"

requirements-completed: [PROV-01, PROV-02, PROV-03, PROV-04, PROV-05, PROV-06]

# Metrics
duration: 5min
completed: 2026-05-13
---

# Phase 4 Plan 01: Provider Backend Summary

**Keyring v4 + provider CRUD + 8 Tauri commands for OS keychain-backed provider management**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-13T02:48:11Z
- **Completed:** 2026-05-13T02:53:49Z
- **Tasks:** 3
- **Files modified:** 13

## Accomplishments
- KeyringStore module using keyring v4 API with save/load/delete/is_available, all wrapped in spawn_blocking
- Provider domain model with snake_case serde (explicit renames), 1:N ProviderModelEntry, GenerationParams
- Migration #3 adding models JSON column, default_params column, and conversation model_name
- Provider CRUD through DbWorker with upsert pattern
- 8 Tauri commands registered: get_providers, save_provider, delete_provider, save_api_key, has_api_key, delete_api_key, check_keyring_available, update_conversation_provider

## Task Commits

Each task was committed atomically:

1. **Task 1: Add keyring dependencies and create KeyringStore module** - `d7cf55b` (feat)
2. **Task 2: Provider domain model, migration #3, and row mapper** - `338702b` (feat)
3. **Task 3: Provider CRUD, Tauri commands, and module registration** - `f77512f` (feat)

## Files Created/Modified
- `src-tauri/Cargo.toml` - Added keyring 4.0, keyring-core 1.0, rt-multi-thread tokio feature
- `src-tauri/src/secrets/mod.rs` - KeyringStore with save_key, load_key, delete_key, is_available
- `src-tauri/src/lib.rs` - mod secrets, keyring init, release_store, 8 new commands registered
- `src-tauri/src/domain/provider.rs` - ProviderType, ProviderModelEntry, GenerationParams, ProviderConfig
- `src-tauri/src/domain/mod.rs` - Added pub mod provider and re-exports
- `src-tauri/src/domain/conversation.rs` - Added model_name: Option<String> field
- `src-tauri/src/db/migrations.rs` - Migration #3 + tests for models/default_params/model_name columns
- `src-tauri/src/db/models.rs` - Added row_to_provider mapper, updated row_to_conversation
- `src-tauri/src/db/conversations.rs` - Updated SQL for model_name, added update_conversation_provider
- `src-tauri/src/db/providers.rs` - insert, load, load single, update, delete provider CRUD
- `src-tauri/src/db/mod.rs` - Added providers module, DbCommand variants, handle_command arms
- `src-tauri/src/commands/providers.rs` - 8 Tauri commands with spawn_blocking for keyring
- `src-tauri/src/commands/mod.rs` - Added pub mod providers

## Decisions Made
- Used explicit `#[serde(rename = "...")]` per ProviderType variant instead of `rename_all = "snake_case"` because serde's automatic conversion splits "OpenAI" into "open_a_i" instead of the desired "openai"
- Keyring v4 requires `use_native_store(true)` initialization before any `Entry::new()` call and `release_store()` cleanup on shutdown
- KeyringStore tests call `ensure_keyring_init()` to set up the native store before each test (keyring v4 requirement)
- `get_providers` returns `ProviderWithKeyStatus` (ProviderConfig + has_api_key bool) to avoid N+1 frontend calls
- `delete_provider` performs best-effort keyring cleanup after successful DB deletion

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Replaced rename_all = "snake_case" with explicit serde renames**
- **Found during:** Task 2 (ProviderType implementation)
- **Issue:** `#[serde(rename_all = "snake_case")]` converts `OpenAI` → `"open_a_i"` instead of the expected `"openai"`. This is because serde's snake_case algorithm splits on capital boundaries, treating "AI" as a separate word.
- **Fix:** Replaced with explicit `#[serde(rename = "openai")]` on each enum variant, ensuring correct TypeScript-compatible snake_case values.
- **Files modified:** src-tauri/src/domain/provider.rs
- **Verification:** `test_provider_type_snake_case_serde` passes, all variant roundtrips work
- **Committed in:** 338702b (Task 2 commit)

**2. [Rule 1 - Bug] Added keyring store initialization in KeyringStore tests**
- **Found during:** Task 1 (KeyringStore tests)
- **Issue:** KeyringStore tests failed with "No default store has been set" because keyring v4 requires `use_native_store()` before `Entry::new()`, but tests run outside the Tauri app context where lib.rs initializes it.
- **Fix:** Added `ensure_keyring_init()` helper that calls `keyring::use_native_store(true)` at the start of each test.
- **Files modified:** src-tauri/src/secrets/mod.rs
- **Verification:** All 3 KeyringStore tests pass
- **Committed in:** d7cf55b (Task 1 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Both auto-fixes necessary for correctness. The serde rename deviation improves type alignment with TypeScript. The test init fix is required by keyring v4 architecture.

## Issues Encountered
- Pre-existing `test_row_to_character` failure (extensions Object {} vs Null) — out of scope, deferred

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Provider backend pipeline complete: keychain → domain model → database → Tauri commands
- Ready for Plan 02 (frontend: ProviderEditorDialog, settings page wiring, AppState provider CRUD actions)
- 8 Tauri commands available for frontend invoke() calls

## Self-Check: PASSED

---
*Phase: 04-provider-setup*
*Completed: 2026-05-13*
