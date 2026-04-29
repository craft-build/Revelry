---
phase: 01-app-shell-provider-foundation
plan: 03
subsystem: settings-ui
tags: [rust, gtk4, libadwaita, relm4, keyring, rig, provider-config, settings-panel]

# Dependency graph
requires:
  - phase: 01
    plan: 01
    provides: ProviderType, ProviderConfig, DbService Worker, DbInput/DbOutput
  - phase: 01
    plan: 02
    provides: AppModel shell, ChatPanel, CharactersPanel
provides:
  - "KeyringStore: OS keychain integration for API key save/load/delete"
  - "provider_test service: 15-provider connectivity test via rig with 10s timeout"
  - "SettingsPanel: provider list with Add/Edit/Delete/Test actions"
  - "ProviderConfigPage: dynamic form in AdwPreferencesWindow"
  - "Delete confirmation dialog with keyring cleanup"
  - "Toast feedback for all operations"
  - "First launch auto-redirect to Settings"
  - "AppModel-SettingsPanel wiring via output messages"
affects: []

# Tech tracking
tech-stack:
  added: [keyring-3.6.3, aes-gcm-0.10.3, argon2-0.5.3, pulldown-cmark-0.13.3, tiktoken-rs-0.11.0]
  patterns: [keyring-background-thread, provider-test-via-rig-agent, command-output-pattern, output-message-delegation, dynamic-form-by-provider-type, toast-overlay-feedback]

key-files:
  created: [src/secrets/mod.rs, src/service/mod.rs, src/service/provider_test.rs]
  modified: [src/ui/settings/mod.rs, src/ui/mod.rs, src/app.rs, src/lib.rs]

key-decisions:
  - id: D-03-01
    decision: "Use macro-based approach for provider test functions instead of generic helper"
    rationale: "rig's CompletionClient trait causes E0310/E0311 lifetime issues when used as generic parameter in a helper function. Macro-based approach generates per-provider functions that monomorphize correctly."
  - id: D-03-02
    decision: "SettingsPanel communicates with DB via output messages to AppModel, not WorkerController clone"
    rationale: "WorkerController<DbService> does not implement Clone. All DB/keyring operations route through AppModel which owns the WorkerController."
  - id: D-03-03
    decision: "Use Component trait (not SimpleComponent) for SettingsPanel and ProviderConfigPage"
    rationale: "Component trait provides CommandOutput/update_cmd for background thread results and widget references in update(). SimpleComponent lacks these."
  - id: D-03-04
    decision: "Merged ProviderConfigPage into settings/mod.rs instead of separate provider_page.rs"
    rationale: "ProviderConfigPage needs access to SettingsPanelMsg types for message forwarding. Single file avoids circular module references."

metrics:
  duration: "45min"
  completed: "2026-04-28"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 6
---

# Phase 01 Plan 03: Provider Config with Keyring & Settings UI Summary

KeyringStore for OS keychain API key storage, rig-based provider connectivity testing for 15 providers, and full SettingsPanel UI with provider configuration.

## One-liner
OS keychain API key storage + 15-provider rig connectivity test + SettingsPanel with dynamic AdwPreferencesWindow config

## What Was Done

### Task 1: KeyringStore and Provider Connectivity Test
- **KeyringStore** (`src/secrets/mod.rs`): `save_key()`, `load_key()`, `delete_key()` using keyring 3.6.3
- **Provider test service** (`src/service/provider_test.rs`): Connectivity test for all 15 provider types via rig-core
  - Macro-based `define_provider_test!` to avoid generic lifetime issues
  - Builder pattern for custom base URL providers (Ollama, Azure, Custom)
  - 10-second timeout via `tokio::time::timeout`
  - API key redaction in error messages (Bearer tokens, sk- prefixes)
  - 9 unit tests for result types, redaction, and validation

### Task 2: SettingsPanel with Provider List and Config Page
- **SettingsPanel** (`src/ui/settings/mod.rs`): 
  - Provider list with Add/Edit/Delete/Test buttons
  - AdwPreferencesGroup with AdwActionRow per provider
  - Delete confirmation via AdwMessageDialog
  - Toast overlay for operation feedback
  - Test state tracking (Idle/Testing/Success/Failure) per provider
  - KeyLoaded flow: parent loads key from keyring → settings panel runs test
- **ProviderConfigPage** (in `src/ui/settings/mod.rs`):
  - AdwPreferencesPage with AdwPreferencesWindow wrapper
  - AdwComboRow for 15 provider types
  - Dynamic form: Base URL row shown/hidden by provider type (D-08)
  - AdwPasswordEntryRow shown/hidden by requires_api_key() (D-08)
  - Test Connection button with spinner/result indicator
  - API key saved to keyring on background thread
- **AppModel wiring** (`src/app.rs`):
  - SettingsPanel launched with `init = ()`
  - All DB/keyring operations routed through AppModel via output messages
  - SettingsPanelOutput::SaveProvider → DbInput::SaveProvider
  - SettingsPanelOutput::DeleteProvider → DbInput::DeleteProvider
  - SettingsPanelOutput::DeleteKey → KeyringStore::delete_key (background thread)
  - SettingsPanelOutput::TestProvider → KeyringStore::load_key → SettingsPanelMsg::KeyLoaded
  - First launch detection: empty providers → auto-switch to Settings view

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Functionality] WorkerController clone not possible, redesigned communication pattern**
- **Found during:** Task 2
- **Issue:** Plan said to pass `WorkerController<DbService>` to SettingsPanel via init, but WorkerController doesn't implement Clone
- **Fix:** Changed SettingsPanel to emit `SettingsPanelOutput` messages that AppModel translates to `DbInput` messages. This is actually a better pattern — SettingsPanel is decoupled from DB implementation details.
- **Files modified:** src/ui/settings/mod.rs, src/app.rs
- **Commit:** fbb6941

**2. [Rule 1 - Bug] SimpleComponent doesn't support CommandOutput/update_cmd**
- **Found during:** Task 2
- **Issue:** Initially used SimpleComponent but needed `spawn_oneshot_command` which requires Component trait's `CommandOutput` type and `update_cmd` method
- **Fix:** Switched both SettingsPanel and ProviderConfigPage from SimpleComponent to Component trait. This adds the `&Self::Root` parameter to update/update_cmd.
- **Files modified:** src/ui/settings/mod.rs
- **Commit:** fbb6941

**3. [Rule 3 - Blocking] ProviderConfigPage in separate file causes circular module references**
- **Found during:** Task 2
- **Issue:** `provider_page.rs` needed `SettingsPanelMsg` types from parent module for message forwarding, creating awkward import paths
- **Fix:** Merged ProviderConfigPage into `settings/mod.rs`. All types are in the same module scope.
- **Files modified:** src/ui/settings/mod.rs (deleted provider_page.rs)
- **Commit:** fbb6941

**4. [Rule 1 - Bug] rig Client::new() returns Result, not infallible**
- **Found during:** Task 1
- **Issue:** Plan's code showed `openai::Client::new(key)` as infallible, but rig-core 0.36.0 returns `Result<Client, Error>`
- **Fix:** Used `match` in macro-generated test functions to handle Result, and `.map_err()` for builder-pattern functions
- **Files modified:** src/service/provider_test.rs
- **Commit:** 09ebc2c

**5. [Rule 2 - Security] API key redaction in error messages**
- **Found during:** Task 1
- **Issue:** rig/reqwest error messages may contain API keys in URLs or headers
- **Fix:** Added `redact_error()` function that strips Bearer tokens and sk- prefixes from error strings
- **Files modified:** src/service/provider_test.rs
- **Commit:** 09ebc2c

## Verification Results

| Check | Expected | Actual | Status |
|-------|----------|--------|--------|
| cargo build | Pass | Pass | ✅ |
| KeyringStore in secrets/mod.rs | ≥1 | 9 refs | ✅ |
| No api_key in DB schema | 0 (ignoring test) | 0 | ✅ |
| test_provider in provider_test.rs | ≥1 | 11 refs | ✅ |
| PreferencesWindow in settings | ≥1 | 4 refs | ✅ |
| ComboRow in settings | ≥1 | 1 ref | ✅ |
| Remove Provider dialog | ≥1 | 1 ref | ✅ |
| Toast messages | ≥3 | 3 refs | ✅ |
| All 56 tests pass | Pass | 56/56 | ✅ |
| No API key logging | 0 hits | 0 hits | ✅ |

## Threat Model Compliance

| Threat | Mitigation | Status |
|--------|-----------|--------|
| T-01-01: api_key in DB | No api_key column in providers table | ✅ Verified |
| T-01-06: Key disclosure | Keys only in OS keychain via keyring crate | ✅ Verified |
| T-01-07: Provider test DoS | 10-second timeout | ✅ Implemented |
| T-01-08: Connection spoofing | TLS via rig/reqwest | ✅ Inherited |
| T-01-09: Key logging | Never log keys, redact in errors | ✅ Verified |

## Known Stubs

None — all functionality is wired end-to-end.

## Threat Flags

None — no security-relevant surface beyond what's in the threat model.

## Self-Check: PASSED

All 7 key files verified present. Both task commits (09ebc2c, fbb6941) verified in git log.
