# Phase 3: Editors — Research

**Created:** 2026-05-12
**Purpose:** Answer "What do I need to know to PLAN this phase well?"

---

## Executive Summary

Phase 3 adds editor UIs for creating/editing characters (4-step wizard), personas (single-form dialog), and lorebooks (standalone editor with keyword testing), plus avatar upload and delete actions. The planning challenge is **orchestrating 12+ new components, a schema migration, new Tauri commands, and wiring into existing AppState** — all while maintaining the established patterns (Svelte 5 runes, Bits.ui, Tauri IPC, DbWorker background thread).

This research identifies 8 knowledge areas the planner must address, 6 key risks, and concrete references to existing code that each plan must touch.

---

## Knowledge Areas

### K-01: Bits.ui Dialog & DropdownMenu APIs

**Why it matters:** Three editor dialogs, one delete confirmation dialog, and one kebab menu all depend on Bits.ui primitives. The planner needs to know the exact API surface available.

**What I found:**
- **No Dialog component exists yet** in `src/lib/components/ui/`. PersonaSwitcher uses `bits-ui/Popover`, not Dialog. The UI-SPEC requires a generic `Dialog` wrapper.
- **No DropdownMenu component exists yet** in `src/lib/components/ui/`. The kebab menu needs Bits.ui `DropdownMenu`.
- **bits-ui ^2.18** is installed (`package.json`). This version ships:
  - `Dialog.Root`, `Dialog.Trigger`, `Dialog.Portal`, `Dialog.Overlay`, `Dialog.Content`, `Dialog.Title`, `Dialog.Description`, `Dialog.Close`
  - `DropdownMenu.Root`, `DropdownMenu.Trigger`, `DropdownMenu.Content`, `DropdownMenu.Item`, `DropdownMenu.Separator`, `DropdownMenu.Group`
  - Both use the same portal-based rendering pattern as Popover (already proven in PersonaSwitcher).
- **Key Dialog features needed:**
  - `open` controlled binding (to manage dialog open/close from parent)
  - `onOpenChange` callback (for dirty-form guard per UI-SPEC § Interaction Patterns)
  - Modal behavior (backdrop click + Escape close)
  - Focus trap (built-in with Bits.ui Dialog modal)
- **Key DropdownMenu features needed:**
  - Trigger with custom content (three-dot `MoreVertical` icon)
  - Destructive item variant (for "Delete" option)
  - `onSelect` handlers per item
  - Keyboard navigation (built-in)

**Gap:** The planner must decide whether to create a generic `Dialog` wrapper component (`src/lib/components/ui/Dialog.svelte`) that all three editor dialogs reuse, or have each editor embed Bits.ui Dialog directly. The UI-SPEC inventory lists a "Dialog" primitive, so a wrapper is expected.

**References:**
- `src/lib/components/chat/PersonaSwitcher.svelte` — existing Bits.ui Popover usage pattern
- `package.json` → `bits-ui: ^2.18`
- UI-SPEC § Component Inventory → "Dialog" and "KebabMenu" rows

---

### K-02: Character Editor Wizard — Step State Machine

**Why it matters:** The 4-step wizard is the most complex new UI component. It has two modes (create: linear, edit: free-jump), step-level validation, and conditional navigation. The planner must define the state machine clearly.

**What I found:**
- **UI-SPEC defines the interaction patterns:**
  - Create mode: linear progression, "Next" validates current step before advancing, "Back" preserves data
  - Edit mode: free-jump, clicking any step header navigates directly
  - "Save" (Step 4) validates ALL required fields across all steps
  - Navigation disabled during save (saving state contract)
- **Archival reference (`gtk_ref/src/ui/character_editor.rs`):**
  - Used `EditorPage` enum with Identity/Personality/Scenario/Lorebook/Advanced pages (6 pages, different grouping)
  - Phase 3 has 4 steps: Identity/Personality/Prompt/Greeting (per CONTEXT D-04)
  - GTK editor used `adw::PreferencesWindow` with `adw::PreferencesPage` per step — the Tauri version will use a custom stepper UI
- **Step field groupings (from CONTEXT D-04):**
  - Step 1 Identity: name, description, avatar, tags, creator, version, creator_notes
  - Step 2 Personality: personality, scenario
  - Step 3 Prompt: system_prompt, post_history_instructions
  - Step 4 Greeting: first_mes, mes_example, alternate_greetings
- **Required fields per step (from UI-SPEC § Validation Errors):**
  - Step 1: name, description
  - Step 2: personality
  - Step 4: first_mes
  - Step 3: none required

**Key planning decisions:**
1. How to model wizard state: `$state` object with `currentStep`, `mode`, `formData`, `errors`, `isSaving`
2. Whether each step is a separate Svelte component or conditional blocks within the dialog
3. How form data flows between steps (single reactive object vs. step-local state)
4. How to implement free-jump in edit mode (enable/disable step header clicks based on visited steps? Or always allow?)

**References:**
- `gtk_ref/src/ui/character_editor.rs` — archival reference (L22-898, 1151 total)
- `src-tauri/src/domain/character.rs` — `CharacterCardV2::new()` constructor, all fields, serde defaults
- UI-SPEC § Interaction Patterns — wizard behavior table
- UI-SPEC § Validation Errors — character wizard validation table
- CONTEXT D-04, D-05, D-06, D-07

---

### K-03: Avatar Upload via Tauri File Dialog + Asset Protocol

**Why it matters:** Avatar upload requires a Tauri backend command, file system interaction, and URL resolution — a cross-cutting concern that touches frontend, backend, and the Tauri runtime.

**What I found:**
- **Tauri plugins already registered** in `src-tauri/src/lib.rs`:
  - `tauri_plugin_fs::init()` — filesystem access
  - `tauri_plugin_dialog::init()` — native file dialogs
  - `tauri_plugin_store::init()` — key-value store
- **Frontend dialog plugin** already installed: `@tauri-apps/plugin-dialog: ^2.7.1` in `package.json`
- **CSP already allows asset protocol:** `img-src: 'self' asset: http://asset.localhost blob: data:` in `tauri.conf.json` (L14-20, L21-28)
- **Tauri v2 asset protocol:** Uses `convertFileSrc()` from `@tauri-apps/api/core` to convert a filesystem path to `http://asset.localhost/{encoded_path}`. The webview can then display the image via this URL.
- **Existing Avatar component** (`src/lib/components/ui/Avatar.svelte`): Accepts `src` (string) and `name` (string). Currently renders `bits-ui/Avatar.Image` with `src` and falls back to initials. No changes needed to the rendering — just need to provide the correct `asset://` URL.
- **Avatar path field:** Both `CharacterCardV2.avatar_path: Option<String>` and `Persona.avatar_path` (not currently on the Rust struct, but the TS type `UserPersona` has it). **Gap:** The Rust `Persona` struct does NOT have `avatar_path` — this must be added.

**Key planning decisions:**
1. **Avatar storage path:** CONTEXT D-10 says `app_data_dir/avatars/{uuid}.{ext}`. The app data dir is `dirs::data_dir()/com.revelry.desktop/` (matching the DB path pattern from `lib.rs`).
2. **Avatar URL resolution strategy (from CONTEXT D-13, undecided):**
   - Option A: Backend resolves `avatar_path` to `asset://` URL before returning entity data (modify row mappers)
   - Option B: Frontend calls `get_avatar_url(avatar_path)` command separately
   - Option A is simpler (no extra round-trip) but couples the DB layer to the Tauri runtime. Option B is cleaner separation but requires an extra invoke per entity load.
3. **Persona `avatar_path` field:** Must be added to `Persona` Rust struct, `personas` DB table (migration), and TypeScript `UserPersona` type.
4. **Avatar cleanup:** CONTEXT D-12 — delete old file on replace, delete on entity delete. Needs a file-delete helper in the backend.

**References:**
- `src-tauri/src/lib.rs` — plugin registration (L15-18)
- `src-tauri/tauri.conf.json` — CSP with asset: protocol (L14-20)
- `src/lib/components/ui/Avatar.svelte` — existing avatar component
- `src-tauri/src/domain/persona.rs` — missing `avatar_path` field
- CONTEXT D-10, D-11, D-12, D-13

---

### K-04: Lorebook Schema Migration — Standalone Lorebooks

**Why it matters:** This is the most architecturally significant backend change in Phase 3. The current `lorebook_entries` table has `character_id` as FK. CONTEXT D-14/D-15 refactors to standalone lorebooks with a many-to-many relationship.

**What I found:**
- **Current schema** (`src-tauri/src/db/migrations.rs` L107-121):
  ```sql
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
  ```
- **Target schema (from CONTEXT D-15):**
  - New `lorebooks` table: id, name, description, created_at, updated_at
  - Modified `lorebook_entries` table: replace `character_id` FK with `lorebook_id` FK, add `activation_state` enum column, add `extensions` JSON column, rename `keywords` → `keys` and `secondary_terms` → `secondary_keys`, rename `insertion_position` → `position`, add `depth` column
  - New `character_lorebooks` junction table: character_id, lorebook_id, sort_order
- **Migration strategy:**
  - Phase 2 used a single collapsed migration. Phase 3 adds a second migration.
  - The `lorebook_entries` table currently has NO data (it was unused in Phase 2 — the DB layer `lorebook.rs` is empty except for a comment). This means the migration can simply DROP and recreate rather than ALTER.
  - **But** `rusqlite_migration` applies migrations incrementally — the new migration will be M::up #2 in the migrations vector. The planner must add it to the existing `get_migrations()` function.
- **Type changes:** The frontend `LorebookEntry` type (in `src/lib/types/index.ts`) has `character_id: string` — must change to `lorebook_id: string`, plus add new fields. A new `Lorebook` type is needed.
- **Archival reference domain model** (`gtk_ref/src/domain/lorebook.rs`): Uses `InsertionPosition` enum (BeforeChar/AfterChar), `depth_from_bottom: i32`, `enabled: bool`, `token_budget: Option<i64>`. The Phase 3 version adds `activation_state` (always/character_only/user_only) per CONTEXT D-18.

**Key planning decisions:**
1. **Migration approach:** Since `lorebook_entries` has no data, can we DROP the old table and create the new schema from scratch? Or should we preserve the table and ALTER it? Dropping is simpler but changes the migration history. **Recommendation:** Add a new migration that drops and recreates since there's no data to preserve. This is consistent with Phase 2's "single collapsed migration" approach.
2. **Field naming alignment:** CONTEXT D-18 uses `keys` (not `keywords`), `secondary_keys` (not `secondary_terms`), `position` (not `insertion_position`), `depth` (not `depth_from_bottom`). The planner must decide whether to align DB column names with CONTEXT or keep backward compatibility with existing column names. Since there's no data, alignment is cleaner.
3. **Rust domain models needed:** `Lorebook` (new) and `LorebookEntry` (new, replacing gtk_ref version). Must be added to `src-tauri/src/domain/` and registered in `mod.rs`.

**References:**
- `src-tauri/src/db/migrations.rs` — current schema (L107-121)
- `src-tauri/src/db/lorebook.rs` — empty stub (1 line comment)
- `src-tauri/src/domain/mod.rs` — must add `lorebook` module
- `gtk_ref/src/domain/lorebook.rs` — archival reference model
- `gtk_ref/src/service/lorebook.rs` — keyword matching logic (for `test_lorebook_keywords` command)
- CONTEXT D-14, D-15, D-16, D-17, D-18

---

### K-05: Keyword Testing Logic for Lorebook Editor

**Why it matters:** CONTEXT D-17 and requirement LORB-06 require an inline keyword test panel. This needs a backend command that simulates keyword activation.

**What I found:**
- **Archival reference** (`gtk_ref/src/service/lorebook.rs` L27-157):
  - `activate_lorebook_entries()` — full keyword matching with scan depth, recursive activation, token budget
  - `keyword_matches()` — OR logic, case-insensitive, any keyword match activates entry
  - `inject_lorebook_entries()` — depth-based insertion into prompt
- **Phase 3 scope is narrower:** The keyword TEST panel only needs to show which entries would activate for given sample text. It does NOT need:
  - Recursive activation (that's prompt engine territory, Phase 9)
  - Token budget enforcement
  - Actual prompt injection
  - It just needs: "given these entries and this sample text, which entries match and which keywords triggered them?"
- **Implementation options:**
  - **Option A: Backend command `test_lorebook_keywords`** — Rust implements the matching logic, returns a list of `{entry_id, matched_keys[]}` results. Frontend just displays them.
  - **Option B: Frontend-only matching** — Svelte/TS implements simple keyword-in-text matching. No backend call needed. Simpler but duplicates logic that Phase 9 will need in Rust anyway.
  - **Recommendation:** Option A is better — the matching logic belongs in Rust (it will be reused by the prompt engine), and the backend can return structured results with matched keys highlighted.

**Key planning decisions:**
1. **Command signature:** `test_lorebook_keywords(entries: Vec<LorebookEntry>, sample_text: string) -> Vec<KeywordTestResult>` where `KeywordTestResult { entry_id, entry_name, matched_keys, is_activated }`
2. **Debouncing:** UI-SPEC says "Debounced (300ms) call to `test_lorebook_keywords`". This is a frontend concern — Svelte 5 `$effect` with a debounce timer.
3. **Keyword highlighting:** Frontend must highlight matched keywords in the test results. This is a presentational concern — the backend returns matched keys, the frontend renders them with accent styling.

**References:**
- `gtk_ref/src/service/lorebook.rs` — `activate_lorebook_entries()` (L27-157), `keyword_matches()` (L219-225)
- `gtk_ref/src/ui/lorebook_editor.rs` — archival GTK lorebook editor (L1-230+)
- UI-SPEC § Interaction Patterns — "Test Keywords panel toggle" and "Keyword test input"
- CONTEXT D-17

---

### K-06: AppState CRUD Actions + Dialog Open/Close State

**Why it matters:** Every editor dialog needs to open from the library page, and every save/delete needs to update both the backend AND the local reactive state. The planner must define the action API clearly.

**What I found:**
- **Current AppState** (`src/lib/stores/app.svelte.ts`): Has `hydrate()`, `openConversation()`, `appendMessage()`, `updateStreamingMessage()`, `switchPersona()`, `toggleSidebar()`. No CRUD actions for characters, personas, or lorebooks. No dialog state management.
- **Current api.ts** (`src/lib/data/api.ts`): Has `saveCharacter()`, `deleteCharacter()`, `savePersona()`, `deletePersona()`. No lorebook commands. No avatar commands.
- **Optimistic update pattern** (from `switchPersona()`): Updates local state first, then persists to backend, reverts on failure. This is the established pattern and should be followed for all CRUD actions.
- **Dialog state:** The library page needs to track which dialog is open and what entity (if any) is being edited. Options:
  - Option A: Dialog state in AppState (`activeEditor: { type: 'character' | 'persona' | 'lorebook', entityId?: string }`)
  - Option B: Dialog state in the library page component (local `$state`)
  - **Recommendation:** Option B is cleaner — dialog open/close is a local UI concern, not global state. Only the CRUD results need to update AppState.

**New AppState actions needed:**
```typescript
// Character
createCharacter(data: Partial<CharacterCard>): Promise<void>
updateCharacter(id: string, data: Partial<CharacterCard>): Promise<void>
deleteCharacter(id: string): Promise<void>

// Persona
createPersona(data: Partial<UserPersona>): Promise<void>
updatePersona(id: string, data: Partial<UserPersona>): Promise<void>
deletePersona(id: string): Promise<void>

// Lorebook
createLorebook(data: Partial<Lorebook>): Promise<void>
updateLorebook(id: string, data: Partial<Lorebook>): Promise<void>
deleteLorebook(id: string): Promise<void>
```

**New api.ts commands needed:**
```typescript
// Lorebook CRUD
getLorebooks(): Promise<Lorebook[]>
saveLorebook(lorebook: Lorebook): Promise<Lorebook>
deleteLorebook(id: string): Promise<void>
getLorebookEntries(lorebookId: string): Promise<LorebookEntry[]>
saveLorebookEntry(entry: LorebookEntry): Promise<LorebookEntry>
deleteLorebookEntry(id: string): Promise<void>
testLorebookKeywords(entries: LorebookEntry[], sampleText: string): Promise<KeywordTestResult[]>
assignLorebookToCharacter(characterId: string, lorebookId: string, sortOrder: number): Promise<void>
removeLorebookFromCharacter(characterId: string, lorebookId: string): Promise<void>

// Avatar
uploadAvatar(filePath: string): Promise<string>  // returns avatar_path
getAvatarUrl(avatarPath: string): Promise<string>  // returns asset:// URL (if Option B from K-03)
```

**Key planning decisions:**
1. Whether CRUD actions are on AppState or called directly from editor components
2. Optimistic vs. wait-for-backend update strategy for each operation
3. Whether `hydrate()` needs to also load lorebooks (yes, for the Lorebooks tab)
4. How delete cascades (character delete → also delete avatar file + conversation cleanup)

**References:**
- `src/lib/stores/app.svelte.ts` — current AppState structure
- `src/lib/data/api.ts` — current API layer
- CONTEXT D-02 (separate dialogs), D-03 (delete on kebab menu)

---

### K-07: Tauri Backend Commands — New Command Registration

**Why it matters:** Every frontend action requires a corresponding Tauri command. The planner must enumerate all new commands, their signatures, and their integration with the DbWorker.

**What I found:**
- **Current command registration** (`src-tauri/src/lib.rs` L25-42): 16 commands registered via `invoke_handler`. Pattern: `commands::module::command_name`.
- **Current command module structure:** `characters.rs`, `personas.rs`, `conversations.rs`, `presets.rs` — each with `get_*`, `save_*`, `delete_*`, `search_*` functions.
- **DbWorker pattern** (`src-tauri/src/db/mod.rs`): Commands send `DbCommand` variants via `mpsc::Sender`, await results via `oneshot::Receiver`. Each new DB operation needs:
  1. New `DbCommand` variant in the enum
  2. New match arm in `DbWorker::handle_command()`
  3. New CRUD function in the appropriate db module
  4. New Tauri command function in `commands/`
  5. Registration in `lib.rs` `invoke_handler`

**New Tauri commands needed:**

| Command | Module | DbCommand Variant | Notes |
|---------|--------|-------------------|-------|
| `get_lorebooks` | commands/lorebooks.rs | LoadLorebooks | New module |
| `save_lorebook` | commands/lorebooks.rs | SaveLorebook | Upsert pattern |
| `delete_lorebook` | commands/lorebooks.rs | DeleteLorebook | Cascade to entries |
| `get_lorebook_entries` | commands/lorebooks.rs | LoadLorebookEntries | By lorebook_id |
| `save_lorebook_entry` | commands/lorebooks.rs | SaveLorebookEntry | Upsert pattern |
| `delete_lorebook_entry` | commands/lorebooks.rs | DeleteLorebookEntry | |
| `upload_avatar` | commands/avatars.rs | (file I/O, not DB) | New module, no DbCommand |
| `get_avatar_url` | commands/avatars.rs | (pure computation) | Only if Option B from K-03 |
| `test_lorebook_keywords` | commands/lorebooks.rs | (pure computation) | Keyword matching, no DB |
| `assign_lorebook_to_character` | commands/lorebooks.rs | AssignLorebookToCharacter | Junction table insert |
| `remove_lorebook_from_character` | commands/lorebooks.rs | RemoveLorebookFromCharacter | Junction table delete |

**Note:** `upload_avatar` and `get_avatar_url` are NOT database operations — they're file I/O and URL construction. They should be plain Tauri commands, NOT routed through DbWorker. `test_lorebook_keywords` is pure computation (keyword matching logic), also NOT a DB operation.

**Key planning decisions:**
1. How `upload_avatar` accesses the app data directory (Tauri app handle vs. `dirs::data_dir()`)
2. Whether avatar commands go in their own module (`commands/avatars.rs`) or are added to an existing module
3. Error handling for avatar upload failures (invalid image format, disk full, permission denied)
4. Whether `test_lorebook_keywords` should reuse `keyword_matches()` from gtk_ref or implement fresh

**References:**
- `src-tauri/src/lib.rs` — command registration (L25-42)
- `src-tauri/src/commands/mod.rs` — current module structure
- `src-tauri/src/db/mod.rs` — DbCommand enum and DbWorker (L1-230)
- `gtk_ref/src/service/lorebook.rs` — keyword matching algorithm (L219-225)

---

### K-08: Existing Component Modifications — Cards & Library Page

**Why it matters:** CharacterCard and PersonaCard need kebab menus. The library page needs editor dialog integration. These are modifications to existing working components — the planner must know exactly what changes.

**What I found:**
- **CharacterCard** (`src/lib/components/characters/CharacterCard.svelte`):
  - Currently has: `onchat` and `onedit` callback props
  - "Start chat" button (primary accent) + "Edit" pencil button
  - UI-SPEC says: Replace "Edit" button with kebab menu, add `ondelete` prop
  - The kebab menu trigger replaces the standalone pencil button
- **PersonaCard** (`src/lib/components/personas/PersonaCard.svelte`):
  - Currently has: `onedit` callback prop
  - "Edit" button + "Duplicate" button (no-op)
  - UI-SPEC says: Replace "Edit" button with kebab menu, add `ondelete` prop
  - "Duplicate" button stays (still no-op per deferred list)
- **Library page** (`src/routes/characters/+page.svelte`):
  - Currently has: Characters tab + Personas tab, search, "New character"/"New persona" buttons (no-op)
  - UI-SPEC says: Add "Lorebooks" tab, wire buttons to open editor dialogs
  - Currently CharacterCard/PersonaCard don't pass `onchat`/`onedit` callbacks — the components are rendered without them. **Bug:** The cards define `onchat` and `onedit` props but the library page doesn't pass them. This needs fixing.
- **AppState hydration:** Currently loads characters, personas, conversations, presets. Must also load lorebooks for the Lorebooks tab.

**Key planning decisions:**
1. Whether the kebab menu is a separate `KebabMenu` UI primitive (per UI-SPEC inventory) or inline in each card component
2. How the library page passes editor callbacks to cards (direct props vs. event delegation)
3. Lorebooks tab empty state vs. populated state layout

**References:**
- `src/lib/components/characters/CharacterCard.svelte` — current card implementation
- `src/lib/components/personas/PersonaCard.svelte` — current card implementation
- `src/routes/characters/+page.svelte` — current library page
- UI-SPEC § Component Inventory — modified existing components table

---

## Key Risks

### R-01: Schema Migration Ordering Risk

**Severity:** High
**Description:** Adding a second migration to `get_migrations()` requires that the migration applies cleanly on top of the Phase 2 schema. If the schema has drifted (e.g., manual DB edits during development), the migration could fail.
**Mitigation:** Since `lorebook_entries` has no data and the change is purely additive (new tables + column restructure of an empty table), the risk is low. However, the planner should include a test that verifies the migration applies on a fresh DB AND on a DB with the Phase 2 schema already applied.

### R-02: Bits.ui Dialog Accessibility & Dirty Form Guard

**Severity:** Medium
**Description:** The UI-SPEC requires an `onBeforeClose` handler for unsaved changes. Bits.ui Dialog may not expose this directly — the planner needs to verify the API supports intercepting close events.
**Mitigation:** Bits.ui Dialog v2 has `onOpenChange` which fires when the open state is about to change. If this fires before the state changes, we can use it as a guard. If not, we may need to control the `open` state manually and prevent the default close behavior. Test this during plan 03-01.

### R-03: Avatar Asset Protocol Path Encoding

**Severity:** Medium
**Description:** Tauri's `convertFileSrc()` or asset protocol requires proper URL encoding of file paths. If the avatar file path contains spaces or special characters (e.g., `avatars/123e4567-e89b.png`), the URL must be correctly encoded for the webview to load the image.
**Mitigation:** The planner should ensure the backend uses consistent path construction (no spaces in UUIDs — UUID v4 strings are safe). Test with the actual Tauri runtime during plan 03-04.

### R-04: Cross-Plan State Synchronization

**Severity:** Medium
**Description:** Plans 03-01 through 03-04 are executed sequentially. Plan 03-01 (character editor) will add `createCharacter`/`updateCharacter`/`deleteCharacter` to AppState. Plan 03-02 (persona editor) adds similar methods. Plan 03-03 (lorebook editor) adds lorebook types + methods. Plan 03-04 (wiring) connects everything. Each plan modifies the same files (AppState, api.ts, types). Merge conflicts or inconsistent intermediate states are possible.
**Mitigation:** Each plan should clearly delineate which files it modifies. The planner should define the complete AppState API upfront in plan 03-01 and have subsequent plans only ADD to it, not restructure it.

### R-05: Persona `avatar_path` Field Gap

**Severity:** Medium
**Description:** The Rust `Persona` struct (`src-tauri/src/domain/persona.rs`) does NOT have an `avatar_path` field, but the TypeScript `UserPersona` type does. The DB schema `personas` table also lacks `avatar_path`. Adding it requires a schema migration.
**Mitigation:** Add `avatar_path TEXT` column to `personas` table in the Phase 3 migration. Add the field to the Rust `Persona` struct with `#[serde(default)]`. Update the row mapper in `models.rs`.

### R-06: Keyword Test Performance

**Severity:** Low
**Description:** The keyword test panel debounces at 300ms. For large lorebooks (100+ entries), the matching computation + Tauri IPC round-trip might feel slow.
**Mitigation:** The Tauri IPC is local (no network), and keyword matching is O(n*m) where n=entries, m=keywords. For reasonable lorebooks (<500 entries, <10 keywords each), this is trivially fast. If performance becomes an issue, the debounce interval can be increased or matching can be done client-side.

---

## Component Dependency Graph

Understanding what must be built before what:

```
                    ┌─────────────────┐
                    │   Dialog (ui)   │ ← ALL editors depend on this
                    └────────┬────────┘
                             │
              ┌──────────────┼──────────────────┐
              │              │                  │
   ┌──────────▼──────┐  ┌───▼──────────┐  ┌───▼────────────┐
   │ StepperHeader    │  │ KebabMenu    │  │ AvatarUpload   │
   │ (ui)             │  │ (ui)         │  │ (ui)           │
   └──────────┬──────┘  └───┬──────────┘  └───┬────────────┘
              │              │                  │
   ┌──────────▼──────────────▼──────────────────▼────────┐
   │                                                       │
   │  CharacterEditorDialog  PersonaEditorDialog           │
   │  (depends on: Dialog, StepperHeader, AvatarUpload)    │
   │  (depends on: Dialog, AvatarUpload)                   │
   │                                                       │
   │  LorebookEditor                                       │
   │  (depends on: Dialog, Textarea, Select)               │
   │                                                       │
   │  DeleteConfirmationDialog                             │
   │  (depends on: Dialog)                                 │
   │                                                       │
   └──────────┬───────────────────┬────────────────────────┘
              │                   │
   ┌──────────▼──────┐  ┌────────▼─────────┐
   │  CharacterCard   │  │  PersonaCard      │
   │  (add KebabMenu) │  │  (add KebabMenu)  │
   └──────────┬──────┘  └────────┬──────────┘
              │                   │
   ┌──────────▼───────────────────▼────────────────────────┐
   │              characters/+page.svelte                     │
   │  (wire "New" buttons, add Lorebooks tab,               │
   │   render editor dialogs, pass callbacks to cards)      │
   └────────────────────────────────────────────────────────┘
              │
   ┌──────────▼──────────────────────────────────────────────┐
   │  AppState (CRUD actions) + api.ts (new commands)          │
   └──────────┬──────────────────────────────────────────────┘
              │
   ┌──────────▼──────────────────────────────────────────────┐
   │  Tauri Backend (schema migration, domain models,         │
   │  DbCommand variants, CRUD functions, Tauri commands,     │
   │  avatar upload, keyword testing)                         │
   └──────────────────────────────────────────────────────────┘
```

**Build order for new UI primitives (must come first):**
1. `Dialog` wrapper (generic)
2. `Textarea` (form input)
3. `Select` (styled dropdown)
4. `KebabMenu` (dropdown menu)
5. `StepperHeader` (wizard navigation)
6. `AvatarUpload` (avatar interaction)

**Then domain editors:**
7. `DeleteConfirmationDialog`
8. `CharacterEditorDialog`
9. `PersonaEditorDialog`
10. `LorebookEditor`

**Then modifications:**
11. `CharacterCard` (add kebab menu)
12. `PersonaCard` (add kebab menu)
13. `characters/+page.svelte` (wire everything)

**Backend can be built in parallel with the UI layer.**

---

## Plan Scope Recommendations

Based on the ROADMAP plan list and the dependency analysis:

| Plan | Scope | Key Deliverables | Depends On |
|------|-------|------------------|------------|
| 03-01 | Backend + UI Primitives + Character Editor | Schema migration, domain models, DbCommand variants, CRUD functions, Tauri commands for lorebooks+avatars, `Dialog`, `Textarea`, `Select`, `KebabMenu`, `StepperHeader`, `AvatarUpload`, `CharacterEditorDialog`, `DeleteConfirmationDialog`, AppState character CRUD actions | Phase 2 |
| 03-02 | Persona Editor | `PersonaEditorDialog`, AppState persona CRUD actions, persona `avatar_path` field | 03-01 (shares Dialog, AvatarUpload, DeleteConfirmationDialog) |
| 03-03 | Lorebook Editor | `Lorebook` type, `LorebookEditor`, keyword test panel, AppState lorebook CRUD actions, `test_lorebook_keywords` command | 03-01 (shares Dialog, Textarea, Select) |
| 03-04 | Wiring & Integration | Wire "New" buttons, add kebab menus to cards, add Lorebooks tab, wire callbacks, hydration of lorebooks, avatar URL resolution | 03-01, 03-02, 03-03 |

**Alternative:** Plans 03-01 could be split further — backend in 03-01a, UI primitives in 03-01b, character editor in 03-01c. But the ROADMAP already defines 4 plans, so the grouping above keeps within budget.

---

## Tauri-Specific Technical Notes

### Avatar File Dialog API

The Tauri v2 dialog plugin (`tauri-plugin-dialog`) provides:
```rust
// Rust side
use tauri_plugin_dialog::DialogExt;
let file_path = app.dialog().file().filter("Images", &["png", "jpg", "jpeg", "webp"]).blocking_pick_file();
```

```typescript
// TypeScript side
import { open } from '@tauri-apps/plugin-dialog';
const filePath = await open({
  multiple: false,
  filters: [{ name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'webp'] }]
});
```

The planner should decide whether the file picker is opened from the frontend (via `@tauri-apps/plugin-dialog`) or the backend (via `tauri-plugin-dialog` Rust API). **Recommendation:** Frontend opens the picker, passes the selected path to a backend `upload_avatar` command. This keeps the UI interaction on the frontend and file I/O on the backend.

### Asset Protocol URL Construction

Tauri v2 `convertFileSrc()` from `@tauri-apps/api/core`:
```typescript
import { convertFileSrc } from '@tauri-apps/api/core';
const assetUrl = convertFileSrc('/path/to/file.png');
// Returns: http://asset.localhost/localhost/path/to/file.png
```

The `<img>` tag can then use this URL. The CSP already allows `asset: http://asset.localhost`.

### Tauri Command for File Copy (Avatar Upload)

The `upload_avatar` command needs to:
1. Receive the source file path (from the frontend file picker)
2. Generate a UUID for the filename
3. Determine the file extension from the source path
4. Copy the file to `app_data_dir/com.revelry.desktop/avatars/{uuid}.{ext}`
5. Create the `avatars/` directory if it doesn't exist
6. Return the relative path `avatars/{uuid}.{ext}`

This does NOT go through DbWorker — it's pure file I/O. The `avatar_path` is then stored as a field on the character/persona when saved.

---

## Checklist for the Planner

Before writing execution plans, ensure you can answer:

- [ ] **K-01:** What Bits.ui Dialog/DropdownMenu APIs are used, and is the `onBeforeClose` guard feasible?
- [ ] **K-02:** What is the exact state machine for the character wizard (steps, transitions, validation triggers)?
- [ ] **K-03:** What is the avatar URL resolution strategy (backend-in-row-mapper vs. separate command)?
- [ ] **K-04:** What does the new schema migration SQL look like, and does it handle the empty `lorebook_entries` table correctly?
- [ ] **K-05:** What is the `test_lorebook_keywords` command signature and return type?
- [ ] **K-06:** What AppState CRUD actions are needed, and do they use optimistic updates?
- [ ] **K-07:** How many new Tauri commands are registered, and which go through DbWorker vs. plain async?
- [ ] **K-08:** How are the existing card components and library page modified to support kebab menus and editor triggers?
- [ ] **R-01:** Does the migration test cover both fresh DB and existing Phase 2 DB?
- [ ] **R-05:** Is `avatar_path` added to the `personas` table and Rust `Persona` struct?
- [ ] **Cross-plan:** Does plan 03-01 define the complete AppState API so subsequent plans only add to it?

---

*Research completed: 2026-05-12*
