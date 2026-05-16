---
phase: 06-smart-prompts
plan: 01
subsystem:
  - ai
  - prompt-engine
tags: [tiktoken-rs, prompt-assembly, token-counting, budget-aware, template-variables, persona-compression]

# Dependency graph
requires:
  - phase: 05-live-conversations
    provides: rig-core 0.37 AI service with streaming, domain models (CharacterCardV2, Persona, Message, LorebookEntry)
provides:
  - TokenCountService with tiktoken-rs cl100k_base BPE tokenizer
  - Template service with variable resolution and 4-tier persona compression
  - PromptEngine with budget-aware 5-layer prompt assembly
  - AssembledPrompt struct for streaming pipeline integration
affects: [06-02, 06-03, 06-04, 09-power-features]

# Tech tracking
tech-stack:
  added: [tiktoken-rs 0.11]
  patterns: [budget-aware-prompt-assembly, 5-layer-priority-queue, persona-compression-tiers, lazy-bpe-initialization]

key-files:
  created:
    - src-tauri/src/service/mod.rs
    - src-tauri/src/service/token_count.rs
    - src-tauri/src/service/template.rs
    - src-tauri/src/service/prompt.rs
  modified:
    - src-tauri/Cargo.toml
    - src-tauri/src/lib.rs

key-decisions:
  - "Used rig_core::completion::Message (not rig::message::Message) to match ai/mod.rs convention"
  - "Lorebook activation stubbed with Phase 9 TODO — assemble_with_budget accepts lorebook_entries but sets active_lorebook_count to 0"
  - "ProviderType::CustomOpenAICompatible used instead of gtk_ref's ProviderType::Custom"

patterns-established:
  - "Service module pattern: service/mod.rs re-exports main types for convenience imports"
  - "Budget-aware assembly: 5-layer priority (sticky → lorebook → persona → history → post-history)"
  - "Persona compression: Full → Trimmed → Minimal → Essential progressive fallback"

requirements-completed: [PRMP-01, PRMP-02]

# Metrics
duration: 5min
completed: 2026-05-16
---

# Phase 6 Plan 01: Port Service Module Summary

**Ported TokenCountService, template service, and PromptEngine from gtk_ref/ with tiktoken-rs dependency and rig_core::completion::Message integration**

## Performance

- **Duration:** 5 min
- **Started:** 2026-05-16T04:34:26Z
- **Completed:** 2026-05-16T04:39:29Z
- **Tasks:** 5
- **Files modified:** 6

## Accomplishments

- TokenCountService with lazy-initialized cl100k_base tokenizer and 10% safety margin
- Template service with {{user}}, {{char}}, {{persona.*}} variable resolution and 4-tier persona compression
- PromptEngine with budget-aware 5-layer prompt assembly (sticky → lorebook → persona → history → post-history)
- 45 unit tests across all three modules, all passing
- Lorebook activation stubbed for Phase 9 forward compatibility

## Task Commits

Each task was committed atomically:

1. **Task 1: Add tiktoken-rs dependency and service module declaration** - `2918242` (feat)
2. **Task 2: Port TokenCountService** - `c20aebf` (feat)
3. **Task 3: Port template service** - `efd435e` (feat)
4. **Task 4: Port PromptEngine** - `53b6971` (feat)
5. **Task 5: Verify compilation** - verified via `cargo check` + `cargo test` (45/45 tests pass)

## Files Created/Modified

- `src-tauri/Cargo.toml` - Added tiktoken-rs = "0.11" dependency
- `src-tauri/src/lib.rs` - Added mod service declaration
- `src-tauri/src/service/mod.rs` - Module barrel with re-exports
- `src-tauri/src/service/token_count.rs` - TokenCountService with BPE tokenizer, context window resolution, 8 tests
- `src-tauri/src/service/template.rs` - Variable resolution, persona blocks, compression, 20 tests
- `src-tauri/src/service/prompt.rs` - PromptEngine with budget-aware assembly, 15 tests

## Decisions Made

- Used `rig_core::completion::Message` to match existing `ai/mod.rs` convention (not `rig::message::Message` from gtk_ref)
- Lorebook activation stubbed in `assemble_with_budget()` with `// TODO: Phase 9` comment — signature accepts `lorebook_entries` for forward compat
- Fixed `ProviderType::Custom` → `ProviderType::CustomOpenAICompatible` to match Tauri domain enum

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] ProviderType::Custom does not exist in Tauri domain**

- **Found during:** Task 5 (compilation verification)
- **Issue:** gtk_ref uses `ProviderType::Custom` but Tauri domain uses `ProviderType::CustomOpenAICompatible`
- **Fix:** Changed test in token_count.rs to use `ProviderType::CustomOpenAICompatible`
- **Files modified:** src-tauri/src/service/token_count.rs
- **Verification:** `cargo check` passes, test passes
- **Committed in:** c20aebf (Task 2 commit)

**2. [Rule 1 - Bug] Unused mut and assignment in prompt.rs**

- **Found during:** Task 5 (compilation verification)
- **Issue:** `history_messages` was declared `mut` unnecessarily, `truncated_messages` was assigned then overwritten
- **Fix:** Removed `mut` from `history_messages`, changed `truncated_messages` to `let` binding
- **Files modified:** src-tauri/src/service/prompt.rs
- **Verification:** `cargo check` passes without warnings for these variables
- **Committed in:** 53b6971 (Task 4 commit)

---

**Total deviations:** 2 auto-fixed (2 bugs)
**Impact on plan:** Minor fixes for type mismatches between gtk_ref and Tauri domain. No scope creep.

## Issues Encountered

None - clean port with all tests passing.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- TokenCountService, template service, and PromptEngine are ready for integration
- Plans 06-02 and 06-03 can now integrate these services into the streaming pipeline and frontend
- Lorebook integration point is prepared for Phase 9 with forward-compatible signature

---

_Phase: 06-smart-prompts_
_Completed: 2026-05-16_
