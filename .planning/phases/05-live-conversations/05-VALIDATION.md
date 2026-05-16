---
phase: 5
slug: live-conversations
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-05-15
---

# Phase 5 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property               | Value                                                            |
| ---------------------- | ---------------------------------------------------------------- |
| **Framework**          | Rust: cargo test (built-in); Svelte: svelte-check + bun run check |
| **Config file**        | Cargo.toml (Rust); tsconfig.json (Svelte)                        |
| **Quick run command**  | `cargo test -p revelry_lib --lib ai:: 2>&1`                      |
| **Full suite command** | `cargo test -p revelry_lib`                                      |
| **Estimated runtime**  | ~5 seconds (Rust unit); ~3 seconds (svelte-check)                |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p revelry_lib --lib` (backend tasks) or `bun run check` (frontend tasks)
- **After every plan wave:** Run `cargo test -p revelry_lib` + `bun run check`
- **Before `/gsd-verify-work`:** Full suite must be green
- **Max feedback latency:** 10 seconds

---

## Per-Task Verification Map

| Task ID  | Plan | Wave | Requirement                                 | Threat Ref | Secure Behavior                                                    | Test Type   | Automated Command                                                        | File Exists | Status     |
| -------- | ---- | ---- | ------------------------------------------- | ---------- | ------------------------------------------------------------------ | ----------- | ------------------------------------------------------------------------ | ----------- | ---------- |
| 05-01-01 | 01   | 1    | CHAT-01                                     | T-05-01    | StreamChunk serde produces correct "event"/"data" JSON structure   | unit        | `cargo test -p revelry_lib --lib ai::test_stream_chunk_serde`             | ❌ W0       | ⬜ pending |
| 05-01-01 | 01   | 1    | CHAT-01                                     | —          | provider_supports_param filters Anthropic/Gemini/Cohere penalties  | unit        | `cargo test -p revelry_lib --lib ai::test_provider_supports_param`       | ❌ W0       | ⬜ pending |
| 05-01-01 | 01   | 1    | CHAT-01                                     | —          | split_last_prompt handles edge cases (empty, single, multiple)     | unit        | `cargo test -p revelry_lib --lib ai::test_split_last_prompt`             | ❌ W0       | ⬜ pending |
| 05-01-01 | 01   | 1    | CHAT-01                                     | —          | build_additional_params filters per provider type                 | unit        | `cargo test -p revelry_lib --lib ai::test_build_additional_params`       | ❌ W0       | ⬜ pending |
| 05-01-01 | 01   | 1    | CHAT-02                                     | T-05-01    | classify_error maps all 5 error categories correctly             | unit        | `cargo test -p revelry_lib --lib ai::test_classify_error`                | ❌ W0       | ⬜ pending |
| 05-01-02 | 01   | 1    | CHAT-01                                     | —          | Full crate compiles with ai module registered                    | compile     | `cargo check -p revelry_lib`                                             | —           | ⬜ pending |
| 05-02-01 | 02   | 2    | CHAT-03, CHAT-04                            | —          | delete_message removes row from DB                                | unit        | `cargo test -p revelry_lib --lib db::conversations::test_delete_message`  | ❌ W0       | ⬜ pending |
| 05-02-01 | 02   | 2    | CHAT-04                                     | —          | delete_messages_after_sort_order truncates and updates preview    | unit        | `cargo test -p revelry_lib --lib db::conversations::test_delete_messages_after` | ❌ W0 | ⬜ pending |
| 05-02-01 | 02   | 2    | CHAT-04                                     | —          | update_message_content changes message text in DB                 | unit        | `cargo test -p revelry_lib --lib db::conversations::test_update_message_content` | ❌ W0 | ⬜ pending |
| 05-02-02 | 02   | 2    | CHAT-01                                     | —          | start_chat command compiles with message_id + sort_order params   | compile     | `cargo check -p revelry_lib`                                             | —           | ⬜ pending |
| 05-02-02 | 02   | 2    | CHAT-02                                     | T-05-05    | CancelState enforces single active stream                         | unit        | `cargo test -p revelry_lib --lib commands::chat::test_cancel_state`      | ❌ W0       | ⬜ pending |
| 05-03-01 | 03   | 3    | CHAT-01                                     | —          | TypeScript types: StreamChunk discriminated union + error_type     | type-check  | `bun run check`                                                          | —           | ⬜ pending |
| 05-03-01 | 03   | 3    | CHAT-01                                     | —          | AppState.startChat creates Channel, passes messageId + sortOrder  | type-check  | `bun run check`                                                          | —           | ⬜ pending |
| 05-03-02 | 03   | 3    | CHAT-01, CHAT-02                            | —          | Mock streaming removed, handleSend calls app.startChat            | type-check  | `bun run check`                                                          | —           | ⬜ pending |
| 05-04-01 | 04   | 4    | CHAT-02, CHAT-03                            | T-05-09    | Error bubbles render with destructive styling, AlertTriangle      | type-check  | `bun run check`                                                          | —           | ⬜ pending |
| 05-04-01 | 04   | 4    | CHAT-02, CHAT-03                            | —          | focusProviderSelector signal triggers ChatHeader focus             | type-check  | `bun run check`                                                          | —           | ⬜ pending |
| 05-04-02 | 04   | 4    | CHAT-03                                     | —          | isLastAiMessage prop computed and passed correctly                | type-check  | `bun run check`                                                          | —           | ⬜ pending |
| 05-05-01 | 05   | 5    | CHAT-04                                     | —          | Edit mode state toggles, Save & resend calls app.editResend       | type-check  | `bun run check`                                                          | —           | ⬜ pending |
| 05-05-02 | 05   | 5    | CHAT-01, CHAT-02, CHAT-03, CHAT-04          | —          | End-to-end streaming, stop, regenerate, edit/resend              | manual      | UI inspection per checkpoint                                            | —           | ⬜ pending |

_Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky_

---

## Wave 0 Requirements

- [ ] `src-tauri/src/ai/mod.rs` — new file: AiService, StreamChunk, GenerationParams, provider_supports_param, classify_error, split_last_prompt, build_additional_params, stream_with_agent tests
- [ ] `src-tauri/src/commands/chat.rs` — new file: start_chat, stop_generation, regenerate, edit_and_resend commands (CancelState test)
- [ ] `src-tauri/src/db/conversations.rs` — add delete_message, delete_messages_after_sort_order, update_message_content with tests
- [ ] `src-tauri/Cargo.toml` — add rig-core = "0.37", futures = "0.3", tokio-util with rt feature, tokio macros feature

---

## Manual-Only Verifications

| Behavior                                         | Requirement            | Why Manual                          | Test Instructions                                                                                                                                                                                                                       |
| ------------------------------------------------ | ---------------------- | ----------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Streaming AI response token-by-token in chat     | CHAT-01                | Requires real AI API key + provider | 1. Open a conversation with provider/model configured 2. Type a message and press Enter 3. Verify AI response streams token-by-token with typewriter animation 4. Verify streaming caret ▍ appears during generation                    |
| Stop generation mid-stream                       | CHAT-02                | Requires active streaming to cancel  | 1. Send a message 2. Click Stop button (red square) mid-stream 3. Verify partial response is finalized with "Stopped" badge                                                                                                             |
| Inline error message rendering                   | CHAT-02, D-07          | Requires provider error condition   | 1. Set an invalid API key 2. Send a message 3. Verify error bubble appears with destructive styling, AlertTriangle icon, Retry/Switch provider links 4. Verify "Switch provider" focuses ProviderModelSelector in ChatHeader          |
| Regenerate last AI response                      | CHAT-03                | Requires completed AI response      | 1. Send a message and wait for completion 2. Hover over last AI message 3. Verify Regenerate icon appears 4. Click it 5. Verify old response replaced with new stream                                                                    |
| Edit/resend user message                         | CHAT-04                | Requires interactive edit flow      | 1. Hover over a user message 2. Click Edit 3. Verify bubble becomes textarea with message content 4. Modify text 5. Click "Save & resend" 6. Verify subsequent messages removed, new AI response streams 7. Verify Escape cancels edit |
| Input disabled during generation                 | CHAT-02                | UI state verification               | 1. Send a message 2. Verify textarea is disabled while generating 3. Verify textarea re-enables after generation completes                                                                                                              |
| ProviderModelSelector focus via error link       | CHAT-02, UI-SPEC       | Cross-component signal interaction | 1. Trigger an error (invalid API key) 2. Click "Switch provider" link in error bubble 3. Verify ProviderModelSelector dropdown in ChatHeader opens/focuses                                                                              |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 10s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
