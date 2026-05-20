# Phase 9: Power Features - Context

**Gathered:** 2026-05-19
**Status:** Ready for planning

<domain>
## Phase Boundary

Lorebook context injection into AI prompts (keyword matching, recursive activation, depth-based insertion, token budget enforcement) and regex scripting that transforms AI output before display. Also includes assigning lorebooks to conversations, wiring lorebook activation into PromptEngine, wiring regex into the response pipeline, and adding frontend editor/testing UIs for both features.

In scope:
- Port LorebookService (keyword matching, recursive activation, depth insertion, budget truncation) from gtk_ref
- Integrate lorebook activation into existing PromptEngine (replacing Phase 6 stub)
- Conversation-level lorebook assignment (new `conversation_lorebooks` junction + chat header UI)
- Port RegexService (pipe-chain transforms, ReDoS validation, capture groups) from gtk_ref
- Integrate regex scripts into AI response pipeline (apply before display)
- Regex script editor UI (per-character, with inline test panel)
- Lorebook injection visibility in context bar (expandable detail)

NOT in scope: Theme system (Phase 10), spec update/distribution (Phase 11), new character/persona editing (Phase 3), streaming chat (Phase 5), prompt assembly architecture (Phase 6 — extending only).

</domain>

<decisions>
## Implementation Decisions

### Regex Script Scoping
- **D-01:** Regex scripts are per-character (not per-conversation). All conversations with a character use the same regex scripts. Matches gtk_ref model and SillyTavern convention.
- **D-02:** Regex editor lives within the character editor UI. The `regex_scripts` table already exists with `character_id` FK in the tauri backend.

### Conversation Lorebook UX
- **D-03:** Conversation-level lorebook assignments managed via a chat header panel — inline, always visible, next to the persona switcher. All conversation-level config in one spot.
- **D-04:** Lorebook token budget split evenly across assigned lorebooks. No per-lorebook manual budget controls at the conversation level. Within each lorebook, entries with explicit token budgets get their allocation; remaining entries share the rest (gtk_ref pattern).
- **D-05:** New `conversation_lorebooks` junction table needed (mirrors `character_lorebooks` structure). Character lorebooks are the default (always active); conversation lorebooks are additional overlays.

### Lorebook Injection Visibility
- **D-06:** Context bar shows an expandable lorebook detail section: collapsed by default shows "Lorebook: N active", expanded shows each activated entry with name, content preview snippet, and token cost.
- **D-07:** No full prompt inspector panel. Visibility is limited to the context bar detail — enough for debugging without UI overload.

### Regex Editor Preview
- **D-08:** Inline test panel inside the regex script editor — user types/pastes sample AI text and sees live transformed output as they edit the pattern. Consistent with Phase 3's lorebook keyword tester pattern.
- **D-09:** Backend provides `apply_single_regex` command for the preview (port from gtk_ref).

### Agent's Discretion
- Exact layout of the chat header lorebook panel (dropdown, popover, or inline chips)
- Exact layout of the regex editor within the character editor (tab, accordion section, or separate step)
- Error handling when lorebook budget overflows at the conversation level
- How regex scripts interact with streaming responses (apply chunk-by-chunk or on completion)
- Whether the context bar lorebook detail shows entry keys that matched or just entry names

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Reference Implementation (MUST read — port from gtk_ref)
- `gtk_ref/src/service/lorebook.rs` — Complete LorebookService: activate_lorebook_entries() with OR keyword matching, scan depth, recursive activation, token budget truncation, depth-based injection, inject_lorebook_entries(), full test suite
- `gtk_ref/src/service/regex.rs` — Complete RegexService: validate_regex_pattern() with size_limit ReDoS protection, validate_regex_flags(), apply_regex_scripts() pipe-chain, apply_single_regex() for preview, full test suite
- `gtk_ref/src/domain/regex_script.rs` — RegexScript domain model with from_extensions_json()/to_extensions_json() for SillyTavern card compatibility
- `gtk_ref/src/domain/lorebook.rs` — LorebookEntry domain model (gtk_ref version — character_id FK; adapt to tauri version which uses lorebook_id FK)
- `gtk_ref/src/db/regex_scripts.rs` — Regex script DB CRUD, per-character isolation tests
- `gtk_ref/src/db/lorebook.rs` — Lorebook DB layer (reference for patterns)

### Prior Phase Decisions (carry-forward)
- `.planning/phases/06-smart-prompts/06-CONTEXT.md` — PromptEngine architecture: layer system (sticky → lorebook → persona → history → post-history), BudgetInfo, token counting, ChatContext pattern. Lorebook activation is stubbed.
- `.planning/phases/03-editors/03-CONTEXT.md` — Lorebook domain model, DB layer, editor UI, standalone lorebooks, character_lorebooks junction table, keyword testing panel pattern
- `.planning/phases/07-rich-output-cards/07-CONTEXT.md` — Markdown rendering pipeline (marked + dompurify + shiki), conditional rendering during streaming

### Active Backend (will be extended)
- `src-tauri/src/service/prompt.rs` — PromptEngine with TODO Phase 9 stub for lorebook activation
- `src-tauri/src/service/token_count.rs` — TokenCountService with tiktoken-rs cl100k_base, 10% safety margin
- `src-tauri/src/domain/lorebook.rs` — Lorebook + LorebookEntry domain models (lorebook_id FK, LorebookPosition, LorebookActivation)
- `src-tauri/src/db/lorebook.rs` — Lorebook + LorebookEntry CRUD, character_lorebooks junction operations
- `src-tauri/src/db/regex_scripts.rs` — Regex script DB CRUD (already exists)
- `src-tauri/src/commands/chat.rs` — start_chat command, ChatContext + load_chat_context pattern
- `src-tauri/src/commands/keywords.rs` — Keyword test command (Phase 3 — pattern reference)
- `src-tauri/src/commands/lorebooks.rs` — Lorebook CRUD commands

### Active Frontend (will be modified)
- `src/lib/types/index.ts` — TypeScript types (Lorebook, LorebookEntry, LorebookPosition, LorebookActivation — no RegexScript type yet)
- `src/lib/components/lorebooks/LorebookEditor.svelte` — Existing lorebook editor with keyword testing panel (pattern for regex editor)
- `src/lib/components/chat/ChatHeader.svelte` — Chat header with persona switcher (will add lorebook panel)
- `src/lib/components/chat/MessageBubble.svelte` — Message rendering (regex transforms apply before display)
- `src/lib/stores/app.svelte.ts` — AppState (will need regex script state/actions)
- `src/lib/data/api.ts` — Tauri invoke wrappers (will need regex + conversation lorebook commands)

### Project Planning
- `.planning/ROADMAP.md` — Phase 9 definition, success criteria, plan outline
- `.planning/REQUIREMENTS.md` — LORB-01–05, PRMP-04–05, REGX-01–05
- `.planning/STATE.md` — Current progress, accumulated decisions

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- **LorebookEditor keyword tester** (`src/lib/components/lorebooks/LorebookEditor.svelte`): Same pattern should be used for regex test panel — input textarea, live output display, error feedback
- **Lorebook domain + DB layer** (`src-tauri/src/domain/lorebook.rs`, `src-tauri/src/db/lorebook.rs`): Fully implemented from Phase 3. CRUD, junction table, character assignment all done.
- **Regex script DB layer** (`src-tauri/src/db/regex_scripts.rs`): Already exists with CRUD operations. Missing: domain model, service layer, Tauri commands, frontend types.
- **TokenCountService** (`src-tauri/src/service/token_count.rs`): Used by lorebook for per-entry budget truncation. Already available.
- **ChatContext pattern** (`src-tauri/src/commands/chat.rs`): Consolidates DB loading for chat commands. Will need to load conversation lorebooks + regex scripts.
- **character_lorebooks junction** (`src-tauri/src/db/lorebook.rs`): Pattern to mirror for conversation_lorebooks junction.

### Established Patterns
- **gtk_ref port pattern**: Previous phases port gtk_ref services by adapting domain models (e.g., NaiveDateTime → DateTime<Utc>, character_id → lorebook_id) while preserving algorithm logic and test coverage
- **Service module pattern**: Services in `src-tauri/src/service/`, domain models in `src-tauri/src/domain/`, DB in `src-tauri/src/db/`, Tauri commands in `src-tauri/src/commands/`
- **Frontend API pattern**: Tauri invoke wrappers in `src/lib/data/api.ts`, types in `src/lib/types/index.ts`, state in AppState
- **PromptEngine layer system**: Sticky → lorebook → persona → history → post-history. Lorebook injection replaces the Phase 6 stub.

### Integration Points
- **PromptEngine lorebook stub**: `src-tauri/src/service/prompt.rs` Layer 2 — replace TODO with actual activation call
- **AI response pipeline**: After streaming completes, apply regex transforms to the full response before MarkdownRenderer renders it (Phase 7 conditional rendering: raw during streaming, markdown on completion)
- **Chat header**: Add lorebook assignment panel alongside existing persona switcher
- **Character editor**: Add regex script tab/section to the existing character editor wizard
- **Context bar**: Extend the existing token budget bar with expandable lorebook detail

</code_context>

<specifics>
## Specific Ideas

- Regex test panel should feel identical to the lorebook keyword tester — same layout pattern, same live-feedback UX
- Context bar lorebook detail should be compact: entry name + content snippet (first ~30 chars) + token count
- Chat header lorebook panel should show checkboxes for available lorebooks, with the even-split budget displayed

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 09-power-features*
*Context gathered: 2026-05-19*
