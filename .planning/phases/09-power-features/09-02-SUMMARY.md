---
phase: 09-power-features
plan: 02
subsystem: prompt-engine
tags: [lorebook, prompt-assembly, tauri-commands, token-budget]

# Dependency graph
requires:
  - phase: 09-power-features
    provides: lorebook service (activate/inject), DB junction tables, DbCommand variants
provides:
  - ChatContext lorebook loading (character + conversation lorebooks)
  - PromptEngine Layer 2 lorebook activation and injection
  - LorebookActivationDetail struct for Complete chunk
  - Three conversation-lorebook Tauri commands
affects: [chat-streaming, prompt-assembly, frontend-lorebook-ui]

# Tech tracking
tech-stack:
  added: []
  patterns: [lorebook-injection-after-truncation, lorebook-budget-25-percent]

key-files:
  created: []
  modified:
    - src-tauri/src/commands/chat.rs
    - src-tauri/src/service/prompt.rs
    - src-tauri/src/domain/lorebook.rs
    - src-tauri/src/commands/lorebooks.rs
    - src-tauri/src/lib.rs
    - src-tauri/src/ai/mod.rs

key-decisions:
  - "Depth-based lorebook entries injected AFTER truncation to prevent silent removal by budget truncation"
  - "Lorebook budget fraction set to 25% of context window as constant (not yet user-configurable)"
  - "LorebookActivationDetail uses lorebook_id as fallback name when lorebook_names map not available in PromptEngine"

patterns-established:
  - "Lorebook injection after truncation: depth-based entries inserted into already-truncated history to prevent oldest-first truncation from removing them"
  - "Budget fraction pattern: lorebook gets fixed 25% slice of context window, split across character + conversation lorebooks"

requirements-completed: [PRMP-04, PRMP-05, LORB-04]

# Metrics
duration: 8min
completed: 2026-05-20
---

# Phase 9 Plan 2: Integrate Lorebook into PromptEngine and ChatContext Summary

**Lorebook entries activated and injected at correct positions in PromptEngine, with conversation-lorebook CRUD Tauri commands**

## Performance

- **Duration:** 8 min
- **Started:** 2026-05-20T02:25:00Z
- **Completed:** 2026-05-20T02:33:00Z
- **Tasks:** 3
- **Files modified:** 6

## Accomplishments
- ChatContext loads lorebook entries from both character-assigned and conversation-assigned lorebooks
- PromptEngine Layer 2 activates lorebook entries and injects them at correct positions (BeforeChar, AfterChar, AtDepth)
- Depth-based lorebook entries injected AFTER truncation to prevent silent removal
- Lorebook tokens counted in budget with 25% allocation fraction
- Three new Tauri commands for conversation-lorebook junction operations

## Task Commits

Each task was committed atomically:

1. **Task 1+2: Integrate lorebook into ChatContext and PromptEngine** - `c15d287` (feat)
2. **Task 3: Add conversation lorebook Tauri commands** - `a41bfc8` (feat)

## Files Created/Modified
- `src-tauri/src/commands/chat.rs` - Extended ChatContext with lorebook fields, added lorebook loading steps 7-9
- `src-tauri/src/service/prompt.rs` - Replaced Layer 2 stub with actual lorebook activation/injection, added lorebook_details
- `src-tauri/src/domain/lorebook.rs` - Added LorebookActivationDetail struct
- `src-tauri/src/commands/lorebooks.rs` - Added three conversation-lorebook Tauri commands
- `src-tauri/src/lib.rs` - Registered three new commands in invoke_handler
- `src-tauri/src/ai/mod.rs` - Fixed StreamChunk::Complete test to include lorebook_details field

## Decisions Made
- Depth-based lorebook entries injected AFTER truncation (not before) — prevents injected system messages at old positions from being silently removed by budget truncation. Depth offsets are relative to truncated history length, matching SillyTavern/gtk_ref behavior.
- Lorebook budget fraction is 25% of context window — constant for now, not yet user-configurable.
- LorebookActivationDetail uses lorebook_id as fallback for lorebook_name since the lorebook_names map requires DB access not available in PromptEngine.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed StreamChunk::Complete test missing lorebook_details field**
- **Found during:** Task 2 (running cargo test after PromptEngine changes)
- **Issue:** The other agent (09-01) added `lorebook_details` to `StreamChunk::Complete` but missed updating a test in `ai/mod.rs`
- **Fix:** Added `lorebook_details: Vec::new()` to the test's `StreamChunk::Complete` construction
- **Files modified:** `src-tauri/src/ai/mod.rs`
- **Verification:** `cargo test` passes (257 tests)
- **Committed in:** `c15d287` (Task 1+2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary fix for compilation. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Lorebook fully integrated into prompt assembly pipeline
- Ready for frontend lorebook UI to consume conversation-lorebook Tauri commands
- LORB-04 (assign lorebooks to characters and conversations) now completable from frontend

---
*Phase: 09-power-features*
*Completed: 2026-05-20*
