# Phase 6: Generation Controls, Regex & Data - Context

**Gathered:** 2026-05-08
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can fine-tune AI generation parameters, apply regex post-processing to AI responses, and backup/restore all application data.

**In scope:**
- Generation parameter presets (temperature, top_p, max_tokens, frequency_penalty, presence_penalty, stop sequences) with CRUD and assignment hierarchy
- Built-in starter presets (Creative, Balanced, Precise) — read-only
- Per-character regex find-and-replace scripts with sequential execution, ReDoS validation, and test preview
- Export all data as SQL dump or ZIP archive, import from previously exported backup

**Out of scope (deferred):**
- Vault mode / database encryption (v2 / SECU-01, SECU-02)
- Batch character import/export improvements (v2 / CHAR-A04)
- Per-provider tokenizer dispatch (future polish)
- {{char.field}} template expansion beyond {{char}} (deferred from Phase 5)

</domain>

<decisions>
## Implementation Decisions

### Preset Parameter Scope

- **D-01:** Generation presets contain GENR-01 parameters only: name, temperature, top_p, max_tokens, frequency_penalty, presence_penalty, stop sequences
  - No system prompt override in presets — character system prompts stay on the character card
  - Presets are purely about generation behavior, not content

- **D-02:** Extend rig API for unsupported parameters
  - Use rig's agent builder for natively supported params (temperature, max_tokens)
  - Pass unsupported params (top_p, frequency_penalty, presence_penalty, stop sequences) via provider-specific request configuration
  - Silently skip unsupported parameters per-provider (not all providers support all params)

- **D-03:** Preset assignment follows persona hierarchy pattern
  - Global default in Settings
  - Per-character default in Character Editor
  - Per-conversation override in Chat Header
  - Resolution: conversation override → character default → global default → provider defaults

### Regex Script Lifecycle

- **D-04:** Regex scripts stored in separate `regex_scripts` database table
  - Per-script fields: id, character_id, name, find_pattern, replace_pattern, flags, sort_order, enabled, created_at, updated_at
  - character_id FK to characters table with CASCADE delete
  - Import/export serializes to/from extensions.regex_scripts JSON for SillyTavern compatibility
  - Same pattern as lorebook_entries table

- **D-05:** Sequential ordered script execution
  - Scripts run in sort_order sequence (pipe chain)
  - Each script's output becomes the next script's input
  - User controls execution order via sort_order field
  - Disabled scripts are skipped

- **D-06:** ReDoS validation blocks saving with a warning
  - Use regex::RegexBuilder::size_limit() to cap backtracking
  - If pattern fails validation, block saving and show which pattern is unsafe
  - User must fix the pattern before the script can be enabled
  - No runtime timeout — catch dangerous patterns at design time

- **D-07:** Regex test panel in the regex editor
  - Input field for sample text at the bottom of the editor
  - Live before/after display showing applied transformations
  - Inline with editing flow — test as you write

### Backup/Restore Format

- **D-08:** SQL dump uses SQLite's .dump output
  - Pure SQL text file that recreates all tables and inserts all rows
  - Portable — can be loaded by any SQLite tool
  - Includes all tables (no filtering)

- **D-09:** ZIP archive contains DB file + avatar images + settings JSON
  - Full SQLite database file
  - Avatar image directory
  - app_settings export as JSON
  - Everything needed to fully restore app state on a new machine

- **D-10:** Import replaces all existing data with confirmation
  - Show confirmation dialog warning that current data will be wiped
  - Destructive but simple and predictable — matches SillyTavern behavior
  - No merge or side-by-side import mode

- **D-11:** Run migrations after restoring backup
  - After restoring a DB file, run the standard migration suite
  - Older backups auto-upgrade via migrations
  - Newer backups fail with clear "incompatible version" message (can't downgrade schema)

### Starter Presets & Defaults

- **D-12:** Built-in starter presets use SillyTavern-inspired values
  - Creative: temp 1.2, top_p 0.9, penalties 0, no stop sequences
  - Balanced: temp 0.8, top_p 0.9, penalties 0, no stop sequences
  - Precise: temp 0.3, top_p 0.75, penalties 0, no stop sequences

- **D-13:** Provider defaults when no preset is assigned
  - If no preset is assigned at any level, send no parameter overrides to the provider
  - Let the API use its own defaults (typically temp ~0.7-1.0)
  - Simplest and most predictable — respects each provider's tuning

- **D-14:** Built-in presets are read-only
  - Users cannot edit or delete Creative, Balanced, Precise
  - Users can duplicate a built-in preset as a starting point for custom presets
  - Prevents accidental deletion and ensures new users always have defaults

### the agent's Discretion

- Exact regex_scripts table column types and naming (follow lorebook_entries pattern)
- Migration numbering (next after M8)
- How to pass unsupported params to rig provider-specific request config (researcher should investigate rig API)
- Backup file naming convention (timestamp-based)
- ZIP archive internal directory structure
- Preset editor UI layout (sliders vs spinbuttons vs text entry for numeric params)
- Stop sequences UI (comma-separated text entry vs tag-style list)
- How provider-specific parameter support is discovered at runtime
- Default max_tokens per model (null = no limit vs explicit value)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope & Requirements
- `.planning/REQUIREMENTS.md` — GENR-01 through GENR-04, RGXO-01 through RGXO-04, DATA-01 through DATA-03
- `.planning/ROADMAP.md` §Phase 6 — Goal, success criteria, and dependency chain
- `.planning/PROJECT.md` — Core value, constraints, active requirements

### Existing AI & Prompt Services
- `src/service/ai.rs` — AiService::stream_chat() and run_stream() (integration point for generation parameters — agent builder needs parameter injection)
- `src/service/prompt.rs` — PromptEngine::assemble_with_budget() (generation params need to flow through to AiService)
- `src/service/template.rs` — Template resolution patterns (reference for new services)

### Domain & Database
- `src/domain/character.rs` — CharacterCardV2 model (extensions field for regex_scripts serialization, preset assignment columns)
- `src/domain/provider.rs` — ProviderConfig (provider-specific parameter support detection)
- `src/db/mod.rs` — DbService Worker, DbInput/DbOutput enums (add preset/regex/backup variants)
- `src/db/migrations.rs` — Migration pattern (currently M1-M8, add M9+ for presets, regex_scripts tables)
- `src/db/models.rs` — Row conversion helpers (pattern for new models)

### UI Integration
- `src/ui/character_editor.rs` — CharacterEditor pattern (add preset selector dropdown, regex scripts tab)
- `src/ui/chat/mod.rs` — ChatPanel header (add per-conversation preset selector — matches persona popover pattern)
- `src/ui/settings/mod.rs` — SettingsPanel (add default preset dropdown — matches persona assignment)
- `src/ui/lorebook_editor.rs` — LorebookEditor pattern (reference for regex script editor UI)
- `src/ui/persona_editor.rs` — PersonaEditor pattern (reference for preset editor)

### Prior Phase Context
- `.planning/phases/05-context-engineering-lorebook/05-CONTEXT.md` — Lorebook activation engine, token budgeting, context UI (patterns to reuse)
- `.planning/phases/04-personas-template-resolution/04-CONTEXT.md` — Persona assignment hierarchy pattern (replicate for presets)
- `.planning/phases/03-ai-chat-streaming/03-CONTEXT.md` — Streaming integration, prompt assembly baseline

### Technology Stack
- `.planning/research/STACK.md` — regex 1.12.3 for find-and-replace + ReDoS validation, zip 8.6.0 for backup archives, tiktoken-rs for token counting

### Design Specification
- `docs/Revelry_Design_Specification_v2.0.docx` — §Generation Controls, §Regex Scripts, §Data Management

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **AiService::stream_chat()** (`src/service/ai.rs`): Current streaming entry point. Agent builder at line ~102 calls `.agent(model_name).preamble(system_prompt).build()` — needs `.temperature()` and `.max_tokens()` added, plus provider-specific parameter injection
- **DbService Worker** (`src/db/mod.rs`): Serial background thread pattern ready for preset/regex CRUD. Add LoadPresets/SavePreset/DeletePreset, LoadRegexScripts/SaveRegexScript/DeleteRegexScript to DbInput
- **Persona assignment hierarchy** (`src/ui/chat/mod.rs`, `src/ui/settings/mod.rs`, `src/ui/character_editor.rs`): Existing 3-level assignment pattern (global → character → conversation). Clone and adapt for preset assignment
- **LorebookEditor** (`src/ui/lorebook_editor.rs`): Pattern for per-character script management UI with list + editor. Adapt for regex scripts
- **regex crate**: Already in dependency tree (used by tiktoken-rs). Use regex::RegexBuilder::size_limit() for ReDoS validation
- **Migrations** (`src/db/migrations.rs`): Currently M1-M8. M9+ needed for generation_presets table, regex_scripts table, and character conversation preset columns

### Established Patterns
- **Database:** UUID v4 IDs, NaiveDateTime as TEXT, prepare_cached() for CRUD, upsert via exists-check-then-insert-or-update
- **UI Components:** Component trait for components with async operations. AppModel bridges between UI components and DbInput via output messages
- **Service layer:** Stateless services — data flows in, results flow out. PromptEngine, AiService, TokenCountService follow this pattern
- **Assignment hierarchy:** Global default (app_settings table) → per-character (FK column) → per-conversation (FK column). Resolution queries cascade through levels
- **Per-character sub-editors:** LorebookEditor pattern: list panel + editor panel, enable/disable toggle, sort_order control
- **Import/export JSON serialization:** character_book JSON column populated from table data on export, parsed on import (same pattern for regex_scripts → extensions.regex_scripts)

### Integration Points
- **AiService::stream_chat()** needs: generation parameters from resolved preset, passed to rig agent builder and provider-specific config
- **PromptEngine** or **ChatPanel** needs: preset resolution (conversation → character → global → none)
- **DbService** needs: LoadPresets/SavePreset/DeletePreset, LoadRegexScripts/SaveRegexScript/DeleteRegexScript, ExportBackup/ImportBackup
- **ChatPanel** needs: per-conversation preset selector in chat header (MenuButton+Popover, matches persona pattern)
- **Character editor** needs: preset dropdown + regex scripts management tab
- **Settings** needs: default preset dropdown + backup/restore buttons
- **Message display pipeline** needs: regex script execution after AI response and before rendering

</code_context>

<specifics>
## Specific Ideas

- Preset assignment UX mirrors persona assignment exactly — same dropdown pattern in Settings, Character Editor, and Chat Header
- Regex scripts use same table-per-entry pattern as lorebook_entries (not JSON blob)
- Regex test panel should be inline in the editor — test as you write, not a separate dialog
- SillyTavern-inspired preset values match community expectations for roleplay apps
- Backup import is destructive with confirmation — matches SillyTavern behavior and keeps things simple
- Older backups auto-upgrade via migration suite — seamless restore experience

</specifics>

<deferred>
## Deferred Ideas

- Vault mode / database encryption (v2 / SECU-01, SECU-02)
- Batch character import/export improvements (v2 / CHAR-A04)
- Per-provider tokenizer dispatch (future polish)
- {{char.field}} template expansion beyond {{char}} (deferred from Phase 5)
- Preset sharing/export between users (future feature)
- Regex script import/export as standalone files (future polish)
- Incremental/differential backups (future enhancement)
- Cloud backup sync (contradicts local-first core value)

</deferred>

---

*Phase: 06-generation-controls-regex-data*
*Context gathered: 2026-05-08*
