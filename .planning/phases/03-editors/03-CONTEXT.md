# Phase 3: Editors - Context

**Gathered:** 2026-05-12
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can create and edit characters, personas, and lorebooks through guided editor UIs. This phase delivers UI editors (wrapped in modal dialogs) that connect to the existing Tauri CRUD commands from Phase 2, plus backend changes needed for standalone lorebooks and avatar file handling.

In scope:
- Character creation/editing via a multi-step wizard in a modal dialog
- Persona creation/editing via a form dialog
- Avatar image upload via native file picker + Tauri asset protocol
- Delete actions (with confirmation) accessible from card kebab menus
- Standalone lorebook editor with keyword activation testing
- Schema migration: lorebook_entries refactored from per-character to standalone shared resource

NOT in scope: AI provider integration (Phase 4), streaming chat (Phase 5), prompt assembly (Phase 6), card import/export (Phase 7), theme system (Phase 10).

</domain>

<decisions>
## Implementation Decisions

### Editor Container Pattern
- **D-01:** **Modal dialog** (`max-w-2xl` / ~640px) — Bits.ui Dialog-based. Opens over the library page. Centered on screen with vertical scrollable content inside.
- **D-02:** **Separate dialogs** — `CharacterEditorDialog` and `PersonaEditorDialog` as distinct components. Each form is self-contained and tuned to its own field layout. No shared shell abstraction.
- **D-03:** **Delete lives on card kebab menu** — Each `CharacterCard` and `PersonaCard` gets a three-dot/kebab menu with "Edit" and "Delete" options. Selecting Delete opens a confirmation dialog. Not inside the editor dialog.

### Character Editor Flow
- **D-04:** **4-step wizard** for character create and edit. Steps are thematic groupings of the CharacterCardV2 fields:
  - Step 1: **Identity** — Name, description, avatar, tags, creator, version, creator_notes
  - Step 2: **Personality & Scenario** — Personality, scenario
  - Step 3: **Prompt Engineering** — system_prompt, post_history_instructions
  - Step 4: **Greeting & Dialogues** — first_mes (greeting), mes_example (example dialogues), alternate_greetings
- **D-05:** **Navigation mode differs by intent**: Linear Next/Back for create (all steps locked until reached); free-jump (click any step header) for edit. Implementation uses a conditional prop on the dialog: `mode: 'create' | 'edit'`.
- **D-06:** **Required fields for save**: Name, description, personality, and first_mes (greeting) must be non-empty. All other fields optional with `serde(default)` empty-string defaults matching the Rust model.
- **D-07:** **Step-level validation**: Validate required fields on the current step before allowing "Next". Validate all required fields before allowing "Save" (i.e., user reached Step 4).

### Persona Editor Flow
- **D-08:** **Single-form dialog** (not a wizard) — Personas have ~8 fields and are conceptually simpler. All fields visible in one scrollable dialog. Fields: name, avatar_path, appearance, personality, background, role_description, relationships, speech_style, tags.
- **D-09:** **Required fields for save**: Name only. Everything else optional.

### Avatar Handling
- **D-10:** **Tauri asset protocol** — Image files stored in the app's platform data directory and served via Tauri's `asset://` protocol. `avatar_path` stores the relative filename (e.g., `avatars/{uuid}.png`), resolved by the backend into an `asset://` URL for the frontend.
- **D-11:** **Native OS file picker** — Click the avatar area → `tauri-file-dialog` or Tauri dialog API opens a native picker. Tauri backend copies the selected file to `app_data_dir/avatars/{uuid}.{ext}` and returns the filename. No drag-and-drop for now.
- **D-12:** **Auto-cleanup on replace/delete** — When a character/persona is saved with a new avatar, delete the old image file. When the entity is deleted, delete its avatar file. If clone/duplicate is added later, revisit reference counting; for now, avatar files are treated as 1:1 with their entity.
- **D-13:** **Avatar display** — Reuse existing `Avatar` component. Backend resolves `avatar_path` to `asset://` URL before sending to frontend, OR frontend asks backend for avatar URLs via a Tauri command (`get_avatar_url`). To be decided during planning.

### Lorebook Editor Scope
- **D-14:** **Standalone lorebooks** — Lorebooks are named, standalone resources with a list of entries. They exist independently of characters and can be assigned to multiple characters.
- **D-15:** **Schema change** — The Phase 2 `lorebook_entries` table had `character_id` as a foreign key. Refactor to:
  - `lorebooks` table: id, name, description, created_at, updated_at
  - `lorebook_entries` table: id, lorebook_id (FK), keys (JSON), secondary_keys (JSON), content, position, depth, activation_state, extensions (JSON), created_at, updated_at
  - `character_lorebooks` junction table: character_id, lorebook_id, sort_order (for priority/budget allocation)
- **D-16:** **Character's `character_book` JSON deprecated** — The `character_book` JSON blob on CharacterCard is no longer the source of truth for lorebooks. Frontend shows assigned lorebooks via the junction table.
- **D-17:** **Inline per-entry keyword test** — Inside the lorebook editor, a "Test Keywords" panel lets the user type sample text and see which entries activate, with matched keys highlighted. Shows cross-entry interactions (multiple matches, ordering).
- **D-18:** **Lorebook editor entry form fields**: keys (comma-separated), secondary_keys, content (textarea), position (select: before_char/after_char/at_depth), depth (number), activation_state (select: always/character_only/user_only).

### Claude's Discretion
- Exact positioning of the "Test Keywords" panel within the lorebook editor (side panel vs. bottom accordion vs. tab) — I will pick what's most ergonomic.
- Exact step header navigation UI (dots vs. numbered tabs vs. stepper bar) — I will pick based on Bits.ui primitives available.
- How the avatar upload button looks (upload icon, overlay, or replaceable image) — I will pick based on Tailwind patterns.
- Loading state during save (optimistic vs. spinner vs. disabled form) — I will pick based on existing patterns in AppState.
- Error display inside the wizard (inline per-field, top-level banner, or step-level summary) — I will pick based on form size and user flow.
- Whether the lorebook editor opens as a dialog (like character/persona) or as a sub-route/tab within the settings/lorebooks page — Given it's a standalone resource with its own list view, a settings-style page or a dialog are both options. I will evaluate during planning based on the expected workflow (create lorebook → assign to character).

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Planning
- `.planning/ROADMAP.md` — Phase 3 definition, plans list (03-01 through 03-04), success criteria
- `.planning/REQUIREMENTS.md` — CHAR-02, CHAR-03, CHAR-04, CHAR-05, PERS-02, PERS-03, PERS-04, LORB-06 requirements
- `.planning/PROJECT.md` — Project context, constraints, tech stack decisions, key decisions table

### Phase 2 Decisions (carry-forward)
- `.planning/phases/02-persistent-data/02-CONTEXT.md` — D-08 through D-19 (Tauri command design, type alignment, DB threading model)
- `.planning/STATE.md` — Blocker note: "No character or persona creation UI — 6 UAT tests blocked." Phase 3 resolves this.

### Existing Frontend (current state — will be modified)
- `src/lib/types/index.ts` — TypeScript type definitions (CharacterCard, UserPersona, LorebookEntry, LorebookPosition, LorebookActivation)
- `src/lib/stores/app.svelte.ts` — AppState (will need actions: `createCharacter`, `updateCharacter`, `deleteCharacter`, `createPersona`, `updatePersona`, `deletePersona`, `createLorebook`, `updateLorebook`, `deleteLorebook`, `assignLorebookToCharacter`)
- `src/lib/data/api.ts` — Existing `save_character`, `save_persona`, `delete_character`, `delete_persona`, etc. (may need new commands for lorebooks and avatar URL resolution)
- `src/routes/characters/+page.svelte` — Library page with tab strip, search, and card grid. "New character" / "New persona" buttons currently no-ops.
- `src/lib/components/characters/CharacterCard.svelte` — Card with name, description, tags, avatar, "Start chat", "Edit" button. Needs kebab menu added.
- `src/lib/components/personas/PersonaCard.svelte` — Card with name, role_description, tags, avatar, "Edit" button. Needs kebab menu added.
- `src/lib/components/ui/` — Existing UI primitives: Avatar, Button, Input, Tag, Tooltip. Will be extended with Dialog primitives (Bits.ui Dialog already imported in PersonaSwitcher; check if a generic `Dialog` wrapper exists).

### Existing Backend (Tauri — will be extended)
- `src-tauri/src/lib.rs` — Tauri command registration (will add new commands for lorebook CRUD, avatar upload, avatar URL resolution)
- `src-tauri/src/commands/` — Existing CRUD command handlers (will add lorebook commands)
- `src-tauri/src/db/` — Existing DB layer (will add lorebook tables and migration)
- `src-tauri/src/domain/` — Existing Rust domain models (will add Lorebook and LorebookEntry models)

### Archival Reference (gtk_ref — patterns to follow)
- `gtk_ref/src/ui/character_editor.rs` — GTK4 character editor layout patterns (field groupings, wizard-like flow)
- `gtk_ref/src/ui/persona_editor.rs` — GTK4 persona editor layout patterns
- `gtk_ref/src/ui/lorebook_editor.rs` — GTK4 lorebook editor layout patterns (keyword test panel)
- `gtk_ref/src/domain/character.rs` — CharacterCardV2 Rust model with serde defaults
- `gtk_ref/src/domain/persona.rs` — Persona Rust model
- `gtk_ref/src/db/lorebook.rs` — (if exists) Lorebook DB operations
- `gtk_ref/src/db/migrations.rs` — Schema definitions (reference for lorebook table migration)

### Architecture Documentation
- `.planning/codebase/ARCHITECTURE.md` — System overview, layers, data flow
- `.planning/codebase/STRUCTURE.md` — Directory layout, where to add new components
- `.planning/codebase/CONVENTIONS.md` — Naming, code style, component patterns

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **Bits.ui Dialog** — Already used in PersonaSwitcher (`bits-ui/Dialog` or similar primitive). The same primitive will wrap the character/persona/lorebook editors. Will need a generic `Dialog` wrapper component if one doesn't exist.
- **`Avatar` component** — Accepts `src` (string) and `name` (string, for initials fallback). Used in CharacterCard, PersonaCard, MessageBubble. Will display uploaded avatars once URL resolution is wired.
- **`Button`, `Input`, `Tag` components** — Already styled, variant-based, ready for form use inside the editor dialogs.
- **`CharacterCard` and `PersonaCard`** — Already have `onedit` callback props, but the callbacks are currently no-ops (no editor exists). Add `ondelete` props and kebab menu trigger.
- **`AppState`** — Reactive store with `hydrate()` loading data. New actions needed for character/persona/lorebook CRUD to update local state optimistically.
- **`api.ts` typed wrappers** — `save_character`, `save_persona`, `delete_character`, `delete_persona` already exist. Extend with lorebook and avatar commands.

### Established Patterns
- **Svelte 5 runes** — `$state`, `$derived`, `$effect` for local component state. The wizard steps will use `$state` for current step and form field values.
- **Variant-based components** — `Button` and `Avatar` use `tailwind-variants` `tv()`. Editor buttons follow the same pattern.
- **Tauri IPC via invoke()** — Commands registered in `lib.rs`, called from `api.ts`. Frontend types must match Rust serde output (D-08 from Phase 2).
- **Barrel exports** — New components exported from `src/lib/components/characters/index.ts`, `src/lib/components/personas/index.ts`, and `src/lib/components/ui/index.ts`.
- **TypeScript strict mode + svelte-check** — All new components must type-check.

### Integration Points
- **`src/routes/characters/+page.svelte`** — Wire "New character" / "New persona" buttons to open editor dialogs with `mode: 'create'`. Pass character/persona data to edit dialogs.
- **`CharacterCard.onedit`** — Wire to open character editor with `mode: 'edit'` and pre-populated data.
- **`PersonaCard.onedit`** — Wire to open persona editor with pre-populated data.
- **`+layout.svelte`** — Ensure `app.hydrate()` loads lorebooks after this phase. May need to add lorebooks to the hydration batch.
- **`src-tauri/src/lib.rs`** — Register new Tauri commands: `upload_avatar`, `get_avatar_url`, `create_lorebook`, `update_lorebook`, `delete_lorebook`, `get_lorebooks`, `get_lorebook_entries`, `save_lorebook_entry`, `delete_lorebook_entry`, `test_lorebook_keywords`, `assign_lorebook_to_character`, `remove_lorebook_from_character`.
- **`src-tauri/src/db/migrations.rs`** — Add migration to refactor lorebook schema (new `lorebooks` table, change `lorebook_entries.character_id` to `lorebook_id`, add `character_lorebooks` junction).
- **Settings route (future)** — Lorebook list view for management likely lives in `/settings` or a new `/lorebooks` route. To be decided during planning.

### Constraints
- No new npm dependencies unless strictly necessary — all UI primitives should come from Bits.ui or existing Tailwind utilities.
- Dialog z-index must be above `AppHeader` (which may have a drag region in future). Use Bits.ui Dialog's built-in portal layering.
- Avatar upload must happen through a Tauri command — webview cannot directly access the filesystem.
- SQLite schema changes need migration (rusqlite_migration). The existing Phase 2 schema is already migrated; add a new migration for lorebook refactoring.

</code_context>

<specifics>
## Specific Ideas

No specific external references. Key shape:
- Character editor wizard: 4 steps, linear create / free-jump edit, identity → personality → prompt → greeting
- Persona editor: single dialog, all fields visible, name required only
- Lorebook editor: standalone resource, many-to-many with characters, keyword test panel inline
- Avatar: native file picker, asset protocol, auto-cleanup

</specifics>

<deferred>
## Deferred Ideas

- **Drag-and-drop avatar upload** — User mentioned native file picker only. Drag-and-drop is deferred to a future enhancement if requested.
- **Character import/export (PNG card I/O)** — Card import/export is Phase 7 (Rich Output & Cards). The avatar upload path and asset directory may be reused for card image extraction later.
- **Lorebook full-text search** — Search across lorebook entries is part of Phase 8 (Advanced Conversations) with FTS5.
- **Lorebook context injection in prompts** — Lorebook entries injecting into the AI prompt is Phase 9 (Power Features). This phase only covers entry CRUD and keyword testing, not injection.
- **Duplicate/clone character or persona** — Cards have a "Copy" icon button (currently a no-op). Cloning is deferred; if implemented later, consider avatar reference counting to avoid auto-cleanup deleting shared files.

</deferred>

---

*Phase: 03-Editors*
*Context gathered: 2026-05-12*
