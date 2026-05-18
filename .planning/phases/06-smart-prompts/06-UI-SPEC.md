---
phase: 6
slug: smart-prompts
status: draft
shadcn_initialized: false
preset: none
created: 2026-05-16
---

# Phase 6 — UI Design Contract

> Visual and interaction contract for the Smart Prompts phase. Budget-aware prompt assembly replaces the minimal Phase 5 prompt. Real token counting via tiktoken-rs replaces the frontend's 4-char-per-token heuristic. The context utilization bar in MessageInput receives real backend data. All design tokens, typography, color, spacing, and radii are inherited from Phase 1's locked design system. This is primarily a backend phase — frontend visual changes are minimal and constrained to the existing context bar component.

---

## Design System

**Inherited from Phase 1.** No changes to tooling, tokens, or component library.

| Property          | Value                                                           |
| ----------------- | --------------------------------------------------------------- |
| Tool              | none (no shadcn — project uses Bits.ui + Tailwind v4 directly)  |
| Component library | Bits.ui ^2.18 (headless Svelte primitives)                      |
| Icon library      | @lucide/svelte ^1.14                                            |
| Font              | Geist (sans), Fraunces (serif/display), Geist Mono (mono)       |
| Styling           | Tailwind CSS v4 with custom OKLCH theme tokens in `src/app.css` |
| Variant system    | tailwind-variants ^3.2 (`tv()`)                                 |

**Reference:** `.planning/phases/01-native-desktop-shell/01-UI-SPEC.md` § Design System — all unchanged.

---

## Spacing Scale

**Inherited from Phase 1.** No changes. 4px-multiple scale (4, 8, 12, 16, 20, 24, 32, 40, 48, 64).

**Phase 6 additions:** None. No new spacing values.

Exceptions: none

---

## Typography

**Inherited from Phase 1.** Consolidated to 4 sizes and 2 weights per design system rules.

**Type scale (4 sizes, 2 weights):**

| Role                                                       | Size        | Weight         | Line Height     | Font Family                             |
| ---------------------------------------------------------- | ----------- | -------------- | --------------- | --------------------------------------- |
| Body (chat bubbles, input, error messages)                 | 15px        | 400 (regular)  | 1.625 (relaxed) | Geist (sans)                            |
| Heading (speaker names, card titles, dialog title)         | 14px–18px\* | 600 (semibold) | 1.2             | Fraunces (serif/display) / Geist (sans) |
| Label / small (sidebar items, nav, metadata, field labels) | 12px (xs)   | 400 (regular)  | 1.25            | Geist (sans)                            |
| Textarea body / edit textarea                              | 14px (sm)   | 400 (regular)  | 1.625           | Geist (sans)                            |

\* Dialog title: 18px Geist (sans), weight 600. Card titles / speaker names: 14px Fraunces (serif/display), weight 600.

**Phase 6 role breakdown by element:**

| Element                                 | Size | Weight | Font Family | Notes                             |
| --------------------------------------- | ---- | ------ | ----------- | --------------------------------- |
| Context bar token count (e.g., "4,200") | 10px | 400    | Geist sans  | Existing — unchanged              |
| Context bar limit (e.g., "/ 128,000")   | 10px | 400    | Geist sans  | Existing — unchanged              |
| Context bar label text ("tok")          | 10px | 400    | Geist sans  | Existing — unchanged              |
| Budget overflow error bubble text       | 15px | 400    | Geist sans  | Same as Phase 5 error bubble text |
| Budget overflow error heading           | 12px | 600    | Geist sans  | Same as Phase 5 error label       |

No new typographic declarations in Phase 6.

---

## Color

All colors defined as OKLCH values in `src/app.css` @theme block. Dark theme only — light theme deferred to Phase 10.

**Inherited from Phase 5.** The full color table is documented in `05-UI-SPEC.md` § Color and `01-UI-SPEC.md` § Color. Not repeated here — no changes.

**Phase 6 context bar color shift (from D-01):**

The existing progress bar fill color changes dynamically based on context utilization percentage. This replaces the current static `--color-accent` fill:

| Utilization | Bar Fill Color         | Tailwind Token        | Visual Effect              |
| ----------- | ---------------------- | --------------------- | -------------------------- |
| < 80%       | `oklch(0.72 0.15 155)` | `--color-success`     | Green — healthy budget     |
| 80% – 95%   | `oklch(0.78 0.14 80)`  | `--color-warning`     | Yellow — approaching limit |
| > 95%       | `oklch(0.66 0.18 25)`  | `--color-destructive` | Red — budget critical      |

All three tokens already exist in `src/app.css` @theme block. No new color tokens.

**Accent reserved for (updated list — adds Phase 6 items):**

All Phase 1–5 accent reservations carry forward. Phase 6 adds:

- Context utilization progress bar fill (< 80% state uses success, not accent)

Note: The context bar previously used `--color-accent` unconditionally. Phase 6 replaces this with the three-tier semantic color shift. Accent is no longer used for the progress bar fill (it uses success/warning/destructive instead).

---

## Copywriting Contract

### CTAs and Button Labels

No new CTAs or button labels in Phase 6. The context bar has no interactive elements — it is read-only.

### Empty States

No new empty states in Phase 6.

### Error States

**Budget overflow error (NEW — rare edge case per D-04):**

When the PromptEngine cannot assemble a prompt even with maximum compression (e.g., sticky layer alone exceeds context window), an inline error bubble appears in the chat flow. This uses the **exact same error bubble visual contract from Phase 5** (see `05-UI-SPEC.md` § Error States) with:

| Element                    | Copy                                                                                       |
| -------------------------- | ------------------------------------------------------------------------------------------ |
| Error bubble heading       | "Generation failed"                                                                        |
| Error bubble body          | "Context budget exceeded. The system prompt is too large for this model's context window." |
| Action link (Retry)        | "Retry"                                                                                    |
| Action link (Switch model) | "Switch model"                                                                             |

The error type for this case is `context_budget_exceeded` — distinct from Phase 5's `context_length_exceeded` (which is a provider API rejection). Phase 6's budget overflow is caught **before** the API call.

### Destructive Actions

No new destructive actions in Phase 6. The auto-truncation (D-04) is silent — no confirmation UI.

---

## Component Inventory

### New UI Primitives (`src/lib/components/ui/`)

None.

### New Domain Components (`src/lib/components/`)

None.

### Modified Existing Components

| Component                    | Change                  | What Changes                                                                                                                                                                                                                                                                                                                                                                                                                                                                            |
| ---------------------------- | ----------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `MessageInput.svelte`        | Context bar data source | (1) Replace `usedContext` derived calculation (sum of `estimateTokens()` + hardcoded 8192) with data from `app.context_budget` (tokens_used, tokens_limit from backend StreamChunk response). (2) Replace static `contextLimit = 8192` with model-specific limit from backend. (3) Replace static `--color-accent` bar fill with three-tier semantic color: success (<80%), warning (80–95%), destructive (>95%). (4) Keep existing layout, sizing, typography — no structural changes. |
| `AppState` (`app.svelte.ts`) | New state field         | Add `context_budget` reactive field: `{ tokens_used: number; tokens_limit: number; truncated_count: number }` with `$state`. Updated from StreamChunk response data after each generation completes (D-02, D-03). Add `updateContextBudget(budget)` action.                                                                                                                                                                                                                             |
| `types/index.ts`             | New type                | Add `ContextBudget` interface: `{ tokens_used: number; tokens_limit: number; truncated_count: number }`. Potentially extend `StreamChunk` "complete" variant to include budget data.                                                                                                                                                                                                                                                                                                    |

### What Does NOT Change

- No changes to design tokens (spacing, typography, color, radii) — all inherited
- No changes to existing page routes
- No new npm dependencies on the frontend side (tiktoken-rs is backend-only)
- No changes to icon library
- No changes to MessageBubble, ChatHeader, ConversationSidebar, AppHeader
- No changes to Dialog, Button, Input, Select, Tag, KebabMenu UI primitives
- No new CSS utilities
- No shadcn or third-party registry components
- No changes to the welcome page, characters page, or settings page
- `estimateTokens()` in `$lib/utils/index.ts` — kept as utility (may be used elsewhere) but no longer drives the context bar

---

## Loading States

### Context Budget Update Timing (from D-02, D-03)

The context bar is **read-only** and updates at these moments:

| Event                                       | Bar Update                                                                |
| ------------------------------------------- | ------------------------------------------------------------------------- |
| Generation completes (StreamChunk.Complete) | `app.context_budget` updated from response data. Bar reflects new values. |
| Regeneration completes                      | Same as above — budget recalculated for the regenerated prompt.           |
| Edit/resend completes                       | Same as above — budget recalculated for the truncated+resent prompt.      |
| Between generations                         | Bar shows last known value. No real-time keystroke-level recalculation.   |
| App startup / conversation opened           | Bar shows default state: `tokens_used: 0, tokens_limit: <model limit>`.   |

**Bar does NOT update on every keystroke.** This is a deliberate decision from D-03 — real-time token counting requires a backend call per keystroke, which is expensive and unnecessary. The user sees budget pressure after each generation, which is when it matters.

### Context Bar Visual States

The context bar has three visual states determined by `contextPct = (tokens_used / tokens_limit) * 100`:

| State    | Utilization | Bar Fill              | Bar Background             | Text Color       |
| -------- | ----------- | --------------------- | -------------------------- | ---------------- |
| Healthy  | < 80%       | `--color-success`     | `--color-surface-elevated` | `--color-subtle` |
| Warning  | 80% – 95%   | `--color-warning`     | `--color-surface-elevated` | `--color-subtle` |
| Critical | > 95%       | `--color-destructive` | `--color-surface-elevated` | `--color-subtle` |

All text in the context bar row remains `text-(--color-subtle)` regardless of state. Only the progress bar fill color changes.

**Transition:** `transition-[width] duration-300` on the bar fill width (existing), plus `transition-colors duration-300` on the bar fill color (new — needed for smooth color shifts between states).

---

## Interaction Patterns

### No New Interaction Patterns

Phase 6 does not introduce any new user interactions. All interactions (send, stop, regenerate, edit/resend) are inherited from Phase 5 unchanged. The only change is that the context bar displays real data instead of a heuristic — but it remains non-interactive.

### Existing Patterns That Change Behavior (Not UI)

| Pattern             | Behavioral Change                                                                                                | Visual Change |
| ------------------- | ---------------------------------------------------------------------------------------------------------------- | ------------- |
| Send message        | Backend now uses PromptEngine with budget-aware assembly (character + persona + history within token limit)      | None          |
| Regenerate          | PromptEngine reassembles prompt from scratch for the regenerated response                                        | None          |
| Edit & resend       | Truncated messages are removed from prompt history; PromptEngine reassembles with the edited context             | None          |
| Context bar display | Shows real token counts from backend instead of `estimateTokens()` heuristic. Color shifts based on utilization. | Color only    |

### Deferred Interactions (Not in Phase 6)

| Pattern                             | Deferred To | Reason                                   |
| ----------------------------------- | ----------- | ---------------------------------------- |
| Lorebook injection in prompt        | Phase 9     | Not in scope                             |
| Per-conversation generation presets | Phase 9+    | Preset picker deferred                   |
| Context window size override UI     | Phase 10    | Settings UI enhancement                  |
| Message branching / forking         | Phase 8     | Branch navigation UI is a separate phase |
| Markdown rendering                  | Phase 7     | Current naive rendering continues        |

---

## Custom Utilities

**No new CSS utilities required.** All Phase 6 visual changes use existing Tailwind tokens and utilities:

- `bg-(--color-success)`, `bg-(--color-warning)`, `bg-(--color-destructive)` — progress bar fill colors (already defined in @theme)
- `transition-colors duration-300` — smooth color transitions on the bar fill

---

## Registry Safety

**No changes.** No shadcn, no third-party registries. All components remain custom-built using Bits.ui + Tailwind v4.

| Registry        | Blocks Used | Safety Gate    |
| --------------- | ----------- | -------------- |
| shadcn official | none        | not applicable |
| Third-party     | none        | not applicable |

---

## Phase 6 Specifics

### Discretionary Decisions Made

The following decisions were in "Claude's Discretion" in CONTEXT.md and are hereby declared:

1. **Context budget data flow — via StreamChunk.Complete, not a new command (from D-02):** Budget info (tokens_used, tokens_limit, truncated_count) is returned as part of the `StreamChunk.Complete` event payload. No new dedicated Tauri command. The frontend reads `data.budget` (or similar field) from the complete event and calls `app.updateContextBudget()`. This means the bar updates only after generation completes (D-03), which is the desired behavior.

2. **ContextBudget stored in AppState, not per-message (from D-02):** A single `context_budget` field on AppState holds the latest budget data. It is not stored per-message in MessageMetadata — budget is a conversation-level concern, not a message-level concern. The bar always shows the latest budget for the active conversation.

3. **estimateTokens() kept but demoted:** The `estimateTokens()` utility in `$lib/utils/index.ts` is not removed. It may be used for UI estimation purposes elsewhere. However, the context bar no longer uses it — it reads from `app.context_budget.tokens_used` instead. The `usedContext` derived calculation in MessageInput is replaced entirely.

4. **Context bar layout unchanged:** The existing layout (`text-[10px] uppercase tracking-wider text-(--color-subtle)`, progress bar `h-1 w-32 rounded-full`, etc.) is preserved. Only the data source and fill color change. The bar width still uses `contextPct = (tokens_used / tokens_limit) * 100`, but `tokens_used` and `tokens_limit` now come from backend data.

5. **Default context limit on conversation open:** When a conversation is opened (before any generation), the context bar shows `tokens_used: 0` and `tokens_limit: <model's context window>` resolved from the provider configuration. The three-tier resolution (user override → built-in table → default 8192) happens on the backend; the frontend simply displays the resolved limit.

6. **Silent truncation indicator (from D-04):** When PromptEngine truncates messages to fit the budget, the truncated count is available in `context_budget.truncated_count` but is NOT displayed in the UI. The color-coded bar communicates pressure. Truncation count is available for future UI enhancement (Phase 10+).

### Frontend Change Scope Summary

The total frontend diff for Phase 6 is small:

1. **MessageInput.svelte** — ~15 lines changed: replace `usedContext` derived calc + `contextLimit` constant with `app.context_budget` reads; add three-tier color logic to progress bar fill.
2. **app.svelte.ts** — ~10 lines added: `context_budget` field + `updateContextBudget()` action; update in stream complete handler.
3. **types/index.ts** — ~5 lines added: `ContextBudget` interface; extend StreamChunk.Complete data type.

Total: ~30 lines of frontend code changed. The remaining ~95% of Phase 6 work is backend (Rust).

---

## Checker Sign-Off

- [ ] Dimension 1 Copywriting: PASS
- [ ] Dimension 2 Visuals: PASS
- [ ] Dimension 3 Color: PASS
- [ ] Dimension 4 Typography: PASS
- [ ] Dimension 5 Spacing: PASS
- [ ] Dimension 6 Registry Safety: PASS

**Approval:** pending
