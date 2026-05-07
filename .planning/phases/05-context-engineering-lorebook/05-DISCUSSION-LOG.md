# Phase 5: Context Engineering & Lorebook - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-07
**Phase:** 05-context-engineering-lorebook
**Areas discussed:** Token Counting & Budgeting, Context Assembly & Truncation, Lorebook Data Model & Activation, Context Utilization UI

---

## Token Counting & Budgeting

### Token counting approach

| Option | Description | Selected |
|--------|-------------|----------|
| Count-as-you-build | Count tokens after each layer. Triggers truncation/compression before adding more. | ✓ |
| Count-after-assembly | Build full prompt, then count. Strip history if over budget. | |
| Pre-estimate + post-verify | Estimate per layer, assemble within estimate, verify with actual count. | |

**User's choice:** Count-as-you-build
**Notes:** Most accurate approach — catches overflows before sending to provider.

### Context window size resolution

| Option | Description | Selected |
|--------|-------------|----------|
| Built-in table | Hardcoded model → context window map with defaults. | |
| User-configured only | Manual per-provider/model setup. | |
| Provider API query | Query provider API for model capabilities at runtime. | ✓ (modified) |

**User's choice:** Provider API query first, with fallback chain to user-configured override, then built-in defaults.
**Notes:** User wants provider API query as primary source. If query fails or provider doesn't support it, fall back to user override, then built-in defaults. Best coverage across all providers including local (Ollama, LM Studio).

### Tokenizer strategy

| Option | Description | Selected |
|--------|-------------|----------|
| tiktoken with safety margin | Single BPE tokenizer for all providers with ~10% margin. | ✓ |
| Per-provider tokenizer dispatch | Use correct tokenizer per provider. | |
| tiktoken + provider-reported usage | tiktoken for estimation, calibrate with actual API usage. | |

**User's choice:** tiktoken with safety margin
**Notes:** Simpler — one tokenizer, one count path. Most providers are close enough.

---

## Context Assembly & Truncation

### Post-history instructions placement

| Option | Description | Selected |
|--------|-------------|----------|
| After history (spec intent) | Per CharacterCardV2 spec — instructions AFTER chat history. | ✓ |
| Before history (sticky) | Part of sticky character layer, never truncated. | |
| Semi-sticky | Same priority as persona — compressed but not truncated. | |

**User's choice:** After history (CharacterCardV2 spec intent)
**Notes:** Matches SillyTavern behavior and CharacterCardV2 spec definition.

### History truncation strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Oldest-first | Remove oldest user/assistant message pairs first. | ✓ |
| Oldest-first with summary | Remove oldest but inject brief summary. | |
| Priority-weighted eviction | Score messages by recency + relevance. | |

**User's choice:** Oldest-first
**Notes:** Standard approach — what SillyTavern and most chat UIs do.

### Persona compression tiers

| Option | Description | Selected |
|--------|-------------|----------|
| 4-tier progressive | Full → trimmed → minimal → essential. | ✓ |
| 2-tier with field omission | Full or minimal only. | |
| User decides field priority | User marks essential vs optional fields. | |

**User's choice:** 4-tier progressive compression
**Notes:** Full (all fields) → trimmed (name, personality, appearance) → minimal (name + personality) → essential (just name).

### Truncation notification

| Option | Description | Selected |
|--------|-------------|----------|
| Silent indicator only | Context indicator changes color. No toast/dialog. | ✓ |
| Toast on first truncation | One-time toast when truncation first occurs. | |
| Always toast | Toast every time prompt is truncated. | |

**User's choice:** Silent indicator only
**Notes:** User can see budget pressure via the context utilization indicator color changes.

---

## Lorebook Data Model & Activation

### Lorebook storage approach

| Option | Description | Selected |
|--------|-------------|----------|
| Separate lorebook_entries table | New table with per-entry fields. Queryable, editable individually. | ✓ |
| JSON in character_book column | Entries as JSON array in existing column. | |
| Hybrid: table + JSON cache | Table for CRUD, JSON cache for import/export. | |

**User's choice:** Separate lorebook_entries table
**Notes:** Better for large lorebooks, individual entry editing, and queryability. character_book column preserved for import/export serialization.

### Keyword matching logic

| Option | Description | Selected |
|--------|-------------|----------|
| Any keyword match (OR) | Entry activates if ANY keyword appears. | ✓ |
| All keywords required (AND) | Entry activates only when ALL keywords appear. | |
| Configurable per entry | Each entry has a match_logic field (OR/AND). | |

**User's choice:** Any keyword match (OR)
**Notes:** Simpler, matches SillyTavern default, good for broad world-building entries.

### Scan depth

| Option | Description | Selected |
|--------|-------------|----------|
| Configurable scan depth | User sets how many recent messages to scan. Default 2-3. | ✓ |
| Scan entire visible history | Always scan all messages in prompt. | |
| Scan only new messages | Only scan messages added since last check. | |

**User's choice:** Configurable scan depth per character
**Notes:** Matches SillyTavern behavior. Default 2-3 messages.

### Insertion position system

| Option | Description | Selected |
|--------|-------------|----------|
| Depth-from-bottom (ST style) | Depth 0 = after last message, depth 1 = one from end, etc. | ✓ |
| Named positions only | Fixed positions: before_char, after_char, etc. | |
| Depth + named positions | Both options for full SillyTavern compatibility. | |

**User's choice:** Depth-from-bottom (SillyTavern style)
**Notes:** Full SillyTavern character_book spec compatibility.

### Recursive activation

| Option | Description | Selected |
|--------|-------------|----------|
| Iterative with max depth counter | Re-scan activated entries' content for keywords up to N times. | ✓ |
| Single-pass only | No recursion — only scan conversation messages. | |
| Recursion with dependency tracking | Full graph traversal with cycle detection. | |

**User's choice:** Iterative with max depth counter
**Notes:** Default max depth 2. Prevents infinite loops via counter.

### Per-entry token budgets

| Option | Description | Selected |
|--------|-------------|----------|
| Soft cap with overflow | Truncate entry at budget, remaining tokens available for next entries. | ✓ |
| Hard cap per entry | Each entry MUST have budget. Drop entries that exceed. | |
| Budget as pool for all entries | Total lorebook budget, entries fill in priority order. | |

**User's choice:** Soft cap with overflow
**Notes:** Entries without explicit budgets share remaining lorebook allocation. Pragmatic — doesn't waste space.

---

## Context Utilization UI

### Indicator position

| Option | Description | Selected |
|--------|-------------|----------|
| Input bar | Small progress bar below/above text input. Always visible. | ✓ |
| Chat header | In header bar next to character name. | |
| Status bar at bottom | Thin bar below message area. | |

**User's choice:** Input bar
**Notes:** Near where the user types — natural place to see "am I running out of room?"

### Indicator visual form

| Option | Description | Selected |
|--------|-------------|----------|
| Thin progress bar | Color-coded: green (<70%), yellow (70-90%), red (>90%). | ✓ |
| Token count text | Show '3.2k / 8k tokens' as text. | |
| Bar + text on hover/click | Progress bar with popover for detailed breakdown. | |

**User's choice:** Thin progress bar with color coding
**Notes:** Compact, glanceable, matches GNOME design patterns.

### Update frequency

| Option | Description | Selected |
|--------|-------------|----------|
| On every prompt assembly | Recalculate after each message, persona change, lorebook activation. | ✓ |
| Periodic background refresh | Update every N seconds or messages. | |
| On demand only | Only update on hover/click. | |

**User's choice:** On every prompt assembly
**Notes:** Always reflects the actual prompt that would be sent.

---

## the agent's Discretion

- tiktoken-rs model selection (cl100k_base vs o200k_base)
- Built-in context window size table contents
- Lorebook entry database schema field naming
- character_book JSON import/export serialization mapping
- Scan depth default value (2 or 3)
- Recursive activation default max depth (2 or 3)
- Safety margin percentage
- Progress bar exact height and styling
- Migration numbering

## Deferred Ideas

- Persona-aware lorebook activation (entries activate based on active persona data) → v2 (CCTX-A01)
- {{char.field}} template expansion beyond {{char}} → Phase 6 or later
- Per-provider tokenizer dispatch → future polish
- History summarization on truncation → future enhancement
- Keyword matching with AND logic or configurable per entry → future enhancement
- Detailed context breakdown popover → future polish
