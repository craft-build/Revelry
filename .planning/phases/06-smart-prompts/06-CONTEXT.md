# Phase 6: Smart Prompts - Context

**Gathered:** 2026-05-16
**Status:** Ready for planning

<domain>
## Phase Boundary

AI conversations use budget-aware prompt assembly with real token counting so responses are contextually appropriate. This phase replaces the minimal Phase 5 prompt assembly (character personality + scenario only, no budget enforcement) with a full PromptEngine ported from gtk_ref that assembles system prompts from character card, active persona, and conversation history while respecting the model's context window budget. Real token counting via tiktoken-rs replaces the frontend's 4-char-per-token heuristic.

Covers requirements: PRMP-01 (system prompt assembly), PRMP-02 (budget-aware prompt), PRMP-03 (context utilization display).

NOT in scope: lorebook injection (Phase 9), regex scripts (Phase 9), conversation branching (Phase 8), markdown rendering (Phase 7), provider settings UI for context window overrides (Phase 10).

</domain>

<decisions>
## Implementation Decisions

### Context Utilization Display

- **D-01:** Enhanced context bar — same progress bar layout as current but with real model-specific context limits (not hardcoded 8192). Color shifts: green <80%, yellow 80–95%, red >95%. No extra text beyond tokens used / limit. Replaces the current `estimateTokens()` + hardcoded limit in `MessageInput.svelte`.

### Token Data Flow & Timing

- **D-02:** Context budget info (tokens used, limit, truncated count) returned as part of the start_chat / StreamChunk flow — no new dedicated Tauri command. The frontend updates the bar from the stream response data.
- **D-03:** Token bar refreshes only after AI generation completes (or regenerates). No real-time keystroke-level recalculation. Between messages the bar shows the last known value.

### Budget Overflow UX

- **D-04:** Silent auto-truncation — PromptEngine truncates oldest messages first and compresses persona tier (Full → Trimmed → Minimal → Essential) to fit within budget. The color-coded bar communicates pressure. Inline error message only when the prompt physically cannot fit even with maximum compression (e.g., sticky layer alone exceeds budget).

### Persona in Prompt Format

- **D-05:** Follow gtk_ref persona format exactly — structured block: `"You are interacting with {{user}}, who is:\nName: X\nPersonality: Y\n..."` with 4-tier compression. Port `template.rs` (resolve_variables, build_persona_block, build_persona_block_compressed, PersonaCompressionTier enum) from gtk_ref.

### Agent's Discretion

- Exact `StreamChunk` variant or response structure for returning budget info (planner decides API shape)
- Built-in model context window defaults table (port gtk_ref's table, update if models are outdated)
- tiktoken-rs integration approach (lazy init + Mutex as gtk_ref does, or alternative)
- Migration for any schema changes needed (e.g., if context_window_size storage needs adjustment)

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Reference Implementation (MUST read — patterns to port)

- `gtk_ref/src/service/prompt.rs` — Complete PromptEngine with assemble(), assemble_with_budget(), AssembledPrompt struct, priority layer ordering (sticky → lorebook → persona → history → post-history), history truncation, full test suite
- `gtk_ref/src/service/token_count.rs` — TokenCountService with tiktoken-rs cl100k_base, 10% safety margin, 3-tier context window resolution (user override → built-in table → default 8192), lazy BPE initialization
- `gtk_ref/src/service/template.rs` — Template variable resolution ({{user}}, {{char}}, {{persona.*}}), persona block builder, 4-tier persona compression (Full/Trimmed/Minimal/Essential), full test suite
- `gtk_ref/src/service/lorebook.rs` — Lorebook activation logic (for awareness — injection is Phase 9, but prompt.rs references it)

### Prior Phase Decisions (carry-forward)

- `.planning/phases/05-live-conversations/05-CONTEXT.md` — D-05/D-06/D-07 define the minimal prompt assembly this phase replaces. Streaming transport, message persistence timing, error handling patterns carry forward.
- `.planning/phases/04-provider-setup/04-CONTEXT.md` — ProviderConfig schema with context_window_size, ProviderModelEntry with per-model context_window_size, GenerationParams, keyring patterns.

### Active Backend (will be extended)

- `src-tauri/src/ai/mod.rs` — AiService with stream_chat(), provider dispatch macro, GenerationParams, StreamChunk enum. PromptEngine output feeds into the agent preamble + chat history here.
- `src-tauri/src/commands/chat.rs` — start_chat command. Currently builds minimal system prompt inline (character personality + scenario). Phase 6 replaces this with PromptEngine call.
- `src-tauri/src/domain/provider.rs` — ProviderConfig with context_window_size, ProviderModelEntry, GenerationParams. PromptEngine reads these for budget resolution.

### Active Frontend (will be modified)

- `src/lib/components/chat/MessageInput.svelte` — Current token display uses estimateTokens() with hardcoded 8192 limit. Phase 6 replaces with real backend data from stream response.
- `src/lib/utils/index.ts` — estimateTokens() function. Kept as fallback or removed in favor of backend data.
- `src/lib/stores/app.svelte.ts` — AppState with messages, active_conversation, updateStreamingMessage. Needs context budget state field.

### Project Planning

- `.planning/ROADMAP.md` — Phase 6 definition, success criteria, plan list
- `.planning/REQUIREMENTS.md` — PRMP-01, PRMP-02, PRMP-03 requirements
- `.planning/PROJECT.md` — Key decisions including "Preserve full gtk_ref feature set"

### Architecture Documentation

- `docs/` — Design spec v2.0 (may contain prompt assembly guidance)

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- **gtk_ref PromptEngine** (`gtk_ref/src/service/prompt.rs`): Complete, tested implementation. Port directly to `src-tauri/src/service/prompt.rs` (new file). Key types: AssembledPrompt, PromptEngine. Key methods: assemble(), assemble_initial(), assemble_with_budget().
- **gtk_ref TokenCountService** (`gtk_ref/src/service/token_count.rs`): tiktoken-rs cl100k_base with 10% safety margin, lazy init, Mutex-cached BPE. Port to `src-tauri/src/service/token_count.rs` (new file).
- **gtk_ref template.rs** (`gtk_ref/src/service/template.rs`): Variable resolution and persona compression. Port to `src-tauri/src/service/template.rs` (new file).
- **ProviderConfig context_window_size**: Already exists on both the provider level and per-model entry in `src-tauri/src/domain/provider.rs`. The 3-tier resolution from gtk_ref (user override → built-in table → default) can leverage these existing fields.
- **AiService + stream_chat**: The streaming pipeline already works. PromptEngine output (preamble + chat_history) feeds into the existing `build_agent_and_stream!` macro as system_prompt + history.

### Established Patterns

- **Service pattern**: gtk_ref uses a service module pattern (prompt.rs, token_count.rs, template.rs in `service/`). The Tauri backend doesn't have a `service/` directory yet — planner decides whether to create one or integrate into `ai/`.
- **DB access**: All DB access goes through DbWorker with oneshot channels. PromptEngine needs character, persona, messages, and lorebook — all available through existing DbCommand variants.
- **Streaming**: StreamChunk events sent via Tauri Channel. Budget info should be added to StreamChunk or returned alongside it.
- **Frontend state**: AppState uses $state fields. Adding a `context_budget` field (tokens_used, tokens_limit, truncated_count) to AppState follows existing patterns.

### Integration Points

- **start_chat command** (`src-tauri/src/commands/chat.rs`): The inline system prompt builder (lines that join personality + scenario) gets replaced with `PromptEngine::assemble_with_budget()` call. The assembled preamble feeds into AiService::stream_chat() as system_prompt.
- **MessageInput.svelte**: Replace `estimateTokens(text)` and hardcoded `contextLimit = 8192` with data from AppState's context budget field. Color logic: green <80%, yellow 80-95%, red >95%.
- **AppState** (`src/lib/stores/app.svelte.ts`): Add context_budget state field. Update from stream response after each generation.
- **Cargo.toml**: Add `tiktoken-rs` dependency.

### Constraints

- **tiktoken-rs is an approximation**: It uses cl100k_base (GPT-4 tokenizer) for all providers. Anthropic and other models use different tokenizers. The 10% safety margin helps compensate. This is the same approach as gtk_ref.
- **Lorebook integration**: PromptEngine references lorebook in assemble_with_budget(), but lorebook activation is Phase 9. For Phase 6, pass empty lorebook entries. The code structure should support lorebook when Phase 9 adds it.
- **Template variables**: {{user}}, {{char}}, {{persona.*}} resolution must happen before token counting so the budget calculation reflects the final prompt text.

</code_context>

<specifics>
## Specific Ideas

No specific requirements — follow gtk_ref reference implementation faithfully.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

_Phase: 06-smart-prompts_
_Context gathered: 2026-05-16_
