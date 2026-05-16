# Phase 6: Smart Prompts - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-16
**Phase:** 06-smart-prompts
**Areas discussed:** Context utilization display, Token data flow & timing, Budget overflow UX, Persona in prompt format

---

## Context Utilization Display

| Option            | Description                                                                 | Selected |
| ----------------- | --------------------------------------------------------------------------- | -------- |
| Enhanced bar      | Progress bar with real model limits, color shifts green→yellow→red. Clean.  | ✓        |
| Detailed status   | Bar + model name, truncated msg count, compression tier. More info, busier. |          |
| Minimal           | Just bar with real numbers, no color, warning only on overflow.             |          |

**User's choice:** Enhanced bar (Recommended)
**Notes:** Color thresholds at 80% (yellow) and 95% (red). No extra text beyond tokens used / limit.

---

## Token Data Flow

| Option              | Description                                                                                    | Selected |
| ------------------- | ---------------------------------------------------------------------------------------------- | -------- |
| Stream response     | Budget info returned as part of start_chat flow. No new Tauri commands. Stale between msgs.    | ✓        |
| Dedicated command   | New `get_context_budget()` Tauri command. Always fresh but extra backend calls.                |          |
| Hybrid              | Both: command for initial load, stream response for updates. Most responsive, most work.       |          |

**User's choice:** Stream response (Recommended)
**Notes:** Context budget (tokens used, limit, truncated count) piggybacks on existing StreamChunk flow. No new Tauri commands.

## Token Refresh Timing

| Option              | Description                                                                                    | Selected |
| ------------------- | ---------------------------------------------------------------------------------------------- | -------- |
| After generation    | Bar updates only after AI responds. Simplest, no extra traffic.                                | ✓        |
| Real-time keystroke | Recalculate on every keystroke. Most accurate but heavy backend traffic.                       |          |
| Debounced typing    | Recalculate after 500ms of inactivity. Balance but needs new Tauri command.                    |          |

**User's choice:** After generation (Recommended)
**Notes:** Bar stays at last known value between messages. Refreshes on stream completion and regeneration.

---

## Budget Overflow UX

| Option                    | Description                                                                                          | Selected |
| ------------------------- | ---------------------------------------------------------------------------------------------------- | -------- |
| Silent truncate + color   | Auto-truncate oldest, compress persona. Color bar shows pressure. Inline error only if can't fit.    | ✓        |
| Auto-truncate with note   | Same auto-truncation plus "3 messages trimmed" text near bar.                                         |          |
| Block sending on overflow | Prevent sending when over budget. User must delete msgs or switch model. Safest, most restrictive.   |          |

**User's choice:** Silent truncate + bar color (Recommended)
**Notes:** Matches gtk_ref approach. Bar color communicates pressure. Inline error only when physically impossible to fit.

---

## Persona in Prompt Format

| Option              | Description                                                                                    | Selected |
| ------------------- | ---------------------------------------------------------------------------------------------- | -------- |
| gtk_ref format      | Structured block with 4-tier compression. Proven, tested, consistent with port approach.        | ✓        |
| Custom format       | Same compression system but different wording/layout.                                           |          |
| Agent's discretion  | Just ensure persona info is included. Exact format up to planner.                              |          |

**User's choice:** gtk_ref format (Recommended)
**Notes:** Port template.rs from gtk_ref exactly. "You are interacting with {{user}}, who is:\nName: X\nPersonality: Y\n..." with Full/Trimmed/Minimal/Essential tiers.

---

## Agent's Discretion

- Exact StreamChunk variant shape for budget info
- Built-in model context window defaults table content
- tiktoken-rs lazy init approach (Mutex as gtk_ref or alternative)
- Service module directory structure

## Deferred Ideas

None — discussion stayed within phase scope.
