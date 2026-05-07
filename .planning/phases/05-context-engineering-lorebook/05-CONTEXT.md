# Phase 5: Context Engineering & Lorebook - Context

**Gathered:** 2026-05-07
**Status:** Ready for planning

<domain>
## Phase Boundary

Token budgeting and lorebook system ensure long conversations stay coherent with proper context management — the intelligence layer that keeps characters in character. Covers: token counting with budgeting, prioritized prompt assembly with truncation, persona compression, keyword-activated lorebook with recursive activation, and context utilization UI indicator.

**In scope:**
- Token counting via tiktoken-rs with safety margin for non-OpenAI providers
- Per-model context window size resolution (provider API query → user override → built-in defaults)
- Count-as-you-build prompt assembly with budget enforcement
- Priority layer system: sticky (system+character) → activated lorebooks → semi-sticky persona → truncated history → post_history_instructions
- 4-tier persona compression (full → trimmed → minimal → essential)
- Lorebook entries per character with keywords, secondary terms, insertion position, depth, activation state, token budget
- Keyword matching (OR logic) with configurable scan depth
- Recursive lorebook activation with iterative max depth counter
- Per-entry soft token budgets with overflow to next entry
- Context utilization indicator (thin progress bar in input bar)

**Out of scope (deferred):**
- Persona-aware lorebook activation (v2 / CCTX-A01)
- {{char.field}} expansion beyond {{char}} → Phase 6
- Per-provider tokenizer dispatch → future polish
- History summarization on truncation → future enhancement

</domain>

<decisions>
## Implementation Decisions

### Token Counting & Budgeting

- **D-01:** Count-as-you-build token counting in PromptEngine
  - After each prompt layer is added, count tokens against remaining budget
  - If budget exceeded, triggers truncation/compression before adding more
  - Most accurate approach — catches overflows before sending to provider

- **D-02:** Context window size resolution via 3-tier fallback chain
  - Primary: query provider API for model capabilities at runtime
  - Secondary: user-configured override in provider settings
  - Tertiary: built-in default table (GPT-4o=128k, Claude 3.5=200k, Llama 3=8k, etc.)
  - Falls back to 8k default for completely unknown models
  - ProviderConfig domain model needs context_window_size field (user override)

- **D-03:** tiktoken-rs BPE tokenization with ~10% safety margin for all providers
  - Single tokenizer for simplicity — tiktoken is close enough for non-OpenAI providers
  - Safety margin compensates for tokenizer differences across providers
  - No per-provider tokenizer dispatch in v1

### Context Assembly & Truncation

- **D-04:** Prompt assembly priority layer order (CCTX-01):
  1. System prompt + character card fields (sticky — never truncated)
  2. Activated lorebook entries (injected at their configured depth)
  3. Persona block (semi-sticky — compressed when budget is tight)
  4. Conversation history (volatile — oldest truncated first)
  5. Post-history instructions (after history, per CharacterCardV2 spec intent)

- **D-05:** Oldest-first history truncation
  - Remove oldest user/assistant message pairs first when history exceeds budget
  - Preserves recent context — standard approach matching SillyTavern behavior

- **D-06:** 4-tier progressive persona compression when budget is tight
  - Full: all fields (name, personality, appearance, background, role, relationships, speech_style)
  - Trimmed: name, personality, appearance only
  - Minimal: name + personality only
  - Essential: just name
  - Drops fields in priority order as context budget shrinks

- **D-07:** Silent indicator only for truncation notification
  - No toast or dialog when truncation occurs
  - Context utilization indicator changes color to show budget pressure
  - User can see truncation state by glancing at the indicator

### Lorebook Data Model & Activation

- **D-08:** Separate `lorebook_entries` database table (not JSON in character_book column)
  - Per-entry fields: id, character_id, name, keywords (comma-separated or JSON array), secondary_terms, content, insertion_position (depth from bottom), activation_depth, enabled, token_budget, position (before/after char definition), sort_order, created_at, updated_at
  - character_book JSON column preserved for import/export compatibility — populated from table data on export, parsed on import

- **D-09:** Keyword matching with OR logic
  - Entry activates if ANY of its keywords appear in recent messages
  - Simpler, matches SillyTavern default, good for broad world-building entries

- **D-10:** Configurable scan depth per character
  - User sets how many recent messages to scan for keywords (default: 2-3)
  - Per-character setting stored in characters table or character_book metadata
  - Matches SillyTavern behavior

- **D-11:** Depth-from-bottom insertion position (SillyTavern style)
  - Entry at depth 0 injects at the very end (after last message)
  - Depth 1 = one message from end, etc.
  - Full SillyTavern character_book spec compatibility

- **D-12:** Recursive activation with iterative max depth counter
  - After initial keyword scan, check if newly-activated entries' content contains other entries' keywords
  - Repeat up to N iterations (configurable, default 2)
  - Prevents infinite loops via max depth counter

- **D-13:** Per-entry soft token budgets with overflow
  - Each entry has an optional token_budget field
  - If entry exceeds its budget, content is truncated from the end
  - Entries without explicit budgets share the remaining lorebook allocation
  - Overflow: if an entry is under budget, remaining tokens available for subsequent entries

### Context Utilization UI

- **D-14:** Context utilization indicator lives in the input bar
  - Thin progress bar below/above the text input area
  - Always visible when chatting — near where the user types

- **D-15:** Thin progress bar with color coding
  - Green: < 70% context used
  - Yellow: 70-90% context used
  - Red: > 90% context used
  - Compact, glanceable, matches GNOME design patterns

- **D-16:** Update on every prompt assembly
  - Recalculate after each message send, AI response, persona change, or lorebook activation
  - Always reflects the actual prompt that would be sent to the provider

### the agent's Discretion

- Exact tiktoken-rs model selection (cl100k_base vs o200k_base) — default to cl100k_base for broadest compatibility
- Built-in context window size table contents (which models to include, exact sizes)
- Lorebook entry field naming and types in the database schema
- How character_book JSON import/export serialization maps to/from the entries table
- Scan depth default value (2 or 3)
- Recursive activation default max depth (2 or 3)
- Safety margin percentage (10% or other)
- Progress bar exact height and styling
- Migration numbering (next after M7)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope & Requirements
- `.planning/REQUIREMENTS.md` — CCTX-01 through CCTX-07 (token budgeting, context assembly, utilization indicator, lorebook management, keyword activation, recursive activation, entry token budgeting)
- `.planning/ROADMAP.md` §Phase 5 — Goal, success criteria, and dependency chain
- `.planning/PROJECT.md` — Core value, constraints, active requirements (token counting, lorebook, context engineering)

### Existing Prompt & AI Services
- `src/service/prompt.rs` — PromptEngine::assemble() and build_system_prompt_with_persona() (integration point for budgeting, lorebook injection, persona compression)
- `src/service/template.rs` — build_persona_block() and resolve_variables() (persona compression needs to modify persona block generation)
- `src/service/ai.rs` — AiService::stream_chat() (needs context window size awareness for budgeting)

### Domain & Database
- `src/domain/character.rs` — CharacterCardV2 model (character_book field, post_history_instructions field)
- `src/domain/conversation.rs` — Message and Conversation models (used in token counting)
- `src/domain/persona.rs` — Persona model (used in persona compression)
- `src/domain/provider.rs` — ProviderConfig (needs context_window_size field)
- `src/db/mod.rs` — DbService Worker, DbInput/DbOutput enums (add lorebook CRUD variants)
- `src/db/migrations.rs` — Migration pattern (currently M1-M7, add M8+ for lorebook_entries table and schema changes)
- `src/db/models.rs` — Row conversion helpers (pattern for new lorebook model)

### UI Integration
- `src/ui/chat/mod.rs` — ChatPanel with input bar (context utilization indicator placement)
- `src/ui/chat/input_bar.rs` — InputBar component (add progress bar widget)
- `src/ui/character_editor.rs` — CharacterEditor pattern (add lorebook tab/editor)
- `src/ui/characters.rs` — CharactersPanel (existing tab switcher pattern)

### Prior Phase Context
- `.planning/phases/04-personas-template-resolution/04-CONTEXT.md` — Persona injection format, template resolution, deferred persona compression
- `.planning/phases/03-ai-chat-streaming/03-CONTEXT.md` — Prompt assembly baseline, deferred token counting
- `.planning/phases/01-app-shell-provider-foundation/01-CONTEXT.md` — Provider config patterns, keyring storage

### Technology Stack
- `.planning/research/STACK.md` — tiktoken-rs 0.11.0 for BPE tokenization, regex 1.12.3 for keyword matching

### Design Specification
- `docs/Revelry_Design_Specification_v2.0.docx` — §Context Engineering, §Lorebook System, §Prompt Assembly

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **PromptEngine** (`src/service/prompt.rs`): `assemble()` and `assemble_initial()` are the integration points for token counting, lorebook injection, and budget enforcement. Currently builds preamble + chat_history without any budgeting.
- **build_persona_block()** (`src/service/template.rs`): Generates persona injection block. Needs 4-tier compression logic added.
- **DbService Worker** (`src/db/mod.rs`): Serial background thread pattern ready for lorebook CRUD. Add LoadLorebookEntries/SaveLorebookEntry/DeleteLorebookEntry to DbInput.
- **CharactersPanel tab switcher** (`src/ui/characters.rs`): Could add lorebook management as another tab or integrate into character editor.
- **InputBar** (`src/ui/chat/input_bar.rs`): Add context utilization progress bar to this component.
- **character_book column** (`src/db/migrations.rs` M4): Already exists on characters table as TEXT column. Used for import/export JSON serialization; entries stored in separate table for CRUD.

### Established Patterns
- **Database:** UUID v4 IDs, NaiveDateTime as TEXT, prepare_cached() for CRUD, upsert via exists-check-then-insert-or-update. Migrations via rusqlite_migration M::up()/M::down().
- **UI Components:** Component trait for components with async operations. AppModel bridges between UI components and DbInput via output messages.
- **Service layer:** PromptEngine for prompt assembly, AiService for streaming. Services are stateless — data flows in, AssembledPrompt flows out.
- **Token counting:** New capability — no existing pattern. tiktoken-rs crate recommended in STACK.md.

### Integration Points
- **PromptEngine::assemble()** needs: token counting service, lorebook activation, persona compression, and budget enforcement
- **AiService::stream_chat()** needs: context window size from provider for budget calculation
- **ProviderConfig** needs: context_window_size field (user override, nullable)
- **DbService** needs: LoadLorebookEntries, SaveLorebookEntry, DeleteLorebookEntry, and activation query variants
- **ChatPanel** needs: context utilization indicator widget, budget status updates
- **InputBar** needs: thin progress bar for context utilization
- **Character editor** needs: lorebook entry management tab or sub-section

</code_context>

<specifics>
## Specific Ideas

- SillyTavern character_book spec format should be supported for import/export compatibility (entries serialized to/from character_book JSON column)
- Depth-from-bottom insertion position follows SillyTavern conventions — important for community card compatibility
- Context utilization indicator should feel natural and unobtrusive — like a battery indicator for context

</specifics>

<deferred>
## Deferred Ideas

- Persona-aware lorebook activation (entries activate based on active persona data) → v2 (CCTX-A01)
- {{char.field}} template expansion beyond {{char}} → Phase 6 or later
- Per-provider tokenizer dispatch → future polish
- History summarization on truncation → future enhancement (would need LLM call)
- Keyword matching with AND logic or configurable per entry → future enhancement
- Detailed context breakdown popover (showing per-layer token usage) → future polish

</deferred>

---

*Phase: 05-context-engineering-lorebook*
*Context gathered: 2026-05-07*
