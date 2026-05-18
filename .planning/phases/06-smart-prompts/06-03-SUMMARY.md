---
phase: 06-smart-prompts
plan: 03
subsystem: ui
tags: [context-budget, token-counting, svelte-5, runes]

# Dependency graph
requires:
  - phase: 06-smart-prompts
    provides: PromptEngine with budget-aware assembly, tiktoken-rs token counting
provides:
  - ContextBudget type for frontend token budget tracking
  - Three-tier color-coded context utilization bar
  - Real backend-driven context display replacing estimateTokens heuristic
affects: [chat-ui, streaming]

# Tech tracking
tech-stack:
  added: []
  patterns: [context-budget-state-pattern, three-tier-color-coding]

key-files:
  created: []
  modified:
    - src/lib/types/index.ts
    - src/lib/stores/app.svelte.ts
    - src/lib/components/chat/MessageInput.svelte

key-decisions:
  - "Fallback to 8192 context limit when no budget data available"
  - "Keep estimateTokens() in utils for potential other uses"

patterns-established:
  - "Context budget pattern: backend returns budget in StreamChunk::Complete, frontend stores in AppState.context_budget, components derive from it"
  - "Three-tier color coding: green (<80%), yellow (80-95%), red (>95%)"

requirements-completed: [PRMP-03]

# Metrics
duration: 4min
completed: 2026-05-16
---

# Phase 6 Plan 3: Wire Token Count Display to Real Backend Data Summary

**Replaced heuristic-based token counting with real backend ContextBudget data and added three-tier color-coded context bar (green/yellow/red)**

## Performance

- **Duration:** 4 min
- **Started:** 2026-05-16T04:47:08Z
- **Completed:** 2026-05-16T04:51:06Z
- **Tasks:** 4
- **Files modified:** 3

## Accomplishments

- Extended StreamChunk complete variant with tokensUsed, tokensLimit, truncatedCount fields
- Added ContextBudget interface and wired it into AppState as reactive state
- Updated all three streaming handlers (startChat, regenerate, editResend) to extract budget from complete events
- Replaced estimateTokens heuristic with real backend data and added semantic color coding

## Task Commits

Each task was committed atomically:

1. **Task 1: Extend StreamChunk type and add ContextBudget interface** - `c98f83f` (feat)
2. **Task 2: Add context_budget state to AppState and update from stream events** - `334236b` (feat)
3. **Task 3: Replace MessageInput context bar with real backend data and color coding** - `7468280` (feat)
4. **Task 4: Verify frontend builds and type-checks** - no code commit (verification only)

**Plan metadata:** pending (docs: complete plan)

## Files Created/Modified

- `src/lib/types/index.ts` - Extended StreamChunk complete variant with budget fields; added ContextBudget interface
- `src/lib/stores/app.svelte.ts` - Added context_budget state, updateContextBudget action, and budget extraction in all three streaming handlers
- `src/lib/components/chat/MessageInput.svelte` - Replaced estimateTokens with real backend data; added three-tier color coding

## Decisions Made

- Fallback to 8192 context limit when no budget data is available from backend (graceful degradation)
- Kept estimateTokens() in utils per UI-SPEC decision #3 (not removed)

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Context bar now reads real backend budget data from StreamChunk::Complete events
- Ready for plan 06-04 (integrate prompt engine into streaming chat flow)

---

_Phase: 06-smart-prompts_
_Completed: 2026-05-16_
