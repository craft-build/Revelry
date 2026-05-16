---
phase: 05-live-conversations
plan: 01
subsystem: ai-service
tags: [rig-core, streaming, tauri-channel, provider-dispatch]
dependency_graph:
  requires:
    [src-tauri/src/domain/provider.rs, src-tauri/src/domain/conversation.rs]
  provides:
    [
      AiService,
      StreamChunk,
      GenerationParams,
      classify_error,
      from_provider_config,
    ]
  affects: [src-tauri/Cargo.toml, src-tauri/src/lib.rs]
tech_stack:
  added: [rig-core 0.37, futures 0.3, tokio-util 0.7]
  patterns:
    [
      Tauri Channel serde-tagged enum,
      rig-core 0.37 builder pattern,
      CancellationToken + tokio::select!,
    ]
key_files:
  created: [src-tauri/src/ai/mod.rs]
  modified: [src-tauri/Cargo.toml, src-tauri/src/lib.rs]
decisions:
  - D-01: Use builder pattern for all rig-core 0.37 provider clients (not from_val/new)
  - D-02: rig-core 0.37 lib name is rig_core (not rig) — API break from 0.36
  - D-03: StreamAssistantItem(Text) pattern still used in 0.37 (RESEARCH incorrectly said MultiTurnStreamItem::Text)
  - D-04: CompletionClient trait must be imported for .agent() method on provider clients
  - D-05: Domain Message aliased as DomainMessage to avoid name clash with rig_core::completion::Message
metrics:
  duration: 522s
  completed: 2026-05-16
  tasks: 2/2
  tests: 20
---

# Phase 5 Plan 01: Port AiService from gtk_ref with rig-core 0.37 Summary

AiService ported from gtk_ref with rig-core 0.37 streaming, Tauri Channel-compatible StreamChunk, all 15 provider type dispatch, cancellation support, and 20 passing tests.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] rig-core 0.37 lib name is `rig_core`, not `rig`**

- **Found during:** Task 1 — initial compilation
- **Issue:** The RESEARCH.md and plan assumed `use rig::...` imports, but rig-core 0.37 renamed its lib from `rig` to `rig_core`
- **Fix:** Changed all `rig::` references to `rig_core::` throughout ai/mod.rs
- **Files modified:** src-tauri/src/ai/mod.rs
- **Commit:** 4d9857e

**2. [Rule 1 - Bug] rig-core 0.37 streaming API unchanged from 0.36**

- **Found during:** Task 1 — source code inspection of rig-core 0.37
- **Issue:** RESEARCH.md claimed `MultiTurnStreamItem::Text(chunk)` replaces `StreamAssistantItem`, but the actual 0.37 API still uses `StreamAssistantItem(StreamedAssistantContent::Text(Text{ text }))`
- **Fix:** Used the correct `StreamAssistantItem(StreamedAssistantContent::Text(text))` pattern matching on `text.text`
- **Files modified:** src-tauri/src/ai/mod.rs
- **Commit:** 4d9857e

**3. [Rule 3 - Blocking] rig-core 0.37 requires CompletionClient trait import for .agent()**

- **Found during:** Task 1 — compilation errors
- **Issue:** The `agent()` method is on the `CompletionClient` trait, not inherent on the client struct. Must `use rig_core::client::CompletionClient;`
- **Fix:** Added `CompletionClient` import
- **Files modified:** src-tauri/src/ai/mod.rs
- **Commit:** 4d9857e

**4. [Rule 3 - Blocking] rig-core 0.37 provider clients use builder pattern**

- **Found during:** Task 1 — type mismatch errors with from_val
- **Issue:** Several providers (Gemini, Cohere, DeepSeek, OpenRouter) expect custom ApiKey types (BearerAuth, GeminiApiKey) not String. The builder pattern's `.api_key(&str)` method handles this uniformly.
- **Fix:** Switched all provider client construction to builder pattern: `Client::builder().api_key(api_key).build()`
- **Files modified:** src-tauri/src/ai/mod.rs
- **Commit:** 4d9857e

**5. [Rule 1 - Bug] Message name clash between domain and rig_core**

- **Found during:** Task 1 — compilation error E0252
- **Issue:** Both `crate::domain::conversation::Message` and `rig_core::completion::Message` are needed, causing a name clash
- **Fix:** Aliased domain Message as `DomainMessage` in the import
- **Files modified:** src-tauri/src/ai/mod.rs
- **Commit:** 4d9857e

### Design Decisions

**6. Used macro for repetitive agent-building pattern**

- **Found during:** Task 1 — reducing code duplication in 15-arm dispatch
- **Decision:** Created `build_agent_and_stream!` macro to DRY the repetitive agent→preamble→temperature→max_tokens→additional_params→build→stream pattern
- **Rationale:** Each provider arm follows identical parameter wiring; macro reduces ~10 lines per arm

## Key Decisions

| Decision                                  | Choice                                                            | Rationale                                                               |
| ----------------------------------------- | ----------------------------------------------------------------- | ----------------------------------------------------------------------- |
| rig-core 0.37 lib name                    | `rig_core`                                                        | Crate renamed its lib from `rig` to `rig_core` in 0.37                  |
| Provider client construction              | Builder pattern                                                   | Uniform `.api_key(&str)` interface avoids type-specific ApiKey wrappers |
| Streaming pattern                         | `StreamAssistantItem(StreamedAssistantContent::Text(Text{text}))` | 0.37 API is unchanged from 0.36 despite RESEARCH claiming otherwise     |
| Domain Message alias                      | `DomainMessage`                                                   | Avoids name clash with `rig_core::completion::Message`                  |
| `from_provider_config` max_tokens mapping | `i64` → `u64` with 0→None                                         | Domain uses i64 (nullable), ai module uses u64 (0 means unset)          |

## Test Results

```
running 20 tests
test ai::tests::test_split_last_prompt_empty ... ok
test ai::tests::test_split_last_prompt_single ... ok
test ai::tests::test_split_last_prompt_multiple ... ok
test ai::tests::test_provider_supports_param_openai ... ok
test ai::tests::test_provider_supports_param_anthropic ... ok
test ai::tests::test_provider_supports_param_gemini ... ok
test ai::tests::test_provider_supports_param_cohere ... ok
test ai::tests::test_classify_error_context_length ... ok
test ai::tests::test_classify_error_authentication ... ok
test ai::tests::test_classify_error_rate_limited ... ok
test ai::tests::test_classify_error_network ... ok
test ai::tests::test_classify_error_provider_default ... ok
test ai::tests::test_build_additional_params_provider_filtering ... ok
test ai::tests::test_stream_chunk_serde_token ... ok
test ai::tests::test_stream_chunk_serde_complete ... ok
test ai::tests::test_stream_chunk_serde_error ... ok
test ai::tests::test_generation_params_default ... ok
test ai::tests::test_from_provider_config_no_defaults ... ok
test ai::tests::test_from_provider_config_with_defaults ... ok
test ai::tests::test_from_provider_config_zero_max_tokens ... ok

test result: ok. 20 passed; 0 failed
```

## Deferred Issues

None — all compilation and test issues were resolved inline.

## Threat Flags

No new threat surface beyond what's in the plan's threat model. `classify_error()` strips error details before sending to frontend, partially addressing T-05-01.

## Self-Check: PASSED

- All 4 key files verified present
- Both commit hashes (4d9857e, bed7f40) found in git log
- 20/20 tests passing
- Full crate compiles with `cargo check -p revelry` exiting 0
