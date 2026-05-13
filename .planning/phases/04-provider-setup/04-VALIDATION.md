---
phase: 4
slug: provider-setup
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-05-13
---

# Phase 4 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property               | Value                                                            |
| ---------------------- | ---------------------------------------------------------------- |
| **Framework**          | Rust: cargo test (built-in); Svelte: Vitest (not yet configured) |
| **Config file**        | Cargo.toml (Rust); none for Svelte yet                           |
| **Quick run command**  | `cargo test -p revelry --lib 2>&1 \| head -30`                   |
| **Full suite command** | `cargo test -p revelry`                                          |
| **Estimated runtime**  | ~5 seconds (Rust unit); ~10 seconds (full suite)                 |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p revelry --lib`
- **After every plan wave:** Run `cargo test -p revelry`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID  | Plan | Wave | Requirement                                 | Threat Ref                                     | Secure Behavior                                             | Test Type                                                  | Automated Command                                          | File Exists | Status     |
| -------- | ---- | ---- | ------------------------------------------- | ---------------------------------------------- | ----------------------------------------------------------- | ---------------------------------------------------------- | ---------------------------------------------------------- | ----------- | ---------- |
| 04-01-01 | 01   | 1    | PROV-04                                     | T-4-01                                         | API keys stored only in OS keychain, never in SQLite        | unit                                                       | `cargo test -p revelry --lib test_no_api_key_column`       | ✅ existing | ⬜ pending |
| 04-01-02 | 01   | 1    | PROV-04                                     | T-4-02                                         | Keyring save/load/delete round-trip works                   | unit                                                       | `cargo test -p revelry --lib test_keyring_roundtrip`       | ❌ W0       | ⬜ pending |
| 04-01-03 | 01   | 1    | PROV-04                                     | T-4-03                                         | Keyring availability check works (create+delete test entry) | unit                                                       | `cargo test -p revelry --lib test_keyring_available`       | ❌ W0       | ⬜ pending |
| 04-01-04 | 01   | 1    | PROV-01, PROV-05, PROV-06                   | —                                              | ProviderConfig serde roundtrip with snake_case ProviderType | unit                                                       | `cargo test -p revelry --lib test_provider_type_serde`     | ❌ W0       | ⬜ pending |
| 04-01-05 | 01   | 1    | PROV-05                                     | —                                              | Conversation model_name column exists                       | unit                                                       | `cargo test -p revelry --lib test_conversation_model_name` | ❌ W0       | ⬜ pending |
| 04-01-06 | 01   | 1    | PROV-06                                     | —                                              | Generation params stored/retrieved                          | unit                                                       | `cargo test -p revelry --lib test_generation_params`       | ❌ W0       | ⬜ pending |
| 04-01-07 | 01   | 1    | PROV-01, PROV-02, PROV-03                   | —                                              | Provider CRUD with models JSON column                       | unit                                                       | `cargo test -p revelry --lib test_provider_crud`           | ❌ W0       | ⬜ pending |
| 04-01-08 | 01   | 1    | PROV-01, PROV-02, PROV-03, T-4-01           | Tauri commands for provider CRUD + keyring ops | integration                                                 | `cargo test -p revelry --lib test_tauri_provider_commands` | ❌ W0                                                      | ⬜ pending  |
| 04-02-01 | 02   | 2    | PROV-01, PROV-05, PROV-06                   | —                                              | TypeScript types, API wrappers, AppState provider actions   | type-check                                                 | `bun run check`                                            | —           | ⬜ pending |
| 04-02-02 | 02   | 2    | PROV-01, PROV-02, PROV-03, PROV-04, PROV-06 | T-4-04                                         | Provider editor dialog saves/edits/deletes                  | manual                                                     | UI inspection                                              | —           | ⬜ pending |
| 04-02-03 | 02   | 2    | PROV-05                                     | —                                              | Provider/model selector in ChatHeader updates conversation  | manual                                                     | UI inspection                                              | —           | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave 0 Requirements

- [ ] `src-tauri/src/db/providers.rs` — test provider CRUD with models JSON column
- [ ] `src-tauri/src/domain/provider.rs` — test ProviderType serde roundtrip with snake_case
- [ ] `src-tauri/src/secrets/mod.rs` — test KeyringStore save/load/delete (requires OS keychain)
- [ ] `src-tauri/src/db/migrations.rs` — add test for migration #3 columns on providers and conversations
- [ ] `src-tauri/src/db/models.rs` — test row_to_provider mapper with models JSON column

---

## Manual-Only Verifications

| Behavior                                                     | Requirement      | Why Manual                                          | Test Instructions                                                                                                                                                                                                               |
| ------------------------------------------------------------ | ---------------- | --------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Provider editor dialog opens, auto-fills defaults, and saves | PROV-01, PROV-02 | UI interaction requires visual inspection           | 1. Open Settings → Providers 2. Click "Add provider" 3. Select type "OpenAI" 4. Verify base_url auto-fills 5. Enter API key 6. Click "Save provider" 7. Verify card appears with "Key set ✓"                                    |
| Keyring banner shows when keyring unavailable                | PROV-04, D-09    | Requires Linux headless or keyring service disabled | 1. On Linux without GNOME Keyring, open Settings 2. Verify banner is visible 3. Verify API key field disabled 4. Verify Ollama provider can still be created                                                                    |
| API key never in Tauri IPC payload                           | PROV-04          | Security audit requires network/IPC inspection      | 1. Use browser DevTools Network tab 2. Create/edit a provider with API key 3. Inspect invoke payloads 4. Verify no `api_key` string in any payload except `save_api_key` command                                                |
| Delete provider removes keyring entry                        | PROV-03          | Cross-system cleanup verification                   | 1. Create provider with API key 2. Delete provider via kebab menu 3. Check OS keychain (Keychain Access on macOS) for orphaned entries                                                                                          |
| Provider/model selector in ChatHeader                        | PROV-05          | UI interaction requires visual inspection           | 1. Open a conversation 2. Verify provider select is visible in ChatHeader 3. Select a provider 4. Verify model select populates with enabled models 5. Select a model 6. Verify conversation provider_id and model_name updated |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 10s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
