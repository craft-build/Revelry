## Phase-Wide Cross-Cutting Constraints

1. **Rust-TS type alignment:** `LorebookEntry` serde output must match the TypeScript `LorebookEntry` interface — `token_budget` as `number | null`, `sort_order` as `number`, `enabled` as `boolean`. `RegexScript` serde must match TypeScript `RegexScript` interface.

2. **Migration safety:** The Phase 9 migration adds columns with defaults (`sort_order DEFAULT 0`, `enabled DEFAULT 1`) and nullable columns (`token_budget`). Existing rows will have sensible defaults. Test migration on a DB with existing data.

3. **Lorebook budget fraction:** 25% of context window allocated to lorebook by default. This is a constant in the PromptEngine, not yet a user-configurable setting. The budget is split evenly across all assigned lorebooks (character + conversation), per D-04.

4. **Regex application timing:** Regex transforms are applied server-side on stream completion, BEFORE the `Complete` chunk is sent and BEFORE saving to DB. During streaming, raw text displays. This is consistent with Phase 7's conditional rendering pattern.

5. **LorebookActivation role filtering:** The tauri `LorebookActivation` enum (Always/CharacterOnly/UserOnly) is an enhancement over gtk_ref's simple `enabled` bool. The activation logic must scan only messages of the matching role for CharacterOnly/UserOnly entries.

6. **DbCommand registration:** Every new `DbCommand` variant must have a corresponding match arm in `DbWorker::handle_command`. Missing match arms cause compile errors.

7. **Tauri command registration:** Every new `#[tauri::command]` must be registered in `lib.rs` `invoke_handler`. Missing registrations cause "command not found" runtime errors.

8. **Regex validation before save:** `save_regex_script` Tauri command MUST validate the pattern (via `validate_regex_pattern` + `validate_regex_flags`) BEFORE persisting. This ensures REGX-05: "Invalid regex patterns are rejected on save with a clear error message."

9. **Frontend streaming unchanged:** The frontend's `startChat`, `regenerate`, and `editResend` methods in AppState do NOT need changes for regex — the backend applies transforms before sending the `Complete` chunk. The frontend receives already-transformed content.

10. **Regex application is server-side (UI-SPEC reconciliation):** If the UI-SPEC document states that regex should be applied on the frontend, that is superseded by this plan's decision. Regex runs server-side before the `Complete` chunk is emitted, which is necessary because (a) the regex-transformed content is what gets saved to the DB, and (b) applying regex on partially-streamed chunks would produce inconsistent intermediate output. The UI-SPEC should be updated to reflect server-side application on next revision.

11. **Depth-based lorebook entries injected AFTER truncation:** Depth-based lorebook system messages are inserted into history AFTER the oldest-first truncation step, not before. This prevents injected messages at old positions from being silently removed by budget truncation. Depth offsets are therefore relative to the truncated history length, matching SillyTavern/gtk_ref behavior.

12. **`inject_lorebook_entries` is the single source of truth for position handling:** The `inject_lorebook_entries` service function in `service/lorebook.rs` handles all position-based injection logic (`BeforeChar`, `AfterChar`, `AtDepth`). `assemble_with_budget` must call this function rather than duplicating the position-handling logic inline. This avoids drift between the service function and its callers.
