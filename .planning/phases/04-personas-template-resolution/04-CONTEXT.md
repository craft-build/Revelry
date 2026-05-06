# Phase 4: Personas & Template Resolution - Context

**Gathered:** 2026-05-05
**Status:** Ready for planning

<domain>
## Phase Boundary

Users can create and assign personas that inject identity into AI prompts, with full template variable resolution — transforming chat into roleplay.

**In scope:**
- Persona domain model with 8 structured fields
- Persona CRUD via multi-step wizard
- Template variable resolution: `{{user}}`, `{{char}}`, `{{persona.field}}`
- Persona injection into AI prompt as persona block
- Assignment hierarchy: global default → per-character default → per-conversation override

**Out of scope (deferred):**
- Persona token compression (Phase 5 / PERS-A02)
- Mid-conversation persona switching (v2 / PERS-A01)
- Persona evolution tracking (v2 / PERS-A05)
- Child persona inheritance (v2 / PERS-A06)
- Dual-persona mode (v2 / PERS-A07)
- `{{char.field}}` expansion beyond `.name` (Phase 5/6)

</domain>

<decisions>
## Implementation Decisions

### Persona Schema & Editor

- **D-01:** Persona editor is a multi-step wizard using AdwPreferencesWindow sub-pages
  - Steps: Basic Info (name, appearance) → Background (background, role) → Personality (personality, speech_style) → Relationships → Review
  - Matches GNOME design patterns and CharacterCardV2 editor conventions
  - Reuses `CharacterEditorInit` enum pattern (New / Edit modes)

- **D-02:** Required fields: `name` and `personality` only
  - All other fields (appearance, background, role_description, relationships, speech_style, tags) are optional
  - Persona can exist as a minimal identity (name + personality) and be filled out over time

- **D-03:** `relationships` field is freeform text (not structured list)
  - Stored as plain TEXT in database, like `personality` and `description` in CharacterCardV2
  - Simpler UI and matches existing character field patterns

- **D-04:** Persona domain model uses UUID v4 ID pattern (same as CharacterCardV2, Conversation)
  - `created_at` and `updated_at` timestamps
  - serde Serialize/Deserialize for potential future export

- **D-05:** Persona data stored in new `personas` database table
  - Migration adds `personas` table with all 8 fields as TEXT columns
  - Migration adds `default_persona_id` to `app_settings` table for global default reference

### Template Variable Resolution

- **D-06:** `{{persona.*}}` uses dot-path syntax: `{{persona.name}}`, `{{persona.appearance}}`, etc.
  - Resolution is simple string replacement (regex or lightweight parser)
  - Field names match domain model field names exactly
  - If persona field is empty, variable resolves to empty string (no placeholder text)

- **D-07:** Template resolution happens at **prompt-assembly-time only**
  - Raw `{{user}}`, `{{char}}`, `{{persona.*}}` remain in stored character data, messages, and exports
  - Only resolved when `PromptEngine::assemble()` builds the prompt for the AI provider
  - Preserves SillyTavern compatibility (they store raw templates too)

- **D-08:** `{{user}}` resolves to active persona's name (or "User" if no active persona)
  - `{{char}}` resolves to character name (unchanged from Phase 3 behavior)
  - `{{char.field}}` expansion is **not implemented** in this phase — belongs in Phase 5/6 when more advanced templates are needed

### Persona Injection Format

- **D-09:** Persona block uses plain text with labeled fields (matching character system prompt style)
  - Format follows the same pattern as `build_system_prompt()` in `src/service/prompt.rs`
  - Introduction line: `You are interacting with {{user}}, who is:`
  - Field lines: `Name: Alice`, `Appearance: Brown hair`, etc.
  - Empty fields are omitted from the prompt (not sent as "Field: ")

- **D-10:** Persona block is injected between character system prompt and conversation history
  - Prompt assembly order: (1) character system prompt (sticky) → (2) persona block (semi-sticky) → (3) conversation history
  - This positions persona data after character definition but before dynamic conversation context

### Persona Assignment UX

- **D-11:** Persona management lives in CharactersPanel via tab switcher
  - Reuses existing "Characters" / "Personas" tab pattern (placeholder already exists)
  - Persona list shows: avatar (placeholder icon), persona name, short personality preview
  - Add/Edit/Delete buttons on the Personas tab

- **D-12:** Global default persona assigned in Settings
  - SettingsPanel adds a "Default Persona" dropdown (populated from persona list)
  - Stored in `app_settings` table as `default_persona_id` key
  - If no global default is set, conversations start with no persona (`{{user}}` = "User")

- **D-13:** Per-character default persona assigned in character editor
  - Character editor's "Basic Info" step adds a "Default Persona" dropdown
  - Stored as `default_persona_id` on the `characters` table (new column via migration)
  - When starting a conversation with this character, this persona is pre-selected

- **D-14:** Per-conversation persona shown/changeable in chat header
  - Chat header (next to character name) shows active persona name in subtitle or adjacent label
  - Click/tap opens popover with persona selection list
  - Changing persona updates `conversation.creator_persona_id` and triggers toast "Persona changed to [Name]"

- **D-15:** Assignment hierarchy resolution at conversation start
  1. Per-conversation override (if set) — highest priority
  2. Per-character default (if set)
  3. Global default (if set)
  4. No persona — `{{user}}` resolves to "User"

### Database Changes

- **D-16:** New migration adds:
  - `personas` table: id, name, appearance, personality, background, role_description, relationships, speech_style, tags, created_at, updated_at
  - `default_persona_id` column to `characters` table (TEXT, nullable, no FK constraint for simplicity)
  - `default_persona_id` app_setting key/value pair support (reuses existing app_settings table)

- **D-17:** DbInput/DbOutput variants added for persona CRUD
  - DbInput: LoadPersonas, SavePersona, DeletePersona
  - DbOutput: PersonasLoaded, PersonaSaved, PersonaDeleted
  - Existing LoadConversations/StartConversation queries join with personas table for name resolution

### Claude's Discretion

- Exact field ordering in persona injection block
- Persona list row styling and avatar handling (placeholder icon vs custom avatar — v2 feature)
- Empty state messaging for "no personas" in assignment dropdowns
- Toast messages and timing
- Migration numbering (use Migration 7 or later, sequential after existing 6)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope & Requirements
- `.planning/REQUIREMENTS.md` — PERS-01 through PERS-06, CHAR-08
- `.planning/ROADMAP.md` §Phase 4 — Goal, success criteria, and dependency chain
- `.planning/PROJECT.md` — Core value, constraints, persona-related active requirements

### Existing Domain & Database
- `src/domain/character.rs` — CharacterCardV2 model (fields referenced in template resolution)
- `src/domain/conversation.rs` — Conversation model with `creator_persona_id` field (Phase 3, D-01)
- `src/db/mod.rs` — DbInput/DbOutput enums (extend with persona variants)
- `src/db/migrations.rs` — Migration pattern (Migrations + M, current max is M6)
- `src/db/conversations.rs` — Conversation CRUD (needs persona-aware queries)
- `src/db/models.rs` — Row conversion helpers (pattern for new persona model)

### Prompt & AI Services
- `src/service/prompt.rs` — PromptEngine::assemble() and build_system_prompt() (integration point for persona block injection)
- `src/service/ai.rs` — AiService streaming integration (persona integrated into prompt before streaming)

### UI Patterns
- `src/ui/characters.rs` — CharactersPanel with tab switcher (reuse for Personas tab)
- `src/ui/character_editor.rs` — CharacterEditor pattern (multi-step form, field editors)
- `src/ui/settings/mod.rs` — SettingsPanel pattern (add Default Persona dropdown)
- `src/ui/chat/mod.rs` — ChatPanel header area (add persona display/selector popover)
- `src/app.rs` — AppModel message routing (bridge persona DbOutput to UI components)

### Design Spec
- `docs/Revelry_Design_Specification_v2.0.docx` — §Personas, §Template Resolution, §Prompt Assembly

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **DbService Worker** (`src/db/mod.rs`): Serial background thread pattern ready for new persona CRUD operations. Add LoadPersonas/SavePersona/DeletePersona to DbInput.
- **CharactersPanel tab switcher** (`src/ui/characters.rs`): Already has "Characters" / "Personas" placeholder. Replace personas placeholder with actual persona list/grid.
- **CharacterEditor pattern** (`src/ui/character_editor.rs`): Multi-field form with text buffers, enum-based init (New/Edit), and save/cancel flow. Clone and adapt for PersonaEditor.
- **AppModel bridging** (`src/app.rs`): Existing pattern routes DbOutput → component messages. Add persona variants to AppMsg.
- **PromptEngine** (`src/service/prompt.rs`): `build_system_prompt()` builds character prompt block. Add `build_persona_block()` and inject between system prompt and chat history in `assemble()`.

### Established Patterns
- **Database:** UUID v4 IDs, NaiveDateTime as TEXT, prepare_cached() for CRUD, upsert via exists-check-then-insert-or-update (from characters.rs).
- **UI Components:** Component trait for components with async operations. AppModel bridges between UI components and DbInput via output messages.
- **Migrations:** rusqlite_migration with M::up()/M::down() per migration. Sequential numbering from M1 to M6 currently.
- **Settings storage:** app_settings key/value TEXT table for global default persona ID.

### Integration Points
- **DbService** needs new DbInput variants (LoadPersonas, SavePersona, DeletePersona) and DbOutput variants (PersonasLoaded, PersonaSaved, PersonaDeleted)
- **Conversation queries** should optionally join with personas table to resolve persona name for sidebar/chat header display
- **PromptEngine::assemble()** needs persona parameter and injection logic between system_prompt and chat_history
- **ChatPanel header** needs persona label + popover (character name is already displayed there)
- **Character editor** needs "Default Persona" dropdown in Basic Info step
- **SettingsPanel** needs "Default Persona" dropdown in preferences

</code_context>

<specifics>
## Specific Ideas

- Multi-step wizard follows GNOME AdwPreferencesWindow sub-page pattern (not a separate window)
- Persona list should feel like the character library grid but simpler (no avatars yet — placeholder icon)
- "You are interacting with {{user}}, who is:" introduction style came from roleplay-focused framing
- Reuse existing character editor patterns rather than inventing new UI paradigms
- Keep persona injection format identical to character system prompt style for consistency

</specifics>

<deferred>
## Deferred Ideas

- Persona token compression (full → trimmed → minimal → essential) → Phase 5 (Context Engineering)
- Mid-conversation persona switching with retroactive branch regeneration → v2 (PERS-A01)
- Persona evolution tracking with timestamped snapshots → v2 (PERS-A05)
- Child persona inheritance with field-level fallback → v2 (PERS-A06)
- Dual-persona mode for two identities in one conversation → v2 (PERS-A07)
- `{{char.field}}` template expansion beyond `{{char}}` → Phase 5/6 (when advanced template/lorebook system exists)
- Persona avatars → v2 (reuse character avatar pattern)
- Persona import/export → Phase 6 (Data Management)

</deferred>

---

*Phase: 04-personas-template-resolution*
*Context gathered: 2026-05-05*
