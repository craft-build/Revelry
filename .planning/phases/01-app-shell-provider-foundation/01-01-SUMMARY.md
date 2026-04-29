---
phase: 01-app-shell-provider-foundation
plan: 01
subsystem: database
tags: [rust, sqlite, rusqlite, relm4-worker, provider-types, serde, uuid]

# Dependency graph
requires:
  - phase: init
    provides: Project structure, STACK.md with version pins
provides:
  - "Cargo.toml with all 25+ Phase 1 dependencies at exact versions"
  - "ProviderType enum with 15 provider variants, serde, requires_api_key(), default_base_url()"
  - "ProviderConfig struct with UUID v4 id, timestamps, serde"
  - "AppSetting struct for key-value settings"
  - "SQLite schema: providers + app_settings tables with WAL mode"
  - "DbService Worker with DbInput/DbOutput message pattern"
  - "Provider CRUD operations using prepare_cached()"
affects: [01-02, 01-03]

# Tech tracking
tech-stack:
  added: [gtk4-0.11.3, libadwaita-0.9.1, relm4-0.11.0, rig-core-0.36.0, rusqlite-0.39.0, rusqlite_migration-2.5.0, keyring-3.6.3, serde-1.0.228, uuid-1.23.1, chrono-0.4.44, thiserror-2.0.18, anyhow-1.0.102, tokio-1.52, glib-0.22.6, glib-build-tools-0.22.0]
  patterns: [relm4-worker-background-db, feature-based-module-organization, timestamp-as-text-in-sqlite, prepare-cached-for-crud, provider-type-as-serde-json-text]

key-files:
  created: [Cargo.toml, build.rs, src/main.rs, src/lib.rs, src/domain/mod.rs, src/domain/provider.rs, src/domain/app_settings.rs, src/db/mod.rs, src/db/migrations.rs, src/db/models.rs, src/db/providers.rs, src/secrets/mod.rs, src/ui/mod.rs, .gitignore]
  modified: []

key-decisions:
  - "Stored NaiveDateTime as TEXT in SQLite (rusqlite doesn't implement FromSql for NaiveDateTime)"
  - "ProviderType stored as serde_json TEXT string in database (variant name serialized)"
  - "DbService::SaveProvider does upsert (insert or update) based on existing record"
  - "WAL mode + busy_timeout=5000 set in migration SQL for file-based databases"
  - "AzureOpenAI and Custom return None for default_base_url (require explicit URL)"

patterns-established:
  - "Relm4 Worker pattern: DbService owns Connection on background thread, processes DbInput messages serially"
  - "Timestamp storage: NaiveDateTime → TEXT string for SQLite storage, parsed back on read"
  - "Provider type storage: Enum → serde_json string for SQLite TEXT column, deserialized on read"
  - "Security invariant: No api_key column in providers table (keys only in OS keychain)"
  - "CRUD query pattern: prepare_cached() for all frequently-executed queries"

requirements-completed: [PROV-01, PROV-05]

# Metrics
duration: 12min
completed: 2026-04-29
---

# Phase 1 Plan 1: Project Scaffold & Database Layer Summary

**Rust project scaffold with 25+ dependencies, ProviderType/ProviderConfig domain types, SQLite schema with WAL mode, and DbService Worker with provider CRUD**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-29T05:22:18Z
- **Completed:** 2026-04-29T05:34:57Z
- **Tasks:** 2
- **Files modified:** 14

## Accomplishments
- Created complete Rust project with all Phase 1 dependencies at exact versions from STACK.md
- Implemented ProviderType enum with 15 variants, requires_api_key(), and default_base_url() for all providers
- Implemented ProviderConfig with UUID v4 id generation and serde serialization
- Created SQLite schema with providers and app_settings tables
- Implemented DbService Worker for background-thread database operations
- Provider CRUD operations with prepare_cached() for query efficiency
- WAL mode and busy_timeout=5000 set in migration SQL
- No api_key column in providers table (security invariant PROV-05)

## Task Commits

Each task was committed atomically:

1. **Task 1 (RED): Add failing tests for domain types** - `ffe8d53` (test)
2. **Task 1 (GREEN): Implement domain types with correct behavior** - `882ddc5` (feat)
3. **Task 2: Implement DbService Worker with migrations and provider CRUD** - `b799533` (feat)
4. **Chore: Add .gitignore** - `ca29d8f` (chore)

## Files Created/Modified
- `Cargo.toml` - Project manifest with 25+ dependencies at exact versions from STACK.md
- `build.rs` - Placeholder for glib-build-tools resource compilation
- `src/main.rs` - Minimal entry point (println for now, RelmApp in Plan 02)
- `src/lib.rs` - Module declarations: domain, db, secrets, ui
- `src/domain/mod.rs` - Domain module exports
- `src/domain/provider.rs` - ProviderType enum (15 variants), ProviderConfig struct, 24 unit tests
- `src/domain/app_settings.rs` - AppSetting struct (key, value)
- `src/db/mod.rs` - DbService Worker, DbInput/DbOutput enums
- `src/db/migrations.rs` - SQLite migrations with WAL, busy_timeout, foreign_keys, 9 tests
- `src/db/models.rs` - row_to_provider (row → ProviderConfig), provider_to_params
- `src/db/providers.rs` - Provider CRUD (insert, load, load_by_id, update, delete), app settings CRUD, 11 tests
- `src/secrets/mod.rs` - Placeholder KeyringStore struct (Plan 03)
- `src/ui/mod.rs` - Placeholder (Plan 02)
- `.gitignore` - Rust project gitignore (target/, IDE, macOS)

## Decisions Made
- Stored NaiveDateTime as TEXT in SQLite because rusqlite doesn't implement FromSql for chrono types; parse on read via format string "%Y-%m-%d %H:%M:%S%.f"
- ProviderType stored as serde_json TEXT string in database rather than a simple name match — serde ensures roundtrip correctness
- DbService::SaveProvider performs upsert (checks existence, then insert or update) rather than requiring separate calls
- AzureOpenAI and Custom return None for default_base_url — they require explicit URL configuration from the user
- WAL mode test accommodates in-memory databases returning "memory" instead of "wal"

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Upgraded Rust toolchain from 1.92.0 to 1.95.0**
- **Found during:** Task 1 (cargo check)
- **Issue:** relm4 0.11.0 requires rustc 1.93+; project had 1.92.0
- **Fix:** Ran `rustup update stable` to upgrade to 1.95.0
- **Files modified:** None (toolchain only)
- **Verification:** `cargo check` passes, `rustc --version` shows 1.95.0
- **Committed in:** ffe8d53 (before any code changes were committed)

**2. [Rule 1 - Bug] Fixed WAL mode test for in-memory SQLite**
- **Found during:** Task 2 (cargo test --lib db)
- **Issue:** In-memory SQLite databases always return "memory" for journal_mode, not "wal". Test assertion was too strict.
- **Fix:** Updated test to accept either "wal" or "memory" as valid journal_mode values, added explicit migration SQL check
- **Files modified:** src/db/migrations.rs
- **Verification:** All 20 DB tests pass
- **Committed in:** b799533 (part of Task 2 commit)

**3. [Rule 2 - Missing Critical] Added .gitignore for Rust project**
- **Found during:** Post-task check for untracked files
- **Issue:** No .gitignore existed; target/ directory and other generated files would be tracked
- **Fix:** Created .gitignore with Rust-specific exclusions (target/, IDE, macOS, .env)
- **Files modified:** .gitignore
- **Verification:** `git status --short | grep '^??'` no longer shows target/
- **Committed in:** ca29d8f

---

**Total deviations:** 3 auto-fixed (1 blocking, 1 bug, 1 missing critical)
**Impact on plan:** All auto-fixes necessary for correctness and build health. No scope creep.

## Issues Encountered
- NaiveDateTime doesn't implement rusqlite's FromSql trait, requiring TEXT storage with manual parse — this is a known pattern in Rust/SQLite projects and is well-documented

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Domain types and DB Worker are ready for Plan 02 (App Shell with Relm4 UI components)
- Provider CRUD and app_settings CRUD are ready for Plan 03 (Keyring secrets integration)
- The `src/secrets/mod.rs` placeholder (KeyringStore) will be replaced in Plan 03

## Self-Check: PASSED

- 14/14 files exist on disk
- 4/4 commits found in git log (ffe8d53, 882ddc5, b799533, ca29d8f)
- 44/44 unit tests pass (24 domain + 20 db)
- 0 api_key references in schema SQL (PROV-05 security invariant verified)
- WAL mode pragma present in migration SQL (2 occurrences)

---
*Phase: 01-app-shell-provider-foundation*
*Completed: 2026-04-29*
