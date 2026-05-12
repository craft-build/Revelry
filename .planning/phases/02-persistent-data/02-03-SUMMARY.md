# Plan 02-03: Tauri IPC Commands & DbWorker Integration — COMPLETE

## Summary

Created 16 Tauri IPC command handlers that expose CRUD operations to the frontend, wired them into the app's `invoke_handler`, and initialized the DbWorker in `lib.rs` with proper database path resolution and error handling.

## Changes Made

### New Files

- **`src-tauri/src/commands/mod.rs`** — Module declarations for all 4 command submodules
- **`src-tauri/src/commands/characters.rs`** — 4 commands: `get_characters`, `save_character`, `delete_character`, `search_characters`
- **`src-tauri/src/commands/personas.rs`** — 4 commands: `get_personas`, `save_persona`, `delete_persona`, `search_personas`
- **`src-tauri/src/commands/conversations.rs`** — 7 commands: `get_conversations`, `start_conversation`, `delete_conversation`, `search_conversations`, `get_messages`, `save_message`, `update_conversation_persona`
- **`src-tauri/src/commands/presets.rs`** — 1 command: `get_presets`

### Modified Files

- **`src-tauri/src/db/mod.rs`** — Major refactor of DbCommand enum and DbWorker:
  - Added `reply: oneshot::Sender<anyhow::Result<T>>` to every `DbCommand` variant, converting from fire-and-forget to request/response pattern
  - Updated `handle_command()` to send results back through the reply channels instead of logging and discarding
  - Updated `handle_command()` return type from `Result<(), String>` to `()` (errors now sent via reply channel)
  - Updated tests to use `blocking_recv()` on the oneshot receiver for synchronous reply waiting
  - Added `Conversation` and `GenerationPreset` imports

- **`src-tauri/src/db/migrations.rs`** — Fixed WAL pragma issue:
  - Removed `PRAGMA journal_mode=WAL`, `PRAGMA busy_timeout=5000`, `PRAGMA foreign_keys=ON` from the migration SQL (they cannot run inside a transaction that `rusqlite_migration` wraps)
  - Added `configure_connection()` function to set PRAGMAs on the connection before migrations run
  - Updated `run_migrations()` to call `configure_connection()` first

- **`src-tauri/src/domain/mod.rs`** — Added re-exports for domain types:
  - `pub use character::CharacterCardV2`
  - `pub use conversation::{Conversation, Message}`
  - `pub use generation::GenerationPreset`
  - `pub use persona::Persona`

- **`src-tauri/src/lib.rs`** — Complete rewrite:
  - Added `mod commands`, `mod db`, `mod domain` declarations
  - Added DbWorker initialization with `dirs::data_dir()` → `com.revelry.desktop/revelry.db`
  - Fatal error handling with `eprintln!` + `std::process::exit(1)` if DbWorker fails to start
  - Registered `.manage(db_worker)` on the Tauri builder
  - Registered all 16 commands in `.invoke_handler(tauri::generate_handler![...])`

## Command Reference (for frontend)

| Frontend invoke()             | Rust function                 | Arguments                                             | Returns                 |
| ----------------------------- | ----------------------------- | ----------------------------------------------------- | ----------------------- |
| `get_characters`              | `get_characters`              | —                                                     | `Vec<CharacterCardV2>`  |
| `save_character`              | `save_character`              | `character: CharacterCardV2`                          | `CharacterCardV2`       |
| `delete_character`            | `delete_character`            | `id: String`                                          | `void`                  |
| `search_characters`           | `search_characters`           | `query: String`                                       | `Vec<CharacterCardV2>`  |
| `get_personas`                | `get_personas`                | —                                                     | `Vec<Persona>`          |
| `save_persona`                | `save_persona`                | `persona: Persona`                                    | `Persona`               |
| `delete_persona`              | `delete_persona`              | `id: String`                                          | `void`                  |
| `search_personas`             | `search_personas`             | `query: String`                                       | `Vec<Persona>`          |
| `get_conversations`           | `get_conversations`           | —                                                     | `Vec<Conversation>`     |
| `start_conversation`          | `start_conversation`          | `characterId: String`                                 | `Conversation`          |
| `delete_conversation`         | `delete_conversation`         | `id: String`                                          | `void`                  |
| `search_conversations`        | `search_conversations`        | `query: String`                                       | `Vec<Conversation>`     |
| `get_messages`                | `get_messages`                | `conversationId: String`                              | `Vec<Message>`          |
| `save_message`                | `save_message`                | `message: Message`                                    | `void`                  |
| `update_conversation_persona` | `update_conversation_persona` | `conversationId: String`, `personaId: Option<String>` | `void`                  |
| `get_presets`                 | `get_presets`                 | —                                                     | `Vec<GenerationPreset>` |

**Note:** Tauri v2 applies camelCase-to-snake_case conversion for command arguments. Frontend sends `{ characterId }` which maps to Rust's `character_id`.

## Bug Fix: WAL Pragma in Transaction

Discovered and fixed a latent bug: `PRAGMA journal_mode=WAL` cannot be executed inside a transaction, but `rusqlite_migration` wraps migrations in a transaction. This was previously masked by the fire-and-forget DbWorker pattern where errors were silently logged. The fix moves PRAGMA statements out of the migration SQL and into a separate `configure_connection()` function called before `run_migrations()`.

## Verification

- `cargo test --lib` — 80 tests pass ✅
- `cargo check` — no errors ✅
- `cargo build` — succeeds ✅
- 16 commands registered in `invoke_handler` ✅
- DbWorker initialized with `dirs::data_dir()` → `com.revelry.desktop/revelry.db` ✅
