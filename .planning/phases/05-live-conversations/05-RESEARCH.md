# Phase 5: Live Conversations - Research

**Researched:** 2026-05-15
**Domain:** AI streaming infrastructure (rig-core + Tauri v2 Channel IPC + frontend streaming state machine)
**Confidence:** HIGH

## Summary

Phase 5 replaces mock streaming with real AI inference through the rig-core provider abstraction, piped from Rust to the SvelteKit frontend via Tauri v2's `Channel<StreamChunk>` IPC primitive. The archival `AiService` from `gtk_ref/src/service/ai.rs` is the direct porting target, but must be adapted for three critical changes: (1) rig-core has been updated from 0.36.x (gtk_ref) to 0.37.0 (current), with a **breaking API change** in `MultiTurnStreamItem` variants; (2) the GLib channel output is replaced with Tauri `Channel<StreamChunk>` using the `tag = "event", content = "data"` serde pattern; (3) cancellation uses `tokio::sync::watch` / `CancellationToken` instead of GLib main-loop integration.

The frontend changes are well-scoped: replace the `setTimeout` mock loop in `chat/[id]/+page.svelte` with a `Channel` listener, add `startChat`/`stopGeneration`/`regenerate`/`editResend` actions to AppState, and extend `MessageBubble` with edit mode, error rendering, and conditional regenerate visibility. The `05-UI-SPEC.md` provides a comprehensive design contract that resolves all of the "Claude's Discretion" items from CONTEXT.md.

**Primary recommendation:** Port `AiService` as a new `src-tauri/src/ai/mod.rs` module using rig-core 0.37.0 (NOT 0.36.x), use Tauri's typed `Channel<StreamChunk>` for streaming, and store the `CancellationToken` in Tauri managed state for stop_generation. The frontend Channel listener replaces the mock loop 1:1.

## Architectural Responsibility Map

| Capability                       | Primary Tier                                                 | Secondary Tier        | Rationale                                                          |
| -------------------------------- | ------------------------------------------------------------ | --------------------- | ------------------------------------------------------------------ |
| AI inference / provider dispatch | Rust backend (src-tauri)                                     | —                     | rig-core runs in Rust; AiService is backend-only                   |
| Streaming transport              | Tauri IPC layer (Channel)                                    | —                     | Tauri v2 Channel bridges Rust→JS; both sides participate           |
| Stream event handling            | Frontend (AppState)                                          | —                     | Frontend accumulates tokens, updates UI, manages state transitions |
| Message persistence              | Rust backend (DbWorker)                                      | —                     | save_message goes through DbWorker per established pattern         |
| Prompt assembly (minimal)        | Rust backend (AiService)                                     | —                     | System prompt built from character data loaded via DbWorker        |
| Cancellation                     | Rust backend (CancellationToken) + Frontend (stopGeneration) | —                     | Backend holds token, frontend invokes stop_generation command      |
| Edit/resend truncation           | Rust backend (DbWorker)                                      | Frontend (UI removal) | DB deletes messages > edit point; frontend removes from state      |
| Error display                    | Frontend (MessageBubble)                                     | —                     | Inline error rendering per UI-SPEC contract                        |

<user_constraints>

## User Constraints (from CONTEXT.md)

### Locked Decisions

- **D-01:** Use Tauri v2 Channels for streaming — request-scoped, typed, no cross-talk between concurrent streams. The start_chat Tauri command returns a Channel<StreamChunk> that the frontend listens to for Token/Complete/Error events.
- **D-02:** Single start_chat Tauri command — takes conversation_id + message content, starts the AI stream, returns the channel. No separate prepare/send split.
- **D-03:** Explicit stop_generation Tauri command — Rust side holds an Option<CancellationSender>. stop_generation drops it, the spawned tokio task detects cancellation via tokio::select! and stops the stream. The partial response is finalized as a complete message.
- **D-04:** Message persistence timing — user message saved to DB immediately when sent. AI streaming message created in memory only (appended to AppState.messages but not persisted). When stream completes (StreamChunk::Complete) or stops, save the final AI message to DB. If app crashes mid-stream, AI response is lost but user message is preserved.
- **D-05:** Minimal prompt assembly for Phase 5 — system prompt = character personality + scenario only. Chat history = all messages (no budget enforcement). Persona, lorebook, post_history_instructions, token counting all deferred to Phase 6.
- **D-06:** Send all conversation messages in history — no hard cap or budget calculation. If context exceeds the model's limit, the API returns an error which is shown inline in the chat.
- **D-07:** AI generation errors displayed as inline error messages in the chat — like a system message. Not a toast or modal. User can retry or switch provider.
- **D-08:** Truncate from edit point — when user edits a message and resends, all subsequent messages are deleted from both UI state and DB.
- **D-09:** Edit trigger — hover action on MessageBubble. Add an "Edit" button (pencil icon) to the hover actions row on user messages.
- **D-10:** Edit/resend applies to user messages only. For AI messages, use Regenerate instead.
- **D-11:** Regenerate replaces the last AI message in-place — delete the old AI message from DB and UI, then stream a new response with the same conversation context.
- **D-12:** Regenerate trigger — hover action on the last AI MessageBubble.
- **D-13:** Regenerate always uses the same provider/model that produced the original response.

### the agent's Discretion

- Exact StreamChunk serialization format for Tauri Channel (JSON string vs structured object) — resolved by Tauri v2 Channel API: use `#[serde(tag = "event", content = "data")]` tagged enum pattern.
- Whether start_chat also saves the user message or if that's a separate save_message call — RESOLVED: start_chat saves the user message atomically using frontend-provided message_id and sort_order (see Q3 resolution).
- How the inline error message is rendered in MessageBubble — resolved by UI-SPEC: conditional rendering within MessageBubble using destructive-tinted styling, not a separate ErrorBubble component.
- Whether edit mode uses a textarea overlay on the existing MessageBubble or replaces the bubble with a dedicated edit component — resolved by UI-SPEC: inline textarea overlay replacing bubble content.
- Generation params resolution for Phase 5 — resolved by UI-SPEC: use provider-level defaults only.
- How AiService gets the API key at stream time — recommend: read from KeyringStore inside start_chat via spawn_blocking (same pattern as Phase 4 provider commands).

### Deferred Ideas (OUT OF SCOPE)

None — discussion stayed within phase scope.

</user_constraints>

<phase_requirements>

## Phase Requirements

| ID      | Description                                                                                          | Research Support                                                                                                            |
| ------- | ---------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| CHAT-01 | User can send a message and receive a streaming token-by-token AI response with typewriter animation | AiService port from gtk_ref + Tauri Channel streaming + frontend Channel listener pattern                                   |
| CHAT-02 | User can stop generation mid-stream                                                                  | CancellationSender in Tauri managed state + stop_generation command + tokio::select! in stream loop                         |
| CHAT-03 | User can regenerate the last AI response                                                             | regenerate command: delete last AI message, re-invoke start_chat with same provider/model + remaining history               |
| CHAT-04 | User can edit a sent message and resend from that point                                              | edit_and_resend command: update message content, DELETE messages WHERE sort_order > N, re-invoke start_chat from edit point |

</phase_requirements>

## Standard Stack

### Core (Rust Backend)

| Library  | Version        | Purpose                                         | Why Standard                                                                                          |
| -------- | -------------- | ----------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| rig-core | 0.37.0         | Provider-agnostic AI abstraction with streaming | Unified clients for 15+ providers; gtk_ref used 0.36.x but 0.37.0 is current [VERIFIED: cargo search] |
| tokio    | 1.x (existing) | Async runtime + CancellationToken               | Already in Cargo.toml; needs `macros` feature added for `#[tokio::main]` in tests                     |
| futures  | 0.3.x          | StreamExt for processing rig streaming          | Required by rig-core's streaming API; gtk_ref dependency                                              |

### Core (Frontend)

| Library         | Version | Purpose                         | Why Standard                                                                                      |
| --------------- | ------- | ------------------------------- | ------------------------------------------------------------------------------------------------- |
| @tauri-apps/api | ^2.11.0 | Channel class for streaming IPC | Already in package.json; provides `Channel<T>` for typed event listening [VERIFIED: package.json] |

### Supporting

| Library    | Version          | Purpose                                  | When to Use                                                                |
| ---------- | ---------------- | ---------------------------------------- | -------------------------------------------------------------------------- |
| serde      | 1.x (existing)   | Serialize StreamChunk for Channel        | Already in Cargo.toml; used for `#[derive(Serialize)]` on StreamChunk enum |
| serde_json | 1.x (existing)   | JSON serialization of StreamChunk events | Already in Cargo.toml                                                      |
| chrono     | 0.4.x (existing) | Timestamps on messages                   | Already in Cargo.toml                                                      |
| uuid       | 1.x (existing)   | Message ID generation                    | Already in Cargo.toml                                                      |

### Alternatives Considered

| Instead of        | Could Use                           | Tradeoff                                                                                   |
| ----------------- | ----------------------------------- | ------------------------------------------------------------------------------------------ |
| Tauri Channel     | Tauri Events (listen/emit)          | Events are global (cross-talk risk); Channel is request-scoped and typed — D-01 locks this |
| rig-core 0.36.x   | rig-core 0.37.0                     | Must use 0.37.0 — current version. API changed (see Pitfall 1)                             |
| CancellationToken | tokio::sync::mpsc sender for cancel | CancellationToken is the standard tokio pattern; D-03 specifies CancellationSender         |

**Installation:**

```bash
# Add to src-tauri/Cargo.toml
# rig-core = "0.37"
# futures = "0.3"
# tokio features: add "macros" to existing features
```

**Version verification:**

- rig-core: 0.37.0 [VERIFIED: `cargo search rig-core` 2026-05-15]
- @tauri-apps/api: ^2.11.0 [VERIFIED: package.json]
- tokio: 1.x already in Cargo.toml [VERIFIED: Cargo.toml]

## Architecture Patterns

### System Architecture Diagram

```
┌────────────────────────────────────────────────────────────────────┐
│                     FRONTEND (SvelteKit)                          │
│                                                                    │
│  User types in MessageInput                                        │
│       │                                                            │
│       ▼                                                            │
│  app.startChat(content) ──► invoke('start_chat', { convId,        │
│       │                      content, onEvent: Channel })          │
│       │                                                            │
│       ▼                                                            │
│  Channel<StreamChunk> listener                                     │
│       │                                                            │
│       ├── on Token ──► updateStreamingMessage(aiId, acc+token)     │
│       ├── on Complete ──► updateStreamingMessage(aiId, full, done) │
│       │                  saveMessage to DB via invoke              │
│       └── on Error ──► append error Message with stop_reason:error │
│                                                                    │
│  app.stopGeneration() ──► invoke('stop_generation')               │
│  app.regenerate() ──► delete last AI msg + invoke('regenerate')   │
│  app.editResend(msgId, newContent) ──► invoke('edit_and_resend') │
└───────────────────────────────┬────────────────────────────────────┘
                                │ Tauri IPC (invoke + Channel)
┌───────────────────────────────▼────────────────────────────────────┐
│                     RUST BACKEND (src-tauri)                        │
│                                                                    │
│  start_chat command:                                               │
│    1. spawn_blocking: KeyringStore::load_key(provider_id)         │
│    2. DbWorker: load provider + character + messages               │
│    3. Build system prompt from character personality + scenario    │
│    4. Convert domain messages → rig::message::Message             │
│    5. Store CancellationToken in managed state                     │
│    6. tokio::spawn: AiService::stream_chat() → rig stream         │
│    7. Stream loop: tokio::select! {                               │
│         token = stream.next() → Channel.send(Token)               │
│         _ = cancel.changed() → break (stop)                        │
│       }                                                            │
│    8. On Complete: Channel.send(Complete) + DbWorker save_message  │
│    9. On Error: Channel.send(Error)                               │
│                                                                    │
│  stop_generation command:                                          │
│    Drop CancellationToken → stream loop detects cancellation       │
│                                                                    │
│  regenerate command:                                               │
│    1. Delete last AI message from DB                               │
│    2. Re-invoke stream with remaining history                     │
│                                                                    │
│  edit_and_resend command:                                          │
│    1. Update message content in DB                                 │
│    2. DELETE messages WHERE sort_order > edit point                │
│    3. Re-invoke stream with truncated history                     │
└────────────────────────────────────────────────────────────────────┘
```

### Recommended Project Structure

```
src-tauri/src/
├── ai/
│   └── mod.rs              # NEW: AiService port from gtk_ref (rig-core 0.37 + Channel output)
├── commands/
│   ├── mod.rs              # Add: pub mod chat;
│   └── chat.rs             # NEW: start_chat, stop_generation, regenerate, edit_and_resend
├── domain/
│   ├── mod.rs              # Existing: add pub mod ai; (or reference from commands/chat)
│   ├── provider.rs         # Existing: ProviderConfig, ProviderType (used by AiService)
│   └── conversation.rs     # Existing: Message, MessageRole (may need metadata field)
├── db/
│   └── conversations.rs    # Existing: add delete_messages_after, update_message_content
└── secrets/
    └── mod.rs              # Existing: KeyringStore (used by start_chat to get API key)

src/lib/
├── stores/
│   └── app.svelte.ts       # Modified: add startChat, stopGeneration, regenerate, editResend
├── data/
│   └── api.ts              # Modified: add chat invoke wrappers
├── components/chat/
│   ├── MessageBubble.svelte # Modified: edit mode, error rendering, regenerate action
│   └── MessageInput.svelte # Minimal change: wire oncancel to app.stopGeneration
├── types/
│   └── index.ts            # Modified: add StreamChunk type, error_type to MessageMetadata
└── routes/chat/[id]/
    └── +page.svelte         # Modified: replace mock streaming with real invoke + Channel
```

### Pattern 1: Tauri v2 Channel Streaming (Core Phase 5 Pattern)

**What:** Use Tauri v2's `Channel<T>` to stream typed, ordered events from Rust to the frontend
**When to use:** Any streaming operation from backend to frontend
**Example:**

```rust
// Rust side — src-tauri/src/commands/chat.rs
use tauri::ipc::Channel;
use serde::Serialize;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
pub enum StreamChunk {
    Token { content: String },
    Complete { message_id: String, content: String },
    Error { error: String, error_type: String },
}

#[tauri::command]
pub async fn start_chat(
    db: State<'_, DbWorker>,
    cancel_state: State<'_, CancelState>,
    channel: Channel<StreamChunk>,
    conversation_id: String,
    content: String,
) -> Result<(), String> {
    // ... build prompt, spawn stream, send via channel.send()
}
```

```typescript
// Frontend side — src/lib/data/api.ts
import { invoke, Channel } from "@tauri-apps/api/core";

type StreamChunk =
  | { event: "token"; data: { content: string } }
  | { event: "complete"; data: { messageId: string; content: string } }
  | { event: "error"; data: { error: string; errorType: string } };

export async function startChat(
  conversationId: string,
  content: string,
  onEvent: Channel<StreamChunk>,
): Promise<void> {
  await invoke("start_chat", { conversationId, content, onEvent });
}
```

Source: [CITED: tauri-apps/tauri-docs — Channel pattern](https://v2.tauri.app/develop/calling-frontend/)

### Pattern 2: rig-core 0.37 Streaming (Updated from gtk_ref)

**What:** Use rig-core 0.37's `StreamingChat` trait with `MultiTurnStreamItem` for token-by-token responses
**When to use:** All AI generation calls
**Key difference from gtk_ref (0.36):** The `MultiTurnStreamItem` variants changed. gtk_ref used `StreamAssistantItem(StreamedAssistantContent::Text(t))`. rig-core 0.37 uses `MultiTurnStreamItem::Text(chunk)` and `MultiTurnStreamItem::FinalResponse(res)`.
**Example:**

```rust
// Source: Context7 rig-core docs (verified 2026-05-15)
use futures::StreamExt;
use rig::agent::MultiTurnStreamItem;
use rig::streaming::StreamingChat;

let mut stream = agent.stream_chat(prompt, &history).await;
while let Some(item) = stream.next().await {
    match item {
        Ok(MultiTurnStreamItem::Text(chunk)) => {
            // chunk is a string fragment — send as StreamChunk::Token
            channel.send(StreamChunk::Token { content: chunk.to_string() })?;
        }
        Ok(MultiTurnStreamItem::FinalResponse(res)) => {
            let full = res.response().to_string();
            channel.send(StreamChunk::Complete { message_id: msg_id.clone(), content: full })?;
            break;
        }
        Ok(_) => {} // other variants (tool calls, etc.)
        Err(e) => {
            channel.send(StreamChunk::Error { error: e.to_string(), error_type: classify_error(&e) })?;
            break;
        }
    }
}
```

Source: [CITED: Context7 /0xplaygrounds/rig — StreamingChat pattern]

### Pattern 3: Cancellation via CancellationToken

**What:** Use `tokio::sync::watch` or `tokio_util::sync::CancellationToken` for stop_generation
**When to use:** Any long-running async task that needs user-initiated cancellation
**Example:**

```rust
use std::sync::Mutex;
use tokio_util::sync::CancellationToken;

pub struct CancelState {
    pub token: Mutex<Option<CancellationToken>>,
}

#[tauri::command]
pub async fn start_chat(cancel_state: State<'_, CancelState>, /* ... */) {
    let cancel = CancellationToken::new();
    *cancel_state.token.lock().unwrap() = Some(cancel.clone());

    tokio::spawn(async move {
        tokio::select! {
            _ = stream_loop(&mut stream, &channel) => {},
            _ = cancel.cancelled() => {
                // Stream was cancelled — finalize partial response
                channel.send(StreamChunk::Complete { /* partial */ })?;
            }
        }
    });
}

#[tauri::command]
pub async fn stop_generation(cancel_state: State<'_, CancelState>) -> Result<(), String> {
    if let Some(token) = cancel_state.token.lock().unwrap().take() {
        token.cancel();
    }
    Ok(())
}
```

Source: [ASSUMED] — standard tokio cancellation pattern. tokio_util::sync::CancellationToken is widely used.

### Anti-Patterns to Avoid

- **Using Tauri Events (emit/listen) instead of Channel for streaming:** Events are global — two concurrent conversations could cross-talk. Channel is request-scoped per D-01.
- **Using rig-core 0.36.x API patterns in the port:** The `StreamAssistantItem` / `StreamedAssistantContent::Text` pattern from gtk_ref no longer exists in 0.37. Must use `MultiTurnStreamItem::Text(chunk)`.
- **Blocking the Tauri async command with rig-core streaming:** The `start_chat` command must spawn the stream loop and return immediately. The Channel continues sending events from the spawned task.
- **Saving AI message to DB on every token:** Only save when stream completes (D-04). Mid-stream messages exist only in AppState.
- **Calling KeyringStore::load_key() directly in an async Tauri command:** KeyringStore is synchronous and blocking. Must use `tokio::task::spawn_blocking()` (same as Phase 4 pattern).

## Don't Hand-Roll

| Problem                       | Don't Build                              | Use Instead                                              | Why                                                                                         |
| ----------------------------- | ---------------------------------------- | -------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| Provider-specific API clients | Custom HTTP clients per provider         | rig-core 0.37 provider clients                           | 15+ providers, streaming parsers, auth headers — enormous complexity to replicate           |
| Streaming cancellation        | Custom cancel flag + polling             | `tokio_util::sync::CancellationToken` + `tokio::select!` | Standard tokio pattern; clean cancellation without race conditions                          |
| Channel-based IPC             | Custom WebSocket or event bridge         | Tauri v2 `Channel<T>`                                    | Built-in typed channel with serde; no cross-talk; no setup overhead                         |
| Error classification          | Manual string matching on error messages | rig-core error types + custom classifier                 | rig returns structured errors; classify by type for UI display                              |
| Provider param filtering      | Send all params to all providers         | `provider_supports_param()` from gtk_ref                 | Anthropic/Gemini/Cohere reject frequency/presence_penalty; silently skip unsupported params |

**Key insight:** The entire AiService pattern from gtk_ref is a "don't hand-roll" — port it, adapt the stream output and cancellation, but don't redesign the provider dispatch.

## Common Pitfalls

### Pitfall 1: rig-core 0.37 API Breaking Change

**What goes wrong:** Copying gtk_ref's `MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t))` pattern — this variant doesn't exist in 0.37
**Why it happens:** gtk_ref was written for rig-core 0.36.x; 0.37 restructured the streaming API
**How to avoid:** Use `MultiTurnStreamItem::Text(chunk)` for token chunks and `MultiTurnStreamItem::FinalResponse(res)` for completion. The `Text` variant yields a string directly (not wrapped in `StreamedAssistantContent`).
**Warning signs:** Compile errors referencing `StreamAssistantItem` or `StreamedAssistantContent`

### Pitfall 2: Channel Not Received on Frontend

**What goes wrong:** The frontend creates a `Channel` but never receives events
**Why it happens:** The Channel must be passed as an argument named in a specific way (the Tauri command parameter must match the JS argument name), and the frontend must create the Channel BEFORE calling invoke
**How to avoid:** Follow the exact pattern from Tauri docs: `const onEvent = new Channel<StreamChunk>(); onEvent.onmessage = handler; await invoke('start_chat', { ..., onEvent });`. The parameter name in Rust must match the JS key — Tauri auto-converts camelCase to snake_case.
**Warning signs:** invoke() succeeds but onmessage never fires

### Pitfall 3: Cancellation Race Condition

**What goes wrong:** User clicks stop but the stream has already completed; the CancellationToken fires after the stream loop has exited
**Why it happens:** Stop and Complete can race — the stream may finish between the user clicking stop and the cancel signal propagating
**How to avoid:** In the `tokio::select!`, handle both branches gracefully. If Complete arrives first, cancel is a no-op. If cancel fires, finalize the partial response. Always clear the CancellationToken from managed state when the stream completes (in both branches).
**Warning signs:** AI message appears with full content AND a "Stopped" badge

### Pitfall 4: Stale CancellationToken Across Conversations

**What goes wrong:** User stops generation in conversation A, switches to B, and the cancel token from A is still held
**Why it happens:** CancellationToken is stored in global managed state; not cleaned up when switching conversations
**How to avoid:** Clear the CancellationToken (set to None) when: (1) stream completes, (2) user switches conversations, (3) a new start_chat is called. The stop_generation command should also clear it after cancelling.
**Warning signs:** New generation immediately stops; "Cancelled" appears on wrong conversation

### Pitfall 5: Missing `futures` Crate for StreamExt

**What goes wrong:** Compile error — `StreamExt` not found when processing rig-core's streaming response
**Why it happens:** `futures` is not in the current Cargo.toml dependencies; rig-core re-exports it but the consumer needs it too
**How to avoid:** Add `futures = "0.3"` to Cargo.toml dependencies. This is a required dependency for consuming rig's stream API.
**Warning signs:** `use futures::StreamExt;` fails to compile

### Pitfall 6: KeyringStore Blocking in start_chat

**What goes wrong:** The start_chat command hangs or blocks the Tauri async runtime while loading the API key from the keyring
**Why it happens:** KeyringStore::load_key() is synchronous and blocking; calling it directly in an async Tauri command blocks the runtime
**How to avoid:** Wrap in `tokio::task::spawn_blocking()` — exactly the same pattern used in Phase 4's provider commands.
**Warning signs:** UI freezes when sending a message; keyring operations take 100ms+ on some platforms

### Pitfall 7: Edit/Resend Sort Order Mismatch

**What goes wrong:** After edit/resend, the new AI message has a sort_order that conflicts with remaining messages
**Why it happens:** The delete operation removes messages after the edit point, but the new AI message sort_order must be `edited_message.sort_order + 1`
**How to avoid:** The backend must compute the next sort_order after truncation: `SELECT MAX(sort_order) FROM messages WHERE conversation_id = ?` after the DELETE, then insert the new AI message at max_sort_order + 1. Or compute it from the edit point: `edit_sort_order + 1`.
**Warning signs:** Messages appear out of order; new AI response shows above the edited message

### Pitfall 8: Error Type Classification for UI Display

**What goes wrong:** All errors show generic "The provider returned an error" instead of specific messages like "Context length exceeded"
**Why it happens:** rig-core errors are typed but not in a way that maps cleanly to the UI-SPEC error_type categories
**How to avoid:** Implement a `classify_error(err: &rig::Error) -> String` function that maps rig error variants to the UI-SPEC error types: `context_length_exceeded`, `authentication_failed`, `network_error`, `rate_limited`, `provider_error`, `no_provider_configured`, `no_api_key`. Fall back to `provider_error` with the error message for unknown types.
**Warning signs:** All errors show the same generic message

## Code Examples

### start_chat Command (Rust)

```rust
// Source: Tauri v2 Channel pattern + rig-core 0.37 streaming
use tauri::{ipc::Channel, State};
use serde::Serialize;
use tokio_util::sync::CancellationToken;

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
pub enum StreamChunk {
    Token { content: String },
    Complete { message_id: String, content: String, stop_reason: String },
    Error { error: String, error_type: String },
}

pub struct CancelState {
    pub token: std::sync::Mutex<Option<CancellationToken>>,
}

#[tauri::command]
pub async fn start_chat(
    db: State<'_, DbWorker>,
    cancel_state: State<'_, CancelState>,
    channel: Channel<StreamChunk>,
    conversation_id: String,
    content: String,
) -> Result<(), String> {
    // 1. Save user message to DB immediately (D-04)
    let user_msg = Message::with_order(
        conversation_id.clone(),
        MessageRole::User,
        content.clone(),
        next_sort_order, // compute from existing messages
    );
    db_send!(db, DbCommand::SaveMessage { message: user_msg.clone() });

    // 2. Load API key via spawn_blocking (D-03, Phase 4 pattern)
    let provider_id = /* from conversation */;
    let api_key = tokio::task::spawn_blocking(move || {
        crate::secrets::KeyringStore::load_key(&provider_id)
    }).await.map_err(|e| e.to_string())?.map_err(|e| e.to_string())?;

    // 3. Load provider config + character + messages from DbWorker
    // ... (load via DbWorker oneshot pattern)

    // 4. Build system prompt from character personality + scenario (D-05)
    let system_prompt = format!("{}\n\n{}", character.personality, character.scenario);

    // 5. Set up cancellation token (D-03)
    let cancel = CancellationToken::new();
    *cancel_state.token.lock().unwrap() = Some(cancel.clone());

    // 6. Spawn the stream loop
    tokio::spawn(async move {
        // Build rig agent and stream
        // Use tokio::select! for cancellation
        // Send StreamChunk events via channel.send()
        // On Complete: save AI message to DB
    });

    Ok(())
}
```

### Frontend Channel Listener (TypeScript)

```typescript
// Source: Tauri v2 Channel API + existing AppState patterns
import { invoke, Channel } from '@tauri-apps/api/core';

type StreamChunk =
  | { event: 'token'; data: { content: string } }
  | { event: 'complete'; data: { messageId: string; content: string; stopReason: string } }
  | { event: 'error'; data: { error: string; errorType: string } };

async startChat(content: string) {
  if (!this.active_conversation_id) return;

  // Create user message and append locally
  const userMsg = { /* ... */ };
  this.appendMessage(userMsg);

  // Save user message to DB immediately (D-04)
  await apiSaveMessage(userMsg);

  // Create AI placeholder
  const aiId = crypto.randomUUID();
  const aiMsg = { id: aiId, content: '', is_streaming: true, /* ... */ };
  this.appendMessage(aiMsg);
  this.is_generating = true;

  // Set up Channel listener
  const onEvent = new Channel<StreamChunk>();
  let accumulated = '';

  onEvent.onmessage = (message) => {
    if (message.event === 'token') {
      accumulated += message.data.content;
      this.updateStreamingMessage(aiId, accumulated);
    } else if (message.event === 'complete') {
      this.updateStreamingMessage(aiId, message.data.content, true);
      // Save AI message to DB
      apiSaveMessage({ id: aiId, content: message.data.content, /* ... */ });
      this.is_generating = false;
    } else if (message.event === 'error') {
      // Replace AI placeholder with error message
      this.messages = this.messages.map(m =>
        m.id === aiId ? {
          ...m,
          content: message.data.error,
          is_streaming: false,
          metadata: { stop_reason: 'error', error_type: message.data.errorType }
        } : m
      );
      this.is_generating = false;
    }
  };

  await invoke('start_chat', {
    conversationId: this.active_conversation_id,
    content,
    onEvent,
  });
}
```

### StreamChunk Serde Tag Pattern

```rust
// Source: Tauri v2 docs — Channel serialization pattern
// The #[serde(tag = "event", content = "data")] produces JSON like:
// { "event": "token", "data": { "content": "Hello" } }
// This matches the TypeScript discriminated union type naturally.

#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
pub enum StreamChunk {
    Token { content: String },
    Complete { message_id: String, content: String, stop_reason: String },
    Error { error: String, error_type: String },
}
```

### Error Classification Function

```rust
// Maps rig-core errors to UI-SPEC error_type categories
fn classify_error(err: &str) -> String {
    let err_lower = err.to_lowercase();
    if err_lower.contains("context") || err_lower.contains("max_tokens") || err_lower.contains("token limit") {
        "context_length_exceeded".to_string()
    } else if err_lower.contains("auth") || err_lower.contains("api key") || err_lower.contains("unauthorized") || err_lower.contains("401") {
        "authentication_failed".to_string()
    } else if err_lower.contains("rate") || err_lower.contains("429") || err_lower.contains("quota") {
        "rate_limited".to_string()
    } else if err_lower.contains("network") || err_lower.contains("connection") || err_lower.contains("timeout") || err_lower.contains("dns") {
        "network_error".to_string()
    } else {
        "provider_error".to_string()
    }
}
```

## State of the Art

| Old Approach                                             | Current Approach                                                         | When Changed   | Impact                                                        |
| -------------------------------------------------------- | ------------------------------------------------------------------------ | -------------- | ------------------------------------------------------------- |
| rig-core 0.36 `StreamAssistantItem`                      | rig-core 0.37 `MultiTurnStreamItem::Text(chunk)`                         | 0.37.0 release | Must adapt AiService stream handler — different variant names |
| rig-core 0.36 `FinalResponse(res)` with `res.response()` | rig-core 0.37 `MultiTurnStreamItem::FinalResponse` with `res.response()` | 0.37.0 release | Same API for final response, different wrapping               |
| Tauri v1 events                                          | Tauri v2 `Channel<T>`                                                    | Tauri 2.0      | Typed, scoped channels replace global event system            |
| gtk_ref GLib channel bridge                              | Tauri Channel IPC                                                        | Phase 5        | Different transport but same StreamChunk concept              |

**Deprecated/outdated:**

- `rig::streaming::StreamedAssistantContent::Text(t)` — replaced by `MultiTurnStreamItem::Text(chunk)` in 0.37 [VERIFIED: Context7 rig-core docs]
- Tauri v1 `window.emit()` for streaming — replaced by typed `Channel<T>` in v2 [CITED: tauri-apps/tauri-docs]

## Assumptions Log

| #   | Claim                                                                                                                                                               | Section                | Risk if Wrong                                                                                                                                        |
| --- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| A1  | `tokio_util::sync::CancellationToken` is the right cancellation primitive (D-03 says "CancellationSender")                                                          | Cancellation Pattern   | If Tauri has a built-in cancellation mechanism, we'd need to adapt; low risk — CancellationToken is standard                                         |
| A2  | rig-core 0.37 `MultiTurnStreamItem::Text(chunk)` yields `String` directly (not a struct with `.text` field)                                                         | rig-core 0.37 API      | If the Text variant yields a different type, the stream handler needs adjustment; Medium risk — verify at implementation                             |
| A3  | The `Channel<StreamChunk>` parameter name in Rust must match the JS argument key (after camelCase→snake_case conversion)                                            | Tauri Channel Pattern  | If naming mismatch, Channel events won't arrive; Low risk — well-documented pattern                                                                  |
| A4  | start_chat should also save the user message atomically (combining D-02 + D-04)                                                                                     | Discretionary Decision | RESOLVED — start_chat saves user message atomically using frontend-provided message_id and sort_order per Q3 recommendation                          |
| A5  | `tokio_util` crate needs to be added to Cargo.toml for CancellationToken                                                                                            | Cancellation Pattern   | If using only `tokio::sync::watch` instead, different API; Low risk — both are standard                                                              |
| A6  | The `message_id` returned in `StreamChunk::Complete` is generated on the Rust side (the frontend creates the placeholder but Rust generates the final persisted ID) | StreamChunk Design     | RESOLVED — Frontend pre-generates UUID and passes it to start_chat; IDs match between frontend placeholder and backend persistence per Q3 resolution |

## Open Questions (RESOLVED)

1. **tokio_util dependency vs. pure tokio cancellation** — RESOLVED: Use `tokio_util::sync::CancellationToken`. It's the standard pattern, cleaner API, and lightweight. Added `tokio-util = "0.7"` to Cargo.toml with `rt` feature. (Implemented in Plan 01 Cargo.toml changes and Plan 02 CancelState.)

2. **rig-core 0.37 `Text` chunk type** — RESOLVED: At implementation, check the rig-core 0.37 source/docs.rs for the exact type. Use `.to_string()` or `.text` field as appropriate. This is a LOW-risk unknown — the adaptation is trivial once the type is known. (Plan 01 action specifies `chunk.to_string()` as the default approach.)

3. **start_chat user message save atomicity** — RESOLVED: start_chat saves the user message atomically using frontend-provided message_id and sort_order. The frontend pre-generates the UUID and sort_order, passes them to start_chat, which persists the user message as its first step. This avoids a separate save_message invoke, eliminates the double-save race condition, and aligns with the single-command pattern (D-02). (Implemented in Plan 02 start_chat signature and Plan 03 startChat action.)

## Environment Availability

| Dependency            | Required By             | Available                 | Version      | Fallback                     |
| --------------------- | ----------------------- | ------------------------- | ------------ | ---------------------------- |
| Rust toolchain        | Backend compilation     | ✓                         | rustc 1.95.0 | —                            |
| Cargo                 | Package management      | ✓                         | cargo 1.95.0 | —                            |
| Node.js               | Frontend tooling        | ✓                         | 25.9.0       | —                            |
| Bun                   | Package management      | ✓                         | 1.3.13       | —                            |
| @tauri-apps/api ^2.11 | Channel class           | ✓                         | 2.11.0       | —                            |
| rig-core crate        | AI provider abstraction | ✗ (not in Cargo.toml yet) | 0.37.0       | —                            |
| futures crate         | StreamExt for rig       | ✗ (not in Cargo.toml yet) | 0.3.x        | —                            |
| tokio-util crate      | CancellationToken       | ✗ (not in Cargo.toml yet) | 0.7.x        | Use tokio::sync::watch<bool> |

**Missing dependencies with no fallback:**

- rig-core 0.37.0 — must be added to Cargo.toml; blocking for all AI operations
- futures 0.3.x — must be added; blocking for stream processing

**Missing dependencies with fallback:**

- tokio-util (CancellationToken) — fallback: `tokio::sync::watch<bool>` (less ergonomic but works)

## Validation Architecture

### Test Framework

| Property           | Value                                  |
| ------------------ | -------------------------------------- |
| Framework          | Rust built-in `#[test]` + `cargo test` |
| Config file        | None (inline in source files)          |
| Quick run command  | `cargo test -p revelry_lib --lib ai::` |
| Full suite command | `cargo test -p revelry_lib`            |

### Phase Requirements → Test Map

| Req ID  | Behavior                                             | Test Type   | Automated Command                                                               | File Exists? |
| ------- | ---------------------------------------------------- | ----------- | ------------------------------------------------------------------------------- | ------------ |
| CHAT-01 | Stream tokens from rig-core provider to Channel      | unit        | `cargo test -p revelry_lib --lib ai::test_stream_chat`                          | ❌ Wave 0    |
| CHAT-01 | Provider dispatch for all 15 types                   | unit        | `cargo test -p revelry_lib --lib ai::test_provider_dispatch`                    | ❌ Wave 0    |
| CHAT-02 | Cancellation stops stream and sends partial Complete | unit        | `cargo test -p revelry_lib --lib ai::test_cancel`                               | ❌ Wave 0    |
| CHAT-03 | Regenerate deletes last AI message and re-streams    | unit        | `cargo test -p revelry_lib --lib db::conversations::test_delete_message`        | ❌ Wave 0    |
| CHAT-04 | Edit/resend truncates and re-streams                 | unit        | `cargo test -p revelry_lib --lib db::conversations::test_delete_messages_after` | ❌ Wave 0    |
| CHAT-04 | Frontend edit mode toggles correctly                 | manual      | Visual inspection in browser                                                    | —            |
| CHAT-01 | End-to-end streaming through Channel                 | integration | `cargo test -p revelry_lib --lib commands::chat::test_start_chat_channel`       | ❌ Wave 0    |

### Sampling Rate

- **Per task commit:** `cargo test -p revelry_lib --lib`
- **Per wave merge:** `cargo test -p revelry_lib`
- **Phase gate:** Full suite green before `/gsd-verify-work`

### Wave 0 Gaps

- [ ] `src-tauri/src/ai/mod.rs` — new file with AiService, StreamChunk, GenerationParams, provider_supports_param, classify_error, tests
- [ ] `src-tauri/src/commands/chat.rs` — new file with start_chat, stop_generation, regenerate, edit_and_resend commands
- [ ] `src-tauri/src/db/conversations.rs` — add `delete_message`, `delete_messages_after_sort_order`, `update_message_content` functions
- [ ] `Cargo.toml` — add `rig-core = "0.37"`, `futures = "0.3"`, `tokio-util = { version = "0.7", features = ["rt"] }`

## Security Domain

### Applicable ASVS Categories

| ASVS Category         | Applies | Standard Control                                                             |
| --------------------- | ------- | ---------------------------------------------------------------------------- |
| V2 Authentication     | yes     | API key authentication via KeyringStore → rig-core provider clients          |
| V3 Session Management | no      | No user sessions — local-first single-user app                               |
| V4 Access Control     | no      | No multi-user access control                                                 |
| V5 Input Validation   | yes     | Content validation before sending to AI provider; conversation_id validation |
| V6 Cryptography       | yes     | API keys stored in OS keychain via KeyringStore — never in DB or logs        |

### Known Threat Patterns for AI Chat Streaming

| Pattern                                                 | STRIDE                 | Standard Mitigation                                                                                            |
| ------------------------------------------------------- | ---------------------- | -------------------------------------------------------------------------------------------------------------- |
| API key leakage in error messages                       | Information Disclosure | Redact API keys from error messages before sending to frontend; use `classify_error()` that strips credentials |
| Prompt injection via user messages                      | Tampering              | Not mitigated in Phase 5 — deferred to Phase 6 (prompt engineering). User content is sent verbatim             |
| Context overflow attack (excessively long conversation) | Denial of Service      | Phase 5 sends full history (D-06); error returned inline. Phase 6 adds budget enforcement                      |
| Concurrent stream abuse (multiple start_chat calls)     | Denial of Service      | Only one active stream per app (enforced by CancelState — new start_chat cancels any existing stream)          |

## Sources

### Primary (HIGH confidence)

- Context7 `/0xplaygrounds/rig` — rig-core 0.37 streaming API (MultiTurnStreamItem, StreamingChat, Agent builder)
- Context7 `/tauri-apps/tauri-docs` — Tauri v2 Channel API (typed Channel, serde tagged enum pattern, frontend Channel class)
- `gtk_ref/src/service/ai.rs` — Complete AiService implementation with 15 provider dispatch types (archival reference for porting)
- `gtk_ref/src/domain/provider.rs` — ProviderConfig, ProviderType with requires_api_key(), default_base_url(), default_model_name() (archival reference)
- `src-tauri/src/domain/provider.rs` — Active ProviderConfig with 15 serde-renamed variants (already ported in Phase 4)
- `src-tauri/src/secrets/mod.rs` — KeyringStore implementation (already ported in Phase 4)

### Secondary (MEDIUM confidence)

- `gtk_ref/src/domain/conversation.rs` — Message, MessageRole domain models (archival reference — active version in src-tauri already ported)
- `05-UI-SPEC.md` — Complete design contract for Phase 5 UI changes (discretionary decisions resolved)
- `src/lib/stores/app.svelte.ts` — Current AppState with mock-friendly streaming patterns
- `src/lib/components/chat/MessageBubble.svelte` — Current component with stubbed edit/regenerate actions

### Tertiary (LOW confidence)

- tokio_util::sync::CancellationToken API — [ASSUMED] based on training knowledge; verify at implementation
- rig-core 0.37 `MultiTurnStreamItem::Text` exact type — partially verified via Context7 (printed with `{chunk}`); exact type needs docs.rs check

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — all versions verified via cargo/npm, Tauri Channel pattern verified via Context7
- Architecture: HIGH — Tauri Channel pattern is well-documented; AiService port is direct from gtk_ref
- Pitfalls: HIGH — rig-core API change is critical and verified; other pitfalls drawn from established patterns

**Research date:** 2026-05-15
**Valid until:** 2026-06-14 (30 days — stable Rust crate versions, Tauri v2 is mature)
