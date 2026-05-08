---
phase: 05-context-engineering-lorebook
plan: 01
subsystem: context-engineering
tags: [tiktoken-rs, cl100k_base, token-counting, lorebook, context-window, migration]

# Dependency graph
requires:
  - phase: 04-persona-system
    provides: CharacterCardV2 domain model, ProviderConfig, DB CRUD patterns, DbInput/DbOutput
provides:
  - TokenCountService with count_tokens() and resolve_context_window_size()
  - DEFAULT_CONTEXT_WINDOW constant (8192)
  - LorebookEntry domain model with InsertionPosition enum
  - Lorebook DB CRUD (insert, load, update, delete per character)
  - ProviderConfig.context_window_size field
  - CharacterCardV2.lorebook_scan_depth and lorebook_recursive_depth fields
  - Migration 008 (providers context_window_size, lorebook_entries table, characters lorebook settings)
affects: [05-02, 05-03, context-assembly, prompt-engineering]

# Tech tracking
tech-stack:
  added: [tiktoken-rs cl100k_base tokenizer (already in Cargo.toml)]
  patterns: [lazy-initialized Mutex<Option<CoreBPE>> for tokenizer, 3-tier context window fallback, JSON TEXT columns for Vec<String> fields, InsertionPosition Display/FromStr for DB TEXT storage]

key-files:
  created:
    - src/service/token_count.rs
    - src/domain/lorebook.rs
    - src/db/lorebook.rs
  modified:
    - src/service/mod.rs
    - src/domain/provider.rs
    - src/domain/character.rs
    - src/domain/mod.rs
    - src/db/migrations.rs
    - src/db/providers.rs
    - src/db/models.rs
    - src/db/characters.rs
    - src/db/mod.rs
    - src/db/conversations.rs
    - src/app.rs

key-decisions:
  - "Used cl100k_base singleton tokenizer for all providers per D-03 (single BPE, no per-provider tokenizers)"
  - "10% safety margin on token counts: count + count/10 (integer rounding, conservative overcount)"
  - "3-tier context window fallback: user override → built-in model defaults → DEFAULT (8192)"
  - "Model name matching uses contains() for flexibility (e.g. gpt-4o-mini matches gpt-4o pattern)"
  - "Lorebook scan depth is per-character (D-10), NOT per-entry — no activation_depth column"
  - "InsertionPosition stored as TEXT in DB with Display/FromStr conversion"
  - "depth_from_bottom default is 4 per UI-SPEC, sort_order default is 0"

patterns-established:
  - "TokenCountService pattern: lazy-init Mutex<Option<CoreBPE>> with helper fn for dereferencing"
  - "Lorebook CRUD pattern: follows personas.rs pattern with JSON TEXT serialization for Vec fields"
  - "DbInput/DbOutput integration: upsert pattern (check exists → update or insert) for Save variants"

requirements-completed: [CCTX-02, CCTX-04]

# Metrics
duration: 6min
completed: 2026-05-08
---

# Phase 05 Plan 01: Token Counting & Lorebook Data Layer Summary

**Token counting service with cl100k_base BPE and 10% safety margin, 3-tier context window fallback, and full lorebook CRUD data layer with per-character scan depth**

## Performance

- **Duration:** 6 min
- **Started:** 2026-05-08T00:58:44Z
- **Completed:** 2026-05-08T01:04:15Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- TokenCountService with lazy-initialized cl100k_base tokenizer counting tokens with 10% safety margin
- 3-tier context window resolution (user override → built-in model defaults → DEFAULT_CONTEXT_WINDOW=8192)
- ProviderConfig.context_window_size field for user override storage
- LorebookEntry domain model with all D-08 spec fields (keywords, secondary_terms, content, insertion position, depth, enabled, token budget, sort order)
- Full lorebook CRUD with DbInput/DbOutput integration and upsert pattern
- CharacterCardV2.lorebook_scan_depth and lorebook_recursive_depth per D-10, D-12
- Migration 008 combining all schema changes (providers, lorebook_entries, characters)

## Task Commits

Each task was committed atomically:

1. **Task 1: Token counting service and context window resolution** - `c735e27` (feat)
2. **Task 2: Lorebook domain model and database layer** - `4cbd5f7` (feat)

## Files Created/Modified
- `src/service/token_count.rs` - TokenCountService with cl100k_base, 3-tier context window fallback, 10% safety margin
- `src/service/mod.rs` - Added `pub mod token_count;`
- `src/domain/lorebook.rs` - LorebookEntry domain model with InsertionPosition enum
- `src/domain/mod.rs` - Added `pub mod lorebook;`
- `src/domain/provider.rs` - Added `context_window_size: Option<i64>` field with serde(default)
- `src/domain/character.rs` - Added lorebook_scan_depth (2) and lorebook_recursive_depth (2) fields
- `src/db/lorebook.rs` - Full CRUD: insert, load, update, delete lorebook entries per character
- `src/db/mod.rs` - Lorebook DbInput/DbOutput variants and DbService wiring
- `src/db/migrations.rs` - Migration 008: context_window_size, lorebook_entries table, character lorebook settings
- `src/db/providers.rs` - Updated INSERT/UPDATE/SELECT for context_window_size
- `src/db/models.rs` - Added row_to_lorebook_entry, updated row_to_provider and row_to_character
- `src/db/characters.rs` - Updated all queries for lorebook_scan_depth, lorebook_recursive_depth
- `src/db/conversations.rs` - Added context_window_size to test ProviderConfig construction
- `src/app.rs` - Added DbOutput lorebook match arms (stubs for later phase)

## Decisions Made
- Used cl100k_base single tokenizer for all providers per D-03 (not o200k_base, not per-provider)
- 10% safety margin uses integer division rounding (count + count/10) — always rounds down on margin, which is conservative
- Built-in model defaults use contains() matching for model names, enabling partial matches (e.g., "gpt-4o-mini" matches "gpt-4o")
- InsertionPosition stored as lowercase TEXT in DB with FromStr/Display impl for type-safe Rust enum conversion
- depth_from_bottom default 4 per UI-SPEC; sort_order default 0; enabled default true

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed MutexGuard dereference for CoreBPE method calls**
- **Found during:** Task 1 (compilation error)
- **Issue:** `get_bpe()` returned `MutexGuard<Option<CoreBPE>>` but `count_with_special_tokens()` is a method on `CoreBPE`, not on `Option<CoreBPE>` or `MutexGuard`
- **Fix:** Created a helper function `count_with_guard()` that unwraps the Option (always Some after init) and calls the method on the inner CoreBPE
- **Files modified:** src/service/token_count.rs
- **Verification:** `cargo test --lib -- token_count` passes (8 tests)
- **Committed in:** c735e27 (Task 1 commit)

**2. [Rule 2 - Missing Critical] Added context_window_size to conversations.rs test ProviderConfig**
- **Found during:** Task 1 (compilation failure in test build)
- **Issue:** `conversations.rs` test helper `make_test_provider()` constructed ProviderConfig directly without the new context_window_size field
- **Fix:** Added `context_window_size: None` to the struct literal
- **Files modified:** src/db/conversations.rs
- **Verification:** All 182 tests pass
- **Committed in:** c735e27 (Task 1 commit)

**3. [Rule 1 - Bug] Fixed INSERT column count mismatch (21 values for 22 columns)**
- **Found during:** Task 1 (character test failure after adding lorebook fields)
- **Issue:** Added lorebook_scan_depth and lorebook_recursive_depth to character INSERT column list but only had 21 placeholders for 22 columns
- **Fix:** Added ?22 placeholder to match the 22 columns
- **Files modified:** src/db/characters.rs
- **Verification:** All character and conversation tests pass
- **Committed in:** c735e27 (Task 1 commit)

**4. [Rule 3 - Blocking] Added DbOutput lorebook match arms to app.rs**
- **Found during:** Task 2 (non-exhaustive match compilation error)
- **Issue:** New DbOutput variants (LorebookEntriesLoaded, LorebookEntrySaved, LorebookEntryDeleted) required match arms in the AppModel update handler
- **Fix:** Added stub match arms with comments indicating UI wiring will come in a later phase
- **Files modified:** src/app.rs
- **Verification:** Full build and all 182 tests pass
- **Committed in:** 4cbd5f7 (Task 2 commit)

**5. [Rule 2 - Missing Critical] Added lorebook fields to to_domain() and to_v2() character conversions**
- **Found during:** Task 1 (character.rs structural field additions)
- **Issue:** CharaCardV2Spec::to_domain() and CharacterV1::to_v2() were missing the new lorebook_scan_depth and lorebook_recursive_depth fields, causing compilation errors
- **Fix:** Added both fields with default value 2 to both conversion methods
- **Files modified:** src/domain/character.rs
- **Verification:** cargo build and all tests pass
- **Committed in:** c735e27 (Task 1 commit)

**6. [Rule 1 - Bug] Removed unused HashMap import from token_count.rs**
- **Found during:** Task 1 (code review)
- **Issue:** `use std::collections::HashMap;` was imported but not used
- **Fix:** Removed the import
- **Files modified:** src/service/token_count.rs
- **Verification:** cargo check passes with no warnings for this file
- **Committed in:** c735e27 (Task 1 commit)

---

**Total deviations:** 6 auto-fixed (3 bugs, 2 missing critical, 1 blocking)
**Impact on plan:** All auto-fixes necessary for compilation and correctness. No scope creep — all fixes directly related to the new fields and code added by this plan.

## Issues Encountered
None beyond the auto-fixed deviations above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Token counting service ready for integration with prompt assembly (Plan 05-02)
- Lorebook CRUD ready for activation engine consumption (Plan 05-02)
- Migration 008 applies cleanly on fresh DB (verified via all test suite using in-memory DBs)
- All 182 tests pass including 8 token_count and 14 lorebook tests

---
*Phase: 05-context-engineering-lorebook*
*Completed: 2026-05-08*

## Self-Check: PASSED

- All 14 key files verified present on disk
- Both commit hashes (c735e27, 4cbd5f7) found in git log
- `cargo test --lib`: 182 passed, 0 failed
- `cargo build`: SUCCESS (no errors)
