---
phase: 07-rich-output-cards
plan: 03
subsystem:
  - ui
tags: [shiki, syntax-highlighting, preloading, markdown, code-blocks]

requires:
  - phase: 07-rich-output-cards
    provides: MarkdownRenderer component with Shiki code highlighting

provides:
  - Shiki engine preloaded on app startup for instant code block rendering
  - Fire-and-forget preload pattern that doesn't block app initialization

affects: [markdown-rendering, chat-messages]

tech-stack:
  added: []
  patterns: [fire-and-forget preloading in onMount]

key-files:
  created: []
  modified:
    - src/routes/+layout.svelte

key-decisions:
  - "Fire-and-forget preload pattern: createHighlighter is not awaited so app startup is not blocked"
  - "Same language list as MarkdownRenderer (9 languages) to ensure cache hit on first render"

patterns-established:
  - "Preload heavy async dependencies in root layout onMount with .then()/.catch() pattern"

requirements-completed: [CHAT-06]

duration: 1min
completed: 2026-05-18
---

# Phase 7 Plan 3: Shiki Preloading + Markdown Polish Summary

**Shiki engine preloaded in root layout so code blocks render instantly on first AI response**

## Performance

- **Duration:** 1 min
- **Started:** 2026-05-18T19:24:27Z
- **Completed:** 2026-05-18T19:25:15Z
- **Tasks:** 1
- **Files modified:** 1

## Accomplishments

- Added Shiki `createHighlighter` fire-and-forget preload in root layout `onMount`
- Preloads vitesse-dark theme with 9 common language grammars (javascript, typescript, python, rust, json, html, css, bash, markdown)
- MarkdownRenderer will reuse the cached engine on first render, eliminating the initial code block plaintext flash

## Task Commits

1. **Task 1: Preload Shiki in root layout** - `7bc70a6` (feat)

## Files Created/Modified

- `src/routes/+layout.svelte` - Added Shiki preloading import and fire-and-forget createHighlighter call in onMount

## Decisions Made

- Used fire-and-forget `.then()/.catch()` pattern rather than `await` to avoid blocking app startup
- Matched the same language list from MarkdownRenderer (9 languages) to maximize cache reuse probability

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Shiki preloading complete, ready for plan 07-04
- All Phase 7 markdown infrastructure is in place (MarkdownRenderer + prose styles + preloading)

---

_Phase: 07-rich-output-cards_
_Completed: 2026-05-18_
