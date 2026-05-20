# Phase 9: Power Features — Research

**Gathered:** 2026-05-19
**Status:** Complete

---

## Research Question

**"What do I need to know to PLAN this phase well?"**

---

## 1. Reference Implementation Analysis

### 1.1 LorebookService (`gtk_ref/src/service/lorebook.rs`)

The gtk_ref provides a complete, tested LorebookService with two public functions:

**`activate_lorebook_entries()`** — The core algorithm:

1. Filter to enabled entries only
2. Take last `scan_depth` messages (reversed, newest first) as scan scope
3. Concatenate scan message content (lowercased) into scan text
4. Initial keyword scan: OR logic, case-insensitive — ANY keyword or secondary_term in scan text activates the entry
5. Recursive activation: iterate up to `max_recursive_depth` times; each iteration checks whether content of already-activated entries contains keywords of not-yet-activated entries; stops early if no new activations
6. Sort activated entries by `sort_order`
7. Token budget enforcement: entries with explicit `token_budget` get truncated (binary search truncation from end); entries without explicit budget share remaining lorebook allocation; entries are skipped when remaining budget hits zero

**`inject_lorebook_entries()`** — Insertion into prompt:

- Depth 0 entries: append to preamble (after character definition)
- Depth > 0 entries: insert as system messages into chat history at N messages from end
- Depth insertions are processed from highest depth to lowest to preserve indices

**Key `ActivatedEntry` struct** carries the original entry, truncated content, and effective depth.

**Domain model differences gtk_ref vs. tauri:**
| Field | gtk_ref | tauri (active) |
|-------|---------|----------------|
| FK | `character_id` | `lorebook_id` |
| Keywords | `keywords` | `keys` |
| Secondary | `secondary_terms` | `secondary_keys` |
| Position | `InsertionPosition` (BeforeChar/AfterChar) | `LorebookPosition` (BeforeChar/AfterChar/AtDepth) |
| Depth | `depth_from_bottom` | `depth` |
| Enabled | `enabled` | `activation_state` (Always/CharacterOnly/UserOnly) — no enabled bool |
| Budget | `token_budget: Option<i64>` | Not present |
| Sort | `sort_order` | Not present |

**Porting adaptations needed:**

- Map `keys` → `keywords`, `secondary_keys` → `secondary_terms`
- Add `token_budget` and `sort_order` fields to tauri `LorebookEntry` domain model
- Add `enabled` bool (or use `activation_state == Always` as a proxy — but `activation_state` is richer: CharacterOnly/UserOnly filter by role)
- Handle `LorebookPosition::AtDepth` (gtk_ref doesn't have this enum variant, uses `InsertionPosition`)
- `LorebookActivation` (Always/CharacterOnly/UserOnly) is an enhancement over gtk_ref's simple `enabled` bool — activation should respect role filtering when `activation_state != Always`

### 1.2 RegexService (`gtk_ref/src/service/regex.rs`)

Three functions:

**`validate_regex_pattern()`** — Compiles pattern with `RegexBuilder::size_limit(1_000_000)` for ReDoS protection. Applies flags `i`, `m`, `s` to builder. Returns compiled `Regex` or error string. Note: `g` flag is NOT a compilation flag — it's handled at replacement time.

**`validate_regex_flags()`** — Only `i`, `g`, `m`, `s` allowed.

**`apply_regex_scripts()`** — Pipe chain: sort by `sort_order`, filter to `enabled`, each script's output becomes the next script's input. Invalid patterns are skipped at runtime (with `tracing::warn`).

**`apply_single_regex()`** — For test panel preview. Validates pattern, applies single find/replace.

**Domain model (`gtk_ref/src/domain/regex_script.rs`):**

- `RegexScript` struct with `id`, `character_id`, `name`, `find_pattern`, `replace_pattern`, `flags`, `sort_order`, `enabled`, timestamps
- `from_extensions_json()` / `to_extensions_json()` for SillyTavern card compatibility
- Defaults: flags="gi", sort_order=0, enabled=true

**DB layer:** `src-tauri/src/db/regex_scripts.rs` is a stub (empty module). The `regex_scripts` table schema exists in migrations (from Phase 3) but no CRUD operations or domain model exist in the tauri codebase.

---

## 2. Integration Points

### 2.1 PromptEngine (`src-tauri/src/service/prompt.rs`)

The `assemble_with_budget()` method has a clear Phase 9 stub at Layer 2:

```rust
// ── Layer 2: Lorebook layer (Phase 9 — skipped for now) ──
// TODO: Phase 9 — activate lorebook entries with lorebook::activate_lorebook_entries()
let active_lorebook_count = 0;
let _ = lorebook_entries; // Suppress unused warning
```

The `lorebook_entries: &[LorebookEntry]` parameter is already accepted but unused. The `AssembledPrompt` struct already has `active_lorebook_count: usize`.

**Integration plan:**

- Replace the stub with: (a) activate lorebook entries via `lorebook::activate_lorebook_entries()`, (b) inject into preamble/messages via `lorebook::inject_lorebook_entries()`, (c) count activated entries, (d) add activated entries' tokens to `tokens_used`
- Need to load character lorebooks AND conversation lorebooks, merge their entries, then activate
- Character's `lorebook_scan_depth` and `lorebook_recursive_depth` fields exist on `CharacterCardV2` (verified in frontend types)

### 2.2 ChatContext / `load_chat_context()` (`src-tauri/src/commands/chat.rs`)

Currently loads: conversation, messages, character, persona, provider, API key.

**Phase 9 additions needed:**

- Load lorebook entries for all character-assigned lorebooks
- Load lorebook entries for all conversation-assigned lorebooks
- Load regex scripts for the character
- Add these to `ChatContext` struct

### 2.3 AI Response Pipeline (regex transforms)

Currently: streaming tokens → `StreamChunk::Complete` with full content → frontend saves message → `MarkdownRenderer` renders it.

**Regex integration point:** Apply `apply_regex_scripts()` to the completed AI response content BEFORE:

1. Sending it in `StreamChunk::Complete`
2. Saving it to the database
3. The frontend renders it

This is in the `AiService::stream_chat()` path. The completed content should be transformed by the character's regex scripts before the `Complete` chunk is sent.

**Key decision (per D-08/D-09):** Regex transforms apply to the full response on completion, NOT during streaming. During streaming, raw text displays (consistent with Phase 7's conditional rendering). This avoids the complexity of applying regex to partial content and matches the existing streaming→markdown transition pattern.

### 2.4 ChatHeader (`src/lib/components/chat/ChatHeader.svelte`)

Currently has: character avatar+name, PersonaSwitcher, ProviderModelSelector, Branch switcher.

**Phase 9 addition (D-03):** Conversation-level lorebook assignment panel. This is a new section in the header, next to the persona switcher, showing checkboxes for available lorebooks. Character lorebooks are always active; conversation lorebooks are additional overlays that can be toggled.

### 2.5 MessageBubble (`src/lib/components/chat/MessageBubble.svelte`)

Currently uses `MarkdownRenderer` for completed AI messages. Regex transforms happen BEFORE this rendering — the `content` field of the message will already contain the regex-transformed text. No MessageBubble changes needed for regex (transforms happen server-side before content reaches the frontend).

### 2.6 Context Bar (`MessageInput.svelte`)

Currently shows: `tokens_used / tokens_limit` with color-coded progress bar.

**Phase 9 addition (D-06/D-07):** Expandable lorebook detail section below the bar. Collapsed: "Lorebook: N active". Expanded: each activated entry with name, content snippet (~30 chars), token cost.

### 2.7 AppState (`src/lib/stores/app.svelte.ts`)

**Phase 9 additions needed:**

- Regex script state (per-character, loaded when opening a conversation)
- Conversation lorebook assignments (loaded when opening a conversation)
- Active lorebook entries detail (for context bar expansion)
- Actions: assign/remove conversation lorebook, CRUD regex scripts, test regex

---

## 3. Schema & DB Gaps

### 3.1 `conversation_lorebooks` Junction Table (NEW)

Does NOT exist yet. Needs creation via migration. Mirrors `character_lorebooks`:

```sql
CREATE TABLE conversation_lorebooks (
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    lorebook_id TEXT NOT NULL REFERENCES lorebooks(id) ON DELETE CASCADE,
    sort_order INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (conversation_id, lorebook_id)
);
CREATE INDEX idx_conversation_lorebooks_conv ON conversation_lorebooks(conversation_id, sort_order);
```

### 3.2 `regex_scripts` Table (EXISTS but unused)

Schema already in migrations from Phase 3:

- `id TEXT PRIMARY KEY`
- `character_id TEXT NOT NULL REFERENCES characters(id) ON DELETE CASCADE`
- `name TEXT NOT NULL DEFAULT ''`
- `find_pattern TEXT NOT NULL DEFAULT ''`
- `replace_pattern TEXT NOT NULL DEFAULT ''`
- `flags TEXT NOT NULL DEFAULT 'gi'`
- `sort_order INTEGER NOT NULL DEFAULT 0`
- `enabled INTEGER NOT NULL DEFAULT 1`
- `created_at TEXT NOT NULL`
- `updated_at TEXT NOT NULL`

**Missing:** Domain model (`RegexScript` struct), DB CRUD functions, DbCommand variants, Tauri commands, frontend types, API wrappers.

### 3.3 `lorebook_entries` Table Enhancement

The tauri `LorebookEntry` domain model is missing fields that gtk_ref has:

- `token_budget: Option<i64>` — per-entry soft token limit
- `sort_order: i32` — display/activation ordering
- `enabled: bool` — activation toggle (or map `activation_state` to this)

The DB `lorebook_entries` table also likely needs columns for `token_budget` and `sort_order` (and possibly `enabled`). **This requires a migration.**

Current `lorebook_entries` columns (from migrations): `id, lorebook_id, keys, secondary_keys, content, position, depth, activation_state, extensions, created_at, updated_at`.

**Missing columns:** `token_budget INTEGER`, `sort_order INTEGER DEFAULT 0`, `enabled INTEGER DEFAULT 1`.

---

## 4. Key Adaptations & Decisions Required

### 4.1 LorebookActivation vs. enabled (gtk_ref → tauri)

gtk_ref uses a simple `enabled: bool`. Tauri has `activation_state: LorebookActivation` (Always/CharacterOnly/UserOnly). The activation logic must:

- Treat `Always` as always-eligible
- Only activate `CharacterOnly` entries when scanning character messages
- Only activate `UserOnly` entries when scanning user messages
- This is an **enhancement** over gtk_ref that should be preserved

**Implementation:** The scan function needs to check message roles against `activation_state` per entry, OR concatenate only messages of matching roles into the scan text per entry.

### 4.2 LorebookPosition::AtDepth

gtk_ref uses `InsertionPosition` (BeforeChar/AfterChar) with a separate `depth_from_bottom` field. Tauri has `LorebookPosition::AtDepth` plus `depth` field.

**Mapping:**

- `BeforeChar` → `LorebookPosition::BeforeChar` (injected before character definition in preamble)
- `AfterChar` → `LorebookPosition::AfterChar` (injected after character definition in preamble)
- `AtDepth` → uses `depth` field for depth-from-bottom insertion into history

This maps cleanly. The `inject_lorebook_entries()` function from gtk_ref already separates depth-0 (preamble) from depth>0 (history injection). We need to also handle `BeforeChar` placement.

### 4.3 Budget Split for Conversation Lorebooks (D-04)

Decision: "Lorebook token budget split evenly across assigned lorebooks." The PromptEngine needs:

1. Determine total lorebook budget (a configurable portion of the context window, or a fixed allocation)
2. Split evenly across all assigned lorebooks (character + conversation)
3. Within each lorebook, entries with explicit budgets get their allocation; remaining entries share the rest

**Open question:** What is the "lorebook token budget" as a portion of the context window? This isn't specified in CONTEXT. The gtk_ref `activate_lorebook_entries()` takes `lorebook_token_budget: i64` as a parameter — the caller decides. Likely this should be `context_window * LOREBOOK_BUDGET_FRACTION` (e.g., 20-30%) or a character-level setting. The planner should decide this.

### 4.4 Regex + Streaming Interaction

Decision: regex applies on completion, not during streaming. This means:

- During streaming: raw content displays (Phase 7 pattern)
- On `StreamChunk::Complete`: backend applies regex transforms before sending the completed content
- Frontend receives already-transformed content
- The `StreamChunk::Complete.data.content` field contains the regex-transformed text
- The database stores the regex-transformed text (not the raw AI output)

**Alternative considered:** Store raw output and transform on display. This is more complex and doesn't match gtk_ref behavior (which transforms before storage). Storing transformed content is simpler and matches the existing save-on-complete pattern.

### 4.5 `apply_single_regex` Command for Preview

Per D-08/D-09: Backend provides `apply_single_regex` command. This is a direct port from gtk_ref's `apply_single_regex()` — takes `text, pattern, replace, flags`, validates, applies, returns result or error.

---

## 5. Full File Change Map

### Backend (Rust / Tauri)

| File                                   | Change                                                                                                                                                                                                                        | Plan                |
| -------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------- |
| `src-tauri/src/domain/lorebook.rs`     | Add `token_budget`, `sort_order`, `enabled` fields to `LorebookEntry`                                                                                                                                                         | 09-01               |
| `src-tauri/src/db/migrations.rs`       | Add migration: `conversation_lorebooks` table, `token_budget`/`sort_order`/`enabled` columns on `lorebook_entries`                                                                                                            | 09-01               |
| `src-tauri/src/db/lorebook.rs`         | Add CRUD for `conversation_lorebooks` junction + update entry queries for new fields                                                                                                                                          | 09-01, 09-05        |
| `src-tauri/src/db/regex_scripts.rs`    | Implement full CRUD (currently stub)                                                                                                                                                                                          | 09-03               |
| `src-tauri/src/db/models.rs`           | Update `row_to_lorebook_entry()` for new fields                                                                                                                                                                               | 09-01               |
| `src-tauri/src/db/mod.rs`              | Add `DbCommand` variants: `LoadRegexScripts`, `SaveRegexScript`, `DeleteRegexScript`, `AssignLorebookToConversation`, `RemoveLorebookFromConversation`, `LoadConversationLorebooks`, `LoadLorebookEntriesForLorebooks` (bulk) | 09-01, 09-03, 09-05 |
| `src-tauri/src/service/lorebook.rs`    | **NEW FILE** — Port `activate_lorebook_entries()`, `inject_lorebook_entries()`, `truncate_to_token_budget()`, `keyword_matches()`, `ActivatedEntry` struct. Adapt to tauri domain model differences.                          | 09-01               |
| `src-tauri/src/service/regex.rs`       | **NEW FILE** — Port `validate_regex_pattern()`, `validate_regex_flags()`, `apply_regex_scripts()`, `apply_single_regex()`                                                                                                     | 09-03               |
| `src-tauri/src/domain/regex_script.rs` | **NEW FILE** — Port `RegexScript` struct, `from_extensions_json()`, `to_extensions_json()`                                                                                                                                    | 09-03               |
| `src-tauri/src/service/prompt.rs`      | Replace Layer 2 stub with actual lorebook activation + injection                                                                                                                                                              | 09-02               |
| `src-tauri/src/commands/chat.rs`       | Add lorebook entries + regex scripts to `ChatContext`; apply regex in stream completion path                                                                                                                                  | 09-02, 09-04        |
| `src-tauri/src/commands/lorebooks.rs`  | Add conversation-lorebook junction commands                                                                                                                                                                                   | 09-05               |
| `src-tauri/src/commands/regex.rs`      | **NEW FILE** — CRUD + `apply_single_regex` + `validate_regex_pattern` commands                                                                                                                                                | 09-03, 09-05        |
| `src-tauri/src/commands/mod.rs`        | Register new command modules                                                                                                                                                                                                  | 09-03, 09-05        |
| `src-tauri/src/main.rs`                | Register new Tauri commands                                                                                                                                                                                                   | 09-03, 09-05        |

### Frontend (Svelte)

| File                                                     | Change                                                                                                      | Plan         |
| -------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------- | ------------ |
| `src/lib/types/index.ts`                                 | Add `RegexScript` type, update `LorebookEntry` with `token_budget`, `sort_order`, `enabled`                 | 09-01, 09-03 |
| `src/lib/data/api.ts`                                    | Add API wrappers for regex CRUD, conversation-lorebook junction, `applySingleRegex`, `validateRegexPattern` | 09-03, 09-05 |
| `src/lib/stores/app.svelte.ts`                           | Add regex scripts state, conversation lorebooks state, active lorebook entries detail, actions              | 09-04, 09-05 |
| `src/lib/components/chat/ChatHeader.svelte`              | Add conversation lorebook assignment panel (checkboxes next to persona switcher)                            | 09-05        |
| `src/lib/components/chat/MessageInput.svelte`            | Add expandable lorebook detail section to context bar                                                       | 09-05        |
| `src/lib/components/characters/RegexScriptEditor.svelte` | **NEW FILE** — Regex script editor with inline test panel (mirrors LorebookEditor pattern)                  | 09-05        |

---

## 6. Requirement Coverage Check

| ID      | Requirement                                                         | Covered By                                                 | Status |
| ------- | ------------------------------------------------------------------- | ---------------------------------------------------------- | ------ |
| LORB-01 | Create lorebook entries with keywords, content, activation strategy | Phase 3 (done) + 09-01 (activation service)                | ✅     |
| LORB-02 | Edit lorebook entries                                               | Phase 3 (done)                                             | ✅     |
| LORB-03 | Delete lorebook entries                                             | Phase 3 (done)                                             | ✅     |
| LORB-04 | Assign lorebooks to characters and conversations                    | 09-05 (conversation_lorebooks junction + UI)               | ⬜     |
| LORB-05 | Set ordering and token budget allocation for entries                | 09-01 (token_budget + sort_order fields) + 09-05 (UI)      | ⬜     |
| PRMP-04 | Lorebook entries inject into prompt when keywords match             | 09-01 (LorebookService) + 09-02 (PromptEngine integration) | ⬜     |
| PRMP-05 | Lorebook injection is recursive with depth limiting                 | 09-01 (recursive activation in LorebookService)            | ⬜     |
| REGX-01 | Create regex scripts with find/replace, enable/disable toggles      | 09-03 (RegexService) + 09-05 (editor UI)                   | ⬜     |
| REGX-02 | Edit regex scripts                                                  | 09-03 + 09-05                                              | ⬜     |
| REGX-03 | Delete regex scripts                                                | 09-03 + 09-05                                              | ⬜     |
| REGX-04 | Regex scripts applied in order to AI output before display          | 09-04 (pipeline integration)                               | ⬜     |
| REGX-05 | Invalid regex patterns rejected on save with clear error            | 09-03 (validate_regex_pattern) + 09-05 (UI error display)  | ⬜     |

---

## 7. Risks & Open Questions

### Risks

1. **LorebookEntry schema migration** — Adding `token_budget`, `sort_order`, `enabled` columns to existing `lorebook_entries` table requires a careful migration. Existing entries will have NULL/default values. Test coverage must verify migration correctness.

2. **Budget split ambiguity** — The "lorebook token budget" as a fraction of context window is not specified. The planner must decide: fixed fraction? Character-level setting? User-level setting? The simplest approach is a constant (e.g., 25% of context window) with a future character-level override.

3. **Activation state filtering** — `LorebookActivation::CharacterOnly` and `UserOnly` are enhancements over gtk_ref's simple `enabled`. The activation logic must scan only messages of the matching role, which requires a more nuanced scan than gtk_ref's simple text concatenation.

4. **Regex during streaming** — If regex transforms are only applied on completion, there's a visual discontinuity: raw streaming text suddenly transforms. This is acceptable per Phase 7's conditional rendering pattern, but worth noting.

5. **ChatContext loading cost** — Adding lorebook entry loading + regex script loading to `load_chat_context()` adds DB round trips. The existing pattern already has 6 sequential loads; adding 2-3 more maintains the pattern but increases latency. Could be parallelized if needed.

### Open Questions for Planning

1. **Lorebook budget fraction**: What percentage of context window to allocate to lorebook? (Suggest 25% as default)
2. **Depth injection + history truncation interaction**: If lorebook entries are injected at depth positions in history, and history is then truncated oldest-first, the injected system messages could be truncated. Need to decide: are injected lorebook system messages considered "sticky" (never truncated) or part of history (can be truncated)?
3. **Regex script count**: Should there be a maximum number of regex scripts per character? No limit in gtk_ref, but ReDoS protection via `size_limit` is the safeguard.
4. **Conversation lorebook removal UX**: When a conversation has lorebook assignments and the user removes one, should the next AI response immediately reflect the change? (Yes — lorebooks are activated fresh on each prompt assembly.)

---

## 8. Dependency Graph for Plan Ordering

```
09-01: Port LorebookService
  ├── Requires: lorebook_entry domain model updates (token_budget, sort_order, enabled)
  ├── Requires: DB migration (new columns + conversation_lorebooks table)
  └── Produces: service/lorebook.rs with activate + inject

09-02: Integrate lorebook into PromptEngine
  ├── Depends on: 09-01
  ├── Modifies: service/prompt.rs (replace Layer 2 stub)
  └── Modifies: commands/chat.rs (load lorebooks in ChatContext)

09-03: Port RegexService with validation
  ├── Requires: domain/regex_script.rs (new)
  ├── Requires: db/regex_scripts.rs (implement CRUD)
  ├── Requires: db/mod.rs (add DbCommand variants)
  └── Produces: service/regex.rs + commands/regex.rs

09-04: Integrate regex into AI response pipeline
  ├── Depends on: 09-03
  ├── Modifies: commands/chat.rs (apply regex before Complete)
  └── Modifies: app.svelte.ts (no frontend changes needed if backend transforms before sending)

09-05: Wire lorebook and regex to editor UIs and conversation flow
  ├── Depends on: 09-01, 09-02, 09-03, 09-04
  ├── Frontend types + API wrappers
  ├── ChatHeader: conversation lorebook panel
  ├── MessageInput: context bar lorebook detail
  ├── RegexScriptEditor: new component
  └── AppState: new state fields + actions
```

---

_Research completed: 2026-05-19_
