# Phase 5: Live Conversations - Context

**Gathered:** 2026-05-15
**Status:** Ready for planning

<domain>
## Phase Boundary

Users have real streaming AI conversations through characters, replacing mock responses. This phase delivers: porting AiService with rig-core streaming, Tauri Channel-based streaming from Rust to SvelteKit, replacing mock streaming with real AI invoke, stop generation, regenerate, and edit/resend.

In scope:

- AiService port from gtk_ref with rig-core provider dispatch (all 15 provider types)
- Tauri Channel-based streaming: start_chat command returns typed channel, StreamChunk events
- stop_generation Tauri command with tokio CancellationSender
- Replace mock streaming in chat/[id]/+page.svelte with real AI invoke
- Regenerate last AI response (replace in-place, same provider/model)
- Edit/resend user messages (truncate subsequent, re-stream from edit point)
- Inline error messages in chat for generation failures
- Message persistence: user msg saved immediately, AI msg saved on stream complete
- Minimal prompt assembly: character personality + scenario as system prompt, full chat history

NOT in scope: budget-aware prompt assembly (Phase 6), token counting (Phase 6), persona in prompt (Phase 6), lorebook injection (Phase 9), markdown rendering (Phase 7), conversation branching (Phase 8), card import/export (Phase 7).

</domain>

<decisions>
## Implementation Decisions

### Streaming Transport

- **D-01:** Use Tauri v2 Channels for streaming — request-scoped, typed, no cross-talk between concurrent streams. The start_chat Tauri command returns a Channel<StreamChunk> that the frontend listens to for Token/Complete/Error events. Closest to gtk_ref's GLib channel pattern but adapted for Tauri's primitives.
- **D-02:** Single start_chat Tauri command — takes conversation_id + message content, starts the AI stream, returns the channel. No separate prepare/send split.
- **D-03:** Explicit stop_generation Tauri command — Rust side holds an Option<CancellationSender>. stop_generation drops it, the spawned tokio task detects cancellation via tokio::select! and stops the stream. The partial response is finalized as a complete message.
- **D-04:** Message persistence timing — user message saved to DB immediately when sent. AI streaming message created in memory only (appended to AppState.messages but not persisted). When stream completes (StreamChunk::Complete) or stops, save the final AI message to DB. If app crashes mid-stream, AI response is lost but user message is preserved.

### Prompt Scope (Phase 5 only)

- **D-05:** Minimal prompt assembly for Phase 5 — system prompt = character personality + scenario only. Chat history = all messages (no budget enforcement). Persona, lorebook, post_history_instructions, token counting all deferred to Phase 6.
- **D-06:** Send all conversation messages in history — no hard cap or budget calculation. If context exceeds the model's limit, the API returns an error which is shown inline in the chat. Phase 6 adds proper budget-aware truncation.
- **D-07:** AI generation errors (context overflow, auth failure, network error) displayed as inline error messages in the chat — like a system message (e.g., "Generation failed: context length exceeded"). Not a toast or modal. User can retry or switch provider.

### Edit/Resend

- **D-08:** Truncate from edit point — when user edits a message and resends, all subsequent messages (AI responses and further exchanges) are deleted from both UI state and DB. New AI response streams from the edited point. This matches the standard chat app pattern (ChatGPT, SillyTavern).
- **D-09:** Edit trigger — hover action on MessageBubble. Add an "Edit" button (pencil icon) to the hover actions row on user messages. Clicking enters inline edit mode; user edits content then clicks a resend button. Consistent with existing hover action patterns.
- **D-10:** Edit/resend applies to user messages only. For AI messages, use Regenerate instead. This is the standard pattern — user owns their own messages, AI messages are corrected via regeneration.

### Regenerate

- **D-11:** Regenerate replaces the last AI message in-place — delete the old AI message from DB and UI, then stream a new response with the same conversation context. Conversation stays clean, no duplicate messages.
- **D-12:** Regenerate trigger — hover action on the last AI MessageBubble. Add a "Regenerate" button (refresh icon) that only appears on the most recent AI message. Consistent with the edit hover action pattern.
- **D-13:** Regenerate always uses the same provider/model that produced the original response. To switch provider, user changes it in ChatHeader's ProviderModelSelector first, then regenerates.

### Claude's Discretion

- Exact StreamChunk serialization format for Tauri Channel (JSON string vs structured object) — I will pick based on Tauri v2 Channel API constraints.
- Whether start_chat also saves the user message or if that's a separate save_message call — I will pick based on atomicity vs. simplicity tradeoff.
- How the inline error message is rendered in MessageBubble (distinct style, role, or a separate ErrorBubble component) — I will pick based on existing component patterns.
- Whether edit mode uses a textarea overlay on the existing MessageBubble or replaces the bubble with a dedicated edit component — I will pick based on UX ergonomics.
- Generation params resolution for Phase 5 — use provider-level defaults only. Per-conversation overrides and preset selection are Phase 6 territory.
- How AiService gets the API key at stream time — reads from KeyringStore inside the start_chat command handler, or requires the key to be passed in. I will pick based on the KeyringStore's sync API and Tauri's async command model.

</decisions>

<canonical_refs>

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Project Planning

- `.planning/ROADMAP.md` — Phase 5 definition, plans list (05-01 through 05-05), success criteria
- `.planning/REQUIREMENTS.md` — CHAT-01, CHAT-02, CHAT-03, CHAT-04 requirements
- `.planning/PROJECT.md` — Project context, constraints, tech stack decisions, key decisions table

### Archival Reference (MUST read — patterns to port)

- `gtk_ref/src/service/ai.rs` — AiService with StreamChunk enum, GenerationParams, provider dispatch for all 15 types, stream_with_agent generic helper, split_last_prompt, build_additional_params, provider_supports_param
- `gtk_ref/src/domain/provider.rs` — ProviderConfig domain model, ProviderType enum with requires_api_key(), default_base_url(), default_model_name()
- `gtk_ref/src/domain/conversation.rs` — Message, MessageRole domain models

### Prior Phase Decisions (carry-forward)

- `.planning/phases/04-provider-setup/04-CONTEXT.md` — D-01 through D-16 (keyring, provider CRUD, models JSON, conversation provider/model linking)
- `.planning/phases/02-persistent-data/02-CONTEXT.md` — D-05 through D-19 (DbWorker pattern, Tauri command design, type alignment, ISO timestamps)

### Active Backend (will be extended)

- `src-tauri/src/lib.rs` — Tauri command registration (will add start_chat, stop_generation commands)
- `src-tauri/Cargo.toml` — Dependencies (will add rig-core, futures, tokio features for streaming)
- `src-tauri/src/domain/mod.rs` — Domain module registry (will add ai module)
- `src-tauri/src/secrets/mod.rs` — KeyringStore (will be called from AiService to retrieve API keys)

### Active Frontend (will be modified)

- `src/routes/chat/[id]/+page.svelte` — Chat view with mock streaming (will replace with real AI invoke via Tauri Channel)
- `src/lib/stores/app.svelte.ts` — AppState (will add startChat, stopGeneration, regenerate, editResend actions)
- `src/lib/data/api.ts` — API layer (will add startChat, stopGeneration invoke wrappers with Channel support)
- `src/lib/components/chat/MessageBubble.svelte` — Message rendering (will add Edit and Regenerate hover actions, inline error rendering)
- `src/lib/components/chat/MessageInput.svelte` — Input with stop button (already has oncancel; minimal changes)
- `src/lib/components/chat/ChatHeader.svelte` — Already has ProviderModelSelector (minimal changes)
- `src/lib/types/index.ts` — TypeScript types (may add StreamChunk type for frontend channel handling)

### Architecture Documentation

- `.planning/codebase/ARCHITECTURE.md` — System overview, layers, data flow (reference implementation flow)
- `.planning/codebase/INTEGRATIONS.md` — AI provider integrations, streaming patterns, rig-core details
- `.planning/codebase/STACK.md` — Technology stack (rig-core, tokio, futures versions from gtk_ref)

</canonical_refs>

<code_context>

## Existing Code Insights

### Reusable Assets

- **AiService from gtk_ref** — Complete, tested AI service with provider dispatch for all 15 types, StreamChunk enum (Token/Complete/Error), GenerationParams normalization, provider_supports_param filtering. Port directly to `src-tauri/src/ai/mod.rs`. Key change: replace GLib channel output with Tauri Channel output + tokio CancellationSender.
- **KeyringStore** — Already ported in Phase 4 (`src-tauri/src/secrets/mod.rs`). AiService reads API keys from it at stream time.
- **ProviderConfig + ProviderType** — Already ported in Phase 4 (`src-tauri/src/domain/provider.rs`). AiService uses ProviderType to dispatch to the correct rig-core client.
- **DbWorker + save_message** — Message persistence already works. Phase 5 adds the timing logic: save user msg immediately, save AI msg on stream complete.
- **ProviderModelSelector** — Already wired in ChatHeader. User can switch provider/model before sending. Regenerate reads the conversation's current provider_id + model_name.
- **AppState streaming pattern** — appendMessage, updateStreamingMessage, is_generating already work. The mock streaming in +page.svelte demonstrates the exact update flow that real streaming will follow.
- **MessageInput stop button** — Already has oncancel wired to handleCancel. Minimal changes needed.

### Established Patterns

- **Tauri IPC via invoke()**: snake_case commands, async, return `Result<T, String>`. New commands follow the same pattern.
- **DbWorker background-thread**: All DB ops go through the mpsc worker. Message save goes through save_message command.
- **TypeScript type alignment**: Rust serde output must match TS interfaces (Phase 2 D-08).
- **Optimistic state updates**: AppState updates locally, persists to backend, reverts on failure.
- **Hover actions on MessageBubble**: Existing hover actions (copy, etc.) — edit and regenerate follow the same pattern.

### Integration Points

- **`src-tauri/src/lib.rs`**: Register new commands: `start_chat`, `stop_generation`, `edit_and_resend`, `regenerate`
- **`src-tauri/Cargo.toml`**: Add `rig-core = "0.36"`, `futures = "0.3"`, update `tokio` features to include `macros`, `sync`, `rt-multi-thread` (already has sync and rt-multi-thread, may need macros)
- **`src-tauri/src/ai/mod.rs`** (NEW): Port AiService from gtk_ref, adapt stream output for Tauri Channels + CancellationSender
- **`src/routes/chat/[id]/+page.svelte`**: Replace mock streaming setTimeout loop with Tauri Channel listener. handleSend calls app.startChat(), handleCancel calls app.stopGeneration().
- **`src/lib/stores/app.svelte.ts`**: Add startChat() action that calls invoke('start_chat'), receives Channel, listens for StreamChunk events, calls appendMessage/updateStreamingMessage accordingly. Add stopGeneration(), regenerate(), editResend() actions.
- **`src/lib/data/api.ts`**: Add startChat() wrapper that invokes start_chat and returns the Channel. Add stopGeneration, regenerate, editAndResend wrappers.
- **`src/lib/components/chat/MessageBubble.svelte`**: Add Edit hover action (user messages only), Regenerate hover action (last AI message only), inline error rendering.
- **`src-tauri/src/domain/mod.rs`**: Add `pub mod ai;`

### Constraints

- rig-core version must match gtk_ref (0.36.x) — same API, same provider dispatch.
- AiService runs in a tokio::spawn — must not block the Tauri async command handler. The start_chat command should spawn the AI task and immediately return the Channel.
- API key retrieval from KeyringStore is synchronous and blocking — must use tokio::task::spawn_blocking() inside the start_chat handler (same pattern as Phase 4's keyring operations).
- Message model is flat (no branching) — truncation on edit/resend is a simple DELETE WHERE sort_order > N.
- The Tauri Channel must be created on the Rust side and returned to the frontend — Tauri v2's Channel API creates the channel in the command and the frontend receives it as part of the invoke result.

</code_context>

<specifics>
## Specific Ideas

- The start_chat command signature: `start_chat(conversation_id: String, content: String, channel: Channel<StreamChunk>) -> Result<(), String>` — the channel is passed from the Rust side via the invoke return, and the frontend listens for events on it.
- CancellationSender can be stored in a `Mutex<Option<CancellationSender>>` managed state (similar to how DbWorker is managed). start_chat creates one, stop_generation drops it.
- For the inline error message, add a `message_type: 'error'` variant or use the existing `metadata` field on Message with a `is_error: true` flag. The MessageBubble renders error messages with a distinct style (red-tinted background, warning icon).
- Edit mode: when the Edit hover action is clicked, replace the MessageBubble content area with a textarea pre-filled with the message content. Show a "Resend" button (and a "Cancel" button) below the textarea. On resend, call app.editResend(messageId, newContent) which truncates and re-streams.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

_Phase: 05-Live Conversations_
_Context gathered: 2026-05-15_
