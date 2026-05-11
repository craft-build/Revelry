# Plan 02-02: SQLite CRUD Layer — COMPLETE

## Objective
Implement the SQLite CRUD layer — row mappers, character/persona/conversation/preset CRUD functions — replacing the stubs from plan 02-01, so that DbWorker can execute real database operations.

## Completed Tasks

### Task 1: Row mappers in models.rs
Replaced empty placeholder with 5 row mapper functions:
- **`row_to_character`**: Extracts all 23 columns including creator_notes, character_book, default_persona_id, default_preset_id, lorebook_scan_depth, lorebook_recursive_depth. Parses JSON TEXT columns (alternate_greetings, tags, extensions, character_book) via `serde_json::from_str`. Uses `parse::<DateTime<Utc>>()` for all timestamps.
- **`row_to_persona`**: Extracts all columns. Parses JSON TEXT column: tags.
- **`row_to_conversation`**: NEW mapper with Phase 2 fields: title, last_message_preview, generation_preset_id, provider_id (Option<String>).
- **`row_to_message`**: NO branch_id. Parses role via `MessageRole::from_str()`. Extracts: id, conversation_id, role, content, sort_order, timestamp.
- **`row_to_generation_preset`**: Parses JSON TEXT column: stop_sequences. Converts is_builtin from i32 to bool via `!= 0`.
- **`parse_timestamp`**: Shared helper for RFC 3339 → `DateTime<Utc>` conversion with proper error mapping.

### Task 2: Character CRUD in characters.rs
Replaced stub with full CRUD:
- **`insert_character`**: INSERT with all 23 columns. `serde_json::to_string()` for JSON fields. `.to_rfc3339()` for timestamps.
- **`load_characters`**: SELECT all columns ORDER BY updated_at DESC.
- **`load_character`**: SELECT by id, returns `Option<CharacterCardV2>`.
- **`update_character`**: UPDATE all mutable fields WHERE id = ?22. Includes default_preset_id (missing from gtk_ref).
- **`delete_character`**: DELETE WHERE id = ?1.
- **`search_characters`**: FTS5 search with JOIN, MATCH, ORDER BY rank. Returns empty Vec for empty/whitespace queries.
- **`sanitize_fts_query`**: Escapes double quotes by doubling, wraps in double quotes for phrase matching.
- 11 unit tests including FTS sync after update/delete, JSON field roundtrips, empty query handling.

### Task 3: Persona CRUD in personas.rs
Replaced stub with full CRUD:
- **`insert_persona`**: INSERT with all columns. `serde_json::to_string()` for tags. `.to_rfc3339()` for timestamps.
- **`load_personas`**: SELECT all columns ORDER BY updated_at DESC.
- **`load_persona`**: SELECT by id, returns Option.
- **`update_persona`**: UPDATE mutable fields.
- **`delete_persona`**: DELETE.
- **`search_personas`**: SQL LIKE on name and personality columns with ESCAPE clause for safe pattern matching.
- 8 unit tests including search by name/personality, tags serialization, no-match cases.

### Task 4: Conversation and message CRUD in conversations.rs
Replaced stub with full CRUD:
- **`start_conversation`**: Loads character via `load_character`, creates Conversation with title="Chat with {char.name}", last_message_preview=char.first_mes, inserts, creates initial assistant message if first_mes is non-empty.
- **`load_conversations`**: SELECT all columns including title, last_message_preview, generation_preset_id ORDER BY updated_at DESC.
- **`delete_conversation`**: DELETE with CASCADE handling messages.
- **`load_messages`**: SELECT ORDER BY sort_order ASC, NO branch_id filtering.
- **`save_message`**: INSERT message, then UPDATE conversations SET updated_at, last_message_preview (truncated to 100 chars).
- **`search_conversations`**: SQL LIKE on title and last_message_preview.
- **`update_conversation_persona`**: UPDATE creator_persona_id and updated_at.
- 9 unit tests including cascade deletion, preview updates, persona assignment/clearing.

### Task 5: Generation preset functions
Added to generation.rs:
- **`load_presets`**: SELECT all presets ORDER BY name using `row_to_generation_preset` mapper.
- Updated existing test to use `load_presets` instead of raw SQL.
- Added `test_load_presets_returns_domain_models` test verifying full domain model roundtrip.

### Task 6: DbWorker wiring
Updated `src-tauri/src/db/mod.rs`:
- Replaced all `todo!()` stubs with real CRUD function calls.
- **SaveCharacter/SavePersona**: Insert-or-update logic (check existence → update or insert).
- **SearchPersonas**: New match arm calling `personas::search_personas`.
- **UpdateConversationPersona**: New match arm calling `conversations::update_conversation_persona`.
- Removed `#[allow(unused_imports)]` markers; cleaned up unused imports.
- Added `test_db_worker_crud_cycle` integration test exercising character and persona CRUD through DbWorker commands.
- Updated `test_db_worker_starts_and_handles_send` to actually send and process commands.

## Verification
- `cd src-tauri && cargo test --lib` → **80 passed, 0 failed**
- `cd src-tauri && cargo check` → **clean, no warnings**
- No `todo!()` remains in any `src-tauri/src/db/` file

## Key Design Decisions
1. **Timestamps**: RFC 3339 format (`.to_rfc3339()` / `.parse::<DateTime<Utc>>()`) instead of gtk_ref's `NaiveDateTime` with custom format.
2. **No branch_id**: Messages table has no branch_id column per Phase 2 schema.
3. **SaveMessage updates preview**: `last_message_preview` is truncated to 100 chars for list display.
4. **SaveCharacter/SavePersona**: Uses upsert pattern (check existence → update or insert) since DbCommand doesn't distinguish between insert and update.
5. **LIKE search escaping**: `%` and `_` in user queries are escaped to prevent unintended pattern matching.
6. **default_preset_id**: Included in character INSERT and UPDATE (was missing from gtk_ref).
