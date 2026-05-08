---
phase: 06-generation-controls-regex-data
plan: 01
subsystem: database
tags: [generation-presets, regex-scripts, sqlite, crud, redos, regex-validation]

# Dependency graph
requires:
  - phase: 05-context-engineering-lorebook
    provides: lorebook_entries table pattern, DB CRUD pattern, DbInput/DbOutput variants
provides:
  - GenerationPreset domain model and CRUD operations
  - RegexScript domain model and CRUD operations
  - Migration M9 (generation_presets, regex_scripts tables, FK columns)
  - Built-in preset seeding (Creative, Balanced, Precise)
  - ReDoS validation service with size_limit
  - Regex script execution pipeline (pipe chain)
affects: [06-02, 06-03]

# Tech tracking
tech-stack:
  added: [regex::RegexBuilder::size_limit]
  patterns: [pipe-chain-execution, builtin-preset-seeding, redos-size-limit]

key-files:
  created:
    - src/domain/generation.rs
    - src/domain/regex_script.rs
    - src/db/generation.rs
    - src/db/regex_scripts.rs
    - src/service/regex.rs
  modified:
    - src/db/migrations.rs
    - src/db/mod.rs
    - src/db/models.rs
    - src/domain/mod.rs
    - src/service/mod.rs
    - src/app.rs

key-decisions:
  - "size_limit set to 1MB (1_000_000 bytes) per D-06 — catches huge NFA automatons while allowing common patterns like (\\w+)"
  - "Regex crate provides bounded backtracking by design — size_limit prevents memory DoS, not CPU DoS"
  - "Built-in presets use fixed UUIDs (builtin-creative, builtin-balanced, builtin-precise) for cross-import stability"
  - "INSERT OR IGNORE for seed_builtin_presets ensures idempotency"

patterns-established:
  - "Domain model pattern: Clone, Debug, Serialize, Deserialize, PartialEq with uuid v4 and NaiveDateTime"
  - "CRUD pattern: prepare_cached with params!, JSON serialization for Vec fields, upsert via exists-check"
  - "Pipe chain execution: scripts sorted by sort_order, each output becomes next input"
  - "Built-in protection: update/delete reject is_builtin=true with descriptive errors"

requirements-completed: [GENR-01, GENR-04, RGXO-01, RGXO-03]

# Metrics
duration: 8min
completed: 2026-05-08
---

# Phase 6 Plan 1: Generation Controls Regex Data Summary

**GenerationPreset and RegexScript domain models, M9 migration, CRUD with built-in seeding, and ReDoS-safe regex validation/execution service**

## Performance

- **Duration:** 8 min
- **Started:** 2026-05-08T05:33:55Z
- **Completed:** 2026-05-08T05:41:42Z
- **Tasks:** 2
- **Files modified:** 11

## Accomplishments
- GenerationPreset domain model with Creative/Balanced/Precise built-in constructors and fixed UUIDs
- RegexScript domain model with SillyTavern extensions.regex_scripts JSON conversion
- Migration M9 creates generation_presets, regex_scripts tables, adds default_preset_id and generation_preset_id columns
- Full CRUD operations for both models following lorebook.rs pattern
- Built-in preset seeding with INSERT OR IGNORE (idempotent, fixed UUIDs)
- Read-only enforcement on built-in presets (update/delete return errors per D-14)
- ReDoS validation via regex::RegexBuilder::size_limit(1MB) per D-06
- Regex execution pipeline with sort_order pipe chain per D-05
- 40 new tests (26 DB + 14 service), all 256 total tests pass

## Task Commits

Each task was committed atomically following TDD RED-GREEN cycle:

1. **Task 1 RED: Domain models, DB migration M9, and CRUD operations** - `3b2f643` (test)
2. **Task 1 GREEN: Implement generation preset and regex script CRUD** - `6557be7` (feat)
3. **Task 2 RED: Regex validation service and execution pipeline** - `ccfc5be` (test)
4. **Task 2 GREEN: Implement regex validation service and execution pipeline** - `b78f3b1` (feat)

_Note: TDD tasks produced RED → GREEN commits. No REFACTOR commits needed — code follows existing patterns cleanly._

## Files Created/Modified
- `src/domain/generation.rs` - GenerationPreset model with new(), creative(), balanced(), precise() constructors
- `src/domain/regex_script.rs` - RegexScript model with from_extensions_json()/to_extensions_json() for SillyTavern
- `src/db/generation.rs` - Preset CRUD: seed, insert, load, load_by_id, update (rejects builtin), delete (rejects builtin)
- `src/db/regex_scripts.rs` - Script CRUD: insert, load, load_by_id, update, delete with CASCADE support
- `src/db/migrations.rs` - Migration M9: generation_presets, regex_scripts tables, FK columns
- `src/db/models.rs` - row_to_generation_preset(), row_to_regex_script() helpers
- `src/db/mod.rs` - DbInput/DbOutput variants for presets and regex scripts, seed_builtin_presets in Worker init
- `src/service/regex.rs` - validate_regex_pattern(), validate_regex_flags(), apply_regex_scripts(), apply_single_regex()
- `src/domain/mod.rs` - Added generation and regex_script modules
- `src/service/mod.rs` - Added regex module
- `src/app.rs` - Added presets, regex_scripts, current_regex_character_id fields and DbOutput match arms

## Decisions Made
- Used 1MB size_limit for regex validation — the regex crate prevents catastrophic backtracking by design; size_limit protects against memory DoS from huge compiled automatons. Common patterns like `(\w+)` compile well within 1MB while deeply nested pathological patterns exceed it.
- Fixed UUIDs for built-in presets ("builtin-creative", "builtin-balanced", "builtin-precise") ensure stability across import/export and repeated seeding
- INSERT OR IGNORE for seeding ensures idempotency — safe to call on every app startup
- Built-in preset read-only enforcement returns descriptive errors (not silent failure) per D-14

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed ReDoS test pattern — (a+)+$ doesn't exceed regex crate's size_limit**
- **Found during:** Task 2 (Regex validation service GREEN phase)
- **Issue:** The plan suggested testing `(a+)+$` as a ReDoS pattern, but the regex crate's size_limit controls compiled NFA size, not runtime backtracking. The regex crate provides bounded backtracking by design, so `(a+)+$` compiles to a small automaton and doesn't exceed the limit.
- **Fix:** Changed ReDoS test to use deeply nested pattern `(((((...)a?)a?)a?...)` with 500 levels, which creates an exponentially large NFA that exceeds the 1MB size_limit. Kept the production size_limit at 1MB per D-06.
- **Files modified:** src/service/regex.rs
- **Verification:** test_validate_redos_pattern passes, all 14 service tests pass, all 256 tests pass
- **Committed in:** b78f3b1 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Minimal — the behavior is functionally equivalent; the test pattern was adjusted to actually trigger the protection mechanism.

## Issues Encountered
- Initially tried 10KB size_limit which was too restrictive — `(\w+)` exceeded it. Reverted to 1MB as per D-06/research. The 1MB limit correctly allows typical user patterns while blocking pathological ones.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- GenerationPreset and RegexScript domain models ready for UI integration (Plan 06-02)
- ReDoS validation service ready for regex editor integration (Plan 06-02)
- Built-in preset seeding ensures presets exist when UI loads
- DbInput/DbOutput variants and match arms ready for UI dispatch

---
*Phase: 06-generation-controls-regex-data*
*Completed: 2026-05-08*

## Self-Check: PASSED

- All 5 created files found on disk
- All 4 commit hashes found in git log
- All 256 tests pass (0 failures)
