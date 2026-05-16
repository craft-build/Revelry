# Phase 6: Smart Prompts — Research

**Date:** 2026-05-16
**Status:** Complete
**Phase requirements:** PRMP-01, PRMP-02, PRMP-03

---

## Research Questions & Answers

### Q1: What are the three canonical Rust files to port, and what do they contain?

**Answer:** Three files from `gtk_ref/src/service/` must be ported. Together they form a self-contained prompt assembly pipeline:

| File             | LOC  | Purpose                                                                                                                                                       | Key Types                                                                                                    |
| ---------------- | ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------ |
| `token_count.rs` | ~140 | tiktoken-rs cl100k_base tokenizer, 10% safety margin, lazy Mutex BPE, 3-tier context window resolution                                                        | `TokenCountService`                                                                                          |
| `template.rs`    | ~180 | Template variable resolution (`{{user}}`, `{{char}}`, `{{persona.*}}`), persona block builder, 4-tier compression (Full/Trimmed/Minimal/Essential)            | `PersonaCompressionTier`, `resolve_variables()`, `build_persona_block()`, `build_persona_block_compressed()` |
| `prompt.rs`      | ~280 | Full PromptEngine with priority-layer assembly (sticky → lorebook → persona → history → post-history), budget-aware truncation, history oldest-first dropping | `PromptEngine`, `AssembledPrompt`                                                                            |

All three have comprehensive test suites in their respective `#[cfg(test)]` modules. The port is mostly mechanical — type and import adjustments for the Tauri project structure.

### Q2: Where should the new service files live in the Tauri backend?

**Answer:** The Tauri backend currently has no `service/` directory. The existing `ai/` module already contains the AiService. Two options:

1. **Create `src-tauri/src/service/`** — new module mirroring gtk_ref structure with `mod.rs`, `token_count.rs`, `template.rs`, `prompt.rs`. Clean separation, scales well for Phase 9 (lorebook service, regex service).

2. **Add to `src-tauri/src/ai/`** — merge PromptEngine alongside AiService. Keeps AI-related code together but makes the ai module large.

**Recommendation:** Create `src-tauri/src/service/` module. This follows the gtk_ref pattern exactly, keeps the ai/ module focused on provider dispatch and streaming, and provides a natural home for Phase 9's lorebook and regex services. Requires adding `mod service;` to `lib.rs`.

### Q3: What is the exact current system prompt assembly code that gets replaced?

**Answer:** The current inline system prompt builder appears in THREE places in `src-tauri/src/commands/chat.rs` — identical code duplicated in `start_chat`, `regenerate`, and `edit_and_resend`:

```rust
let system_prompt = match &character {
    Some(ch) => {
        let parts: Vec<&str> = [&ch.personality, &ch.scenario]
            .iter()
            .filter(|s| !s.is_empty())
            .map(|s| s.as_str())
            .collect();
        if parts.is_empty() {
            "You are a helpful assistant.".to_string()
        } else {
            parts.join("\n\n")
        }
    }
    None => "You are a helpful assistant.".to_string(),
};
```

This minimal builder only uses personality + scenario, ignoring: system_prompt, description, post_history_instructions, persona, and all budget logic. All three instances must be replaced with PromptEngine calls. The refactoring also eliminates the triplication by extracting a shared helper.

### Q4: What additional data does PromptEngine need that start_chat doesn't currently load?

**Answer:** Currently `start_chat` loads: conversation, messages, character, provider. PromptEngine additionally needs:

| Data                                             | How to load                                  | Already has DbCommand? |
| ------------------------------------------------ | -------------------------------------------- | ---------------------- |
| Persona (from `conversation.creator_persona_id`) | LoadPersonas → find by id                    | Yes                    |
| Lorebook entries (Phase 9, but structure needed) | LoadLorebookEntries by character's lorebooks | Yes                    |
| Context window size                              | From ProviderConfig (already loaded)         | N/A                    |

The persona load is the critical addition for Phase 6. Lorebook entries should be loaded as an empty vec for now (Phase 9 fills this in). The `TokenCountService::resolve_context_window_size()` handles the 3-tier fallback from the already-loaded ProviderConfig.

### Q5: How does budget info flow from PromptEngine to the frontend context bar?

**Answer (per D-02, D-03):** Budget info rides on the existing StreamChunk channel — no new Tauri command needed. The approach:

1. `AssembledPrompt` already contains `context_utilization: f64`, `truncated_messages: usize`, and the preamble.
2. After PromptEngine assembles, the backend sends a new `StreamChunk::ContextBudget` variant (or extends `StreamChunk::Complete`) with `{ tokensUsed, tokensLimit, truncatedCount }`.
3. Per D-03, the token bar refreshes only after AI generation completes (or regenerates). The frontend updates AppState from the `complete` event.

**Implementation options for budget transport:**

- **Option A:** Add a `StreamChunk::ContextBudget` variant sent before `StreamChunk::Token` events. Frontend stores it when received.
- **Option B:** Add budget fields to `StreamChunk::Complete` — `tokensUsed`, `tokensLimit`, `truncatedCount` alongside existing `messageId`, `content`, `stopReason`.

**Recommendation:** Option B (extend Complete). Budget info is most useful at completion time (D-03: refreshes after generation completes), and this avoids introducing a new event type that the frontend must handle across all streaming paths. The Complete event is already the natural place to finalize the token display.

### Q6: What frontend state and UI changes are needed?

**Answer:** Three files need modification:

**1. `src/lib/stores/app.svelte.ts` — Add context_budget state**

```typescript
// New $state field:
context_budget = $state<{ tokensUsed: number; tokensLimit: number; truncatedCount: number } | null>(null);

// Update in StreamChunk::Complete handler:
case "complete":
  // ... existing logic ...
  this.context_budget = {
    tokensUsed: chunk.data.tokensUsed,
    tokensLimit: chunk.data.tokensLimit,
    truncatedCount: chunk.data.truncatedCount,
  };
  break;
```

**2. `src/lib/components/chat/MessageInput.svelte` — Replace hardcoded context bar**

Replace the current `usedContext`/`contextLimit`/`contextPct` derived values with data from `app.context_budget`. Color logic per D-01:

- Green: < 80%
- Yellow: 80-95%
- Red: > 95%

The existing progress bar markup stays, but the data source and color class become dynamic.

**3. `src/lib/types/index.ts` — Extend StreamChunk type**

Add `tokensUsed`, `tokensLimit`, `truncatedCount` to the `complete` variant:

```typescript
| { event: "complete"; data: { messageId: string; content: string; stopReason: string; tokensUsed: number; tokensLimit: number; truncatedCount: number } }
```

### Q7: What Cargo.toml dependency is needed?

**Answer:** Add `tiktoken-rs = "0.11"` to `[dependencies]`. This is the same version used in gtk_ref. It provides:

- `cl100k_base()` — returns `CoreBPE` tokenizer (GPT-4 compatible)
- `CoreBPE::count_with_special_tokens()` — token counting method

The lazy initialization pattern (Mutex<Option<CoreBPE>>) avoids the upfront cost of loading the BPE model until first token count request.

### Q8: What are the key differences between gtk_ref and Tauri domain types that affect the port?

**Answer:** Several minor but important type differences:

| Aspect                   | gtk_ref                         | Tauri backend                                     | Adjustment needed                                           |
| ------------------------ | ------------------------------- | ------------------------------------------------- | ----------------------------------------------------------- |
| Timestamps               | `NaiveDateTime`                 | `DateTime<Utc>`                                   | No impact — PromptEngine only uses message content and role |
| Message type name        | `Message`                       | `Message` (aliased as `DomainMessage` in chat.rs) | Use the same import alias pattern                           |
| Message role enum        | `MessageRole`                   | `MessageRole`                                     | Same variants — compatible                                  |
| Persona type             | `Persona` (gtk_ref)             | `Persona` (src-tauri)                             | Same fields — compatible                                    |
| Character type           | `CharacterCardV2`               | `CharacterCardV2`                                 | Same fields — compatible                                    |
| Chat history format      | `Vec<rig::message::Message>`    | `Vec<rig::message::Message>`                      | Same — PromptEngine builds rig messages                     |
| rig-core version         | 0.36                            | 0.37                                              | Different API — use 0.37 conventions                        |
| rig message construction | `rig::message::Message::user()` | Same but from `rig_core`                          | Import path differs                                         |

The PromptEngine's `build_chat_history()` function currently uses `rig::message::Message` which needs to be `rig_core::message::Message` (the existing ai/mod.rs already uses this import).

### Q9: What is the persona loading path and how does conversation → persona resolution work?

**Answer:** The persona resolution chain:

1. `start_chat` loads the `Conversation` (already does this)
2. `Conversation.creator_persona_id: Option<String>` holds the persona ID
3. Currently `start_chat` does NOT load the persona — this is a new data fetch
4. Load via `DbCommand::LoadPersonas` → find by id (same pattern as character loading)
5. The persona is passed to `PromptEngine::assemble_with_budget()` as `Option<&Persona>`

The conversation's `creator_persona_id` is set when the user selects a persona via the persona switcher (already functional in Phase 2). If None, PromptEngine operates without persona (no persona block injected).

### Q10: What is the shared helper extraction pattern for the three chat commands?

**Answer:** Currently `start_chat`, `regenerate`, and `edit_and_resend` each independently load conversation, character, provider, API key, and build the system prompt. With Phase 6, each also needs to load persona and call PromptEngine.

**Recommended refactoring:**

```rust
/// Shared context for prompt assembly
struct ChatContext {
    character: Option<CharacterCardV2>,
    persona: Option<Persona>,
    provider: ProviderConfig,
    messages: Vec<Message>,
    api_key: String,
}

/// Loads all context needed for prompt assembly and streaming.
async fn load_chat_context(db: &DbWorker, conversation_id: &str) -> Result<ChatContext, String> {
    // Load conversation, character, persona, provider, messages, API key
    // All in one place — eliminates triplication
}
```

This reduces the ~60-line duplicated blocks to a single function call in each command. The planner can decide whether to extract this as a standalone function or a method.

### Q11: What are the Phase 9 forward-compatibility requirements?

**Answer:** Per CONTEXT.md: "Lorebook integration is Phase 9, but PromptEngine references lorebook in assemble_with_budget(). For Phase 6, pass empty lorebook entries."

Specific forward-compat requirements:

1. `assemble_with_budget()` signature must accept `lorebook_entries: &[LorebookEntry]` — pass `&[]` for now
2. `AssembledPrompt.active_lorebook_count` will always be 0 in Phase 6
3. The `lorebook::activate_lorebook_entries()` call in `assemble_with_budget()` should be gated or the lorebook module stubbed. **Simpler option:** extract the prompt assembly logic without the lorebook layer activation call (since passing empty entries produces no activations anyway), then add it back in Phase 9 when the lorebook service is ported.

**Recommendation:** Port `prompt.rs` with lorebook awareness but pass empty `&[]` entries. The `lorebook::activate_lorebook_entries` call will return empty results with empty input — no code gating needed. The `lorebook` module import will need a Phase 9 stub or the lorebook call can be commented out with a TODO. The planner decides the cleanest approach.

### Q12: What is the context window size resolution flow?

**Answer:** Three-tier resolution implemented in `TokenCountService::resolve_context_window_size()`:

1. **User override:** `provider.context_window_size` (per-provider override in settings) — if `Some(n)` and `n > 0`, use it
2. **Built-in table:** Substring match `provider.model_name` against known models (gpt-4o → 128000, claude-sonnet-4 → 200000, etc.)
3. **Default fallback:** 8192 (conservative default)

The existing `ProviderConfig` already has `context_window_size: Option<i64>` and `models: Vec<ProviderModelEntry>` with per-model `context_window_size`. The built-in table from gtk_ref needs updating for 2026 model names but the structure is correct.

**Note:** The model-level context window (from `ProviderModelEntry.context_window_size`) should also be checked. Currently `resolve_context_window_size` only checks the provider-level override. The planner should decide whether to add a 4th tier: user override → per-model entry → built-in table → default. This gives finer-grained control when providers have multiple models with different context limits.

---

## Risks & Pitfalls

### R1: tiktoken-rs compilation on cross-platform targets

tiktoken-rs includes native C code. It uses the `bundled` feature pattern. Need to verify it compiles cleanly on the Tauri target triple for macOS (already working) and that it doesn't add significant build time.

### R2: StreamChunk type change is a breaking wire change

Adding fields to `StreamChunk::Complete` changes the TypeScript discriminated union. Both Rust serde and TypeScript type must be updated in the same plan. The frontend currently destructures `chunk.data.content`, `chunk.data.messageId`, `chunk.data.stopReason` — adding three more fields is backward-compatible (TypeScript just ignores extra fields in the union).

### R3: Token counting is approximate for non-OpenAI models

cl100k_base is the GPT-4 tokenizer. Anthropic, Gemini, local models use different tokenizers. The 10% safety margin helps but is not perfect. This is the same approach as gtk_ref — documented as a known limitation, not a blocker.

### R4: The three chat commands are ~200 lines each with heavy duplication

The shared context extraction (Q10) is important for maintainability and for avoiding bugs where one command gets the PromptEngine update but another doesn't. The planner should make this extraction a priority early in the implementation.

---

## File Map Summary

### New files (port from gtk_ref)

| File                                   | Source                                         |
| -------------------------------------- | ---------------------------------------------- |
| `src-tauri/src/service/mod.rs`         | New — module declarations                      |
| `src-tauri/src/service/token_count.rs` | Port from `gtk_ref/src/service/token_count.rs` |
| `src-tauri/src/service/template.rs`    | Port from `gtk_ref/src/service/template.rs`    |
| `src-tauri/src/service/prompt.rs`      | Port from `gtk_ref/src/service/prompt.rs`      |

### Modified files (backend)

| File                             | Change                                                                                                                                                  |
| -------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `src-tauri/Cargo.toml`           | Add `tiktoken-rs = "0.11"`                                                                                                                              |
| `src-tauri/src/lib.rs`           | Add `mod service;`                                                                                                                                      |
| `src-tauri/src/commands/chat.rs` | Extract shared helper, replace inline prompt builder with PromptEngine calls in all 3 commands, load persona, send budget info in StreamChunk::Complete |
| `src-tauri/src/ai/mod.rs`        | Update StreamChunk::Complete to include budget fields                                                                                                   |

### Modified files (frontend)

| File                                          | Change                                                                                 |
| --------------------------------------------- | -------------------------------------------------------------------------------------- |
| `src/lib/types/index.ts`                      | Extend StreamChunk complete variant with `tokensUsed`, `tokensLimit`, `truncatedCount` |
| `src/lib/stores/app.svelte.ts`                | Add `context_budget` state, update from complete events                                |
| `src/lib/components/chat/MessageInput.svelte` | Replace hardcoded context bar with real budget data + color coding                     |
| `src/lib/utils/index.ts`                      | Keep `estimateTokens()` as fallback or remove (planner decision)                       |

### Unchanged files

All other files — domain models, DB layer, API layer, other UI components. The change is well-contained.

---

## Dependencies & Integration Points

```
token_count.rs  ←  tiktoken-rs crate, domain::provider::ProviderConfig
template.rs     ←  domain::character::CharacterCardV2, domain::persona::Persona
prompt.rs       ←  token_count, template, domain::{character, persona, conversation, lorebook}
commands/chat.rs ←  prompt.rs (PromptEngine), token_count.rs (for context window resolution)
ai/mod.rs       ←  StreamChunk enum extended
```

The dependency chain is linear: tiktoken → token_count → template → prompt → chat commands. No circular dependencies.

---

_Research completed: 2026-05-16_
