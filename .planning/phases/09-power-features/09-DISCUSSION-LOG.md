# Phase 9: Power Features - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-19
**Phase:** 09-power-features
**Areas discussed:** Regex Script Scoping, Conversation Lorebook UX, Lorebook Injection Visibility, Regex Output Preview

---

## Regex Script Scoping

| Option                      | Description                                                                                                                      | Selected |
| --------------------------- | -------------------------------------------------------------------------------------------------------------------------------- | -------- |
| Per-character               | Scripts belong to the character. All conversations use same scripts. Matches gtk_ref and SillyTavern. Simpler data model and UI. | ✓        |
| Per-conversation            | Scripts belong to each conversation. More flexible but requires new table and chat UI management.                                |          |
| Hybrid (inherit + override) | Character defines defaults, conversation can override/add. Most flexible but most complex.                                       |          |

**User's choice:** Per-character (Recommended)
**Notes:** Aligns with gtk_ref implementation. Regex editor lives in character editor. Existing `regex_scripts` table already has `character_id` FK.

---

## Conversation Lorebook UX

| Option                       | Description                                                             | Selected |
| ---------------------------- | ----------------------------------------------------------------------- | -------- |
| Chat header panel            | Inline section near persona switcher. Quick access, always visible.     | ✓        |
| Settings popover from header | Gear icon opens popover/dialog. Cleaner header, more room for options.  |          |
| Sidebar conversation details | No UI during chat. Assign in conversation properties view from sidebar. |          |

**User's choice:** Chat header panel
**Notes:** All conversation-level config (persona + lorebooks) in one spot.

---

## Lorebook Token Budget Allocation

| Option                     | Description                                                                                   | Selected |
| -------------------------- | --------------------------------------------------------------------------------------------- | -------- |
| Even split                 | Equal share per lorebook. Simple, predictable, no decision fatigue.                           | ✓        |
| Per-lorebook manual budget | User sets specific token cap per lorebook. More control but requires budget understanding.    |          |
| Auto + manual override     | System auto-allocates, users can override specific lorebooks. Most flexible but most complex. |          |

**User's choice:** Even split (Recommended)
**Notes:** Backend splits lorebook token budget evenly. Within each lorebook, entries follow gtk_ref pattern (explicit budgets first, remaining share the rest).

---

## Lorebook Injection Visibility

| Option                       | Description                                                                                        | Selected |
| ---------------------------- | -------------------------------------------------------------------------------------------------- | -------- |
| Invisible (token count only) | Context bar token count updates, no way to see what was injected.                                  |          |
| Context bar detail           | Expandable section listing activated entries with name, snippet, token cost. Collapsed by default. | ✓        |
| Full prompt inspector panel  | Dedicated debug panel showing full assembled prompt with highlights. Off by default.               |          |

**User's choice:** Context bar detail (Recommended)
**Notes:** Collapsed shows "Lorebook: N active", expanded shows entry details. Practical debugging without UI overload.

---

## Regex Output Preview

| Option                | Description                                                                                                  | Selected |
| --------------------- | ------------------------------------------------------------------------------------------------------------ | -------- |
| Inline test panel     | Inside regex editor, sample text input with live transformed output. Matches Phase 3 keyword tester pattern. | ✓        |
| No preview            | Users trust patterns, verify by chatting.                                                                    |          |
| Preview dialog button | Button opens small dialog with sample input/output. More space but not always visible.                       |          |

**User's choice:** Inline test panel (Recommended)
**Notes:** Consistent with lorebook keyword tester from Phase 3. Backend provides apply_single_regex command.

---

## Agent's Discretion

- Exact layout of chat header lorebook panel (dropdown, popover, or inline chips)
- Exact layout of regex editor within character editor (tab, accordion, or step)
- Error handling for lorebook budget overflow at conversation level
- Regex + streaming interaction (chunk-by-chunk vs on completion)
- Context bar detail content (matched keys vs entry names only)

## Deferred Ideas

None — discussion stayed within phase scope
