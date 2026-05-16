# Phase 5: Live Conversations - Pattern Map

**Mapped:** 2026-05-15
**Files analyzed:** 12 new/modified files
**Analogs found:** 12 / 12

## File Classification

| New/Modified File | Role | Data Flow | Closest Analog | Match Quality |
|-------------------|------|-----------|----------------|---------------|
| `src-tauri/src/ai/mod.rs` (NEW) | service | streaming | `gtk_ref/src/service/ai.rs` | exact (porting target) |
| `src-tauri/src/commands/chat.rs` (NEW) | controller | request-response + streaming | `src-tauri/src/commands/providers.rs` | role-match |
| `src-tauri/src/commands/mod.rs` (MODIFY) | config | — | existing file | self |
| `src-tauri/src/lib.rs` (MODIFY) | config | — | existing file | self |
| `src-tauri/Cargo.toml` (MODIFY) | config | — | existing file | self |
| `src-tauri/src/domain/mod.rs` (MODIFY) | config | — | existing file | self |
| `src-tauri/src/db/conversations.rs` (MODIFY) | model | CRUD | existing file | self |
| `src-tauri/src/db/mod.rs` (MODIFY) | model | CRUD | existing file | self |
| `src/routes/chat/[id]/+page.svelte` (MODIFY) | component | streaming | existing file | self |
| `src/lib/stores/app.svelte.ts` (MODIFY) | service | event-driven | existing file | self |
| `src/lib/data/api.ts` (MODIFY) | service | request-response | existing file | self |
| `src/lib/components/chat/MessageBubble.svelte` (MODIFY) | component | event-driven | existing file | self |
| `src/lib/components/chat/MessageInput.svelte` (MODIFY) | component | event-driven | existing file | self |
| `src/lib/types/index.ts` (MODIFY) | model | — | existing file | self |

## Pattern Assignments

### `src-tauri/src/ai/mod.rs` (NEW — service, streaming)

**Analog:** `gtk_ref/src/service/ai.rs` (direct porting target)

**Critical adaptation:** Replace rig-core 0.36 `MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t))` with rig-core 0.37 `MultiTurnStreamItem::Text(chunk)`. Replace `tokio::sync::mpsc` channel output with `tauri::ipc::Channel<StreamChunk>`. Add `CancellationToken` for stop_generation support.

**Imports pattern** (from `gtk_ref/src/service/ai.rs` lines 1-7, adapted for 0.37):
```rust
use crate::domain::conversation::{Message, MessageRole};
use crate::domain::provider::{ProviderConfig, ProviderType};

use futures::StreamExt;
use rig::streaming::StreamingChat;
use serde::Serialize;
use tauri::ipc::Channel;
```

**StreamChunk enum** (new — replaces gtk_ref's `StreamChunk` lines 72-80, adapted for Tauri Channel serde):
```rust
/// A chunk of data from a streaming AI response.
/// Serialized via Tauri Channel to the frontend.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase", rename_all_fields = "camelCase", tag = "event", content = "data")]
pub enum StreamChunk {
    Token { content: String },
    Complete { message_id: String, content: String, stop_reason: String },
    Error { error: String, error_type: String },
}
```

**GenerationParams** (from `gtk_ref/src/service/ai.rs` lines 11-52 — port directly):
```rust
#[derive(Clone, Debug, Default)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub stop_sequences: Option<Vec<String>>,
}
```

**provider_supports_param** (from `gtk_ref/src/service/ai.rs` lines 56-69 — port directly):
```rust
fn provider_supports_param(provider_type: &ProviderType, param_name: &str) -> bool {
    match param_name {
        "frequency_penalty" | "presence_penalty" => {
            !matches!(
                provider_type,
                ProviderType::Anthropic | ProviderType::Gemini | ProviderType::Cohere
            )
        }
        "top_p" | "stop" => true,
        _ => false,
    }
}
```

**Provider dispatch** (from `gtk_ref/src/service/ai.rs` lines 154-422 — port with 0.37 API changes):
The 15-arm match block dispatching `ProviderType` to rig-core clients ports directly. Each arm follows the same pattern: build client → build agent with preamble/temperature/max_tokens/additional_params → `stream_with_agent`. **Key change:** use `ProviderType` variants from `src-tauri/src/domain/provider.rs` (which has `CustomOpenAICompatible` instead of `Custom`).

**stream_with_agent** (from `gtk_ref/src/service/ai.rs` lines 476-515 — MUST adapt for 0.37):
```rust
// OLD (0.36) — DO NOT COPY:
// Ok(MultiTurnStreamItem::StreamAssistantItem(StreamedAssistantContent::Text(t))) => {
//     if tx.send(StreamChunk::Token(t.text)).await.is_err() { break; }
// }

// NEW (0.37) — correct pattern:
Ok(MultiTurnStreamItem::Text(chunk)) => {
    if tx.send(StreamChunk::Token { content: chunk.to_string() }).await.is_err() {
        break;
    }
}
Ok(MultiTurnStreamItem::FinalResponse(res)) => {
    let _ = tx.send(StreamChunk::Complete {
        message_id: msg_id.clone(),
        content: res.response().to_string(),
        stop_reason: "end".to_string(),
    }).await;
    break;
}
```

**build_additional_params** (from `gtk_ref/src/service/ai.rs` lines 426-457 — port directly, identical logic).

**split_last_prompt** (from `gtk_ref/src/service/ai.rs` lines 143-151 — port directly, identical logic).

**classify_error** (NEW — no analog in gtk_ref):
```rust
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

---

### `src-tauri/src/commands/chat.rs` (NEW — controller, request-response + streaming)

**Analog:** `src-tauri/src/commands/providers.rs`

**Imports pattern** (from `src-tauri/src/commands/providers.rs` lines 1-5, extended for chat):
```rust
use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db::{DbCommand, DbWorker};
use crate::domain::provider::ProviderConfig;
use crate::ai::StreamChunk;
```

**Tauri command pattern** (from `src-tauri/src/commands/providers.rs` lines 17-44):
```rust
#[tauri::command]
pub async fn get_providers(db: State<'_, DbWorker>) -> Result<Vec<ProviderWithKeyStatus>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadProviders { reply: reply_tx })
        .map_err(|e| e.to_string())?;
    let providers = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())?;
    // ... process results
    Ok(result)
}
```

**DbWorker oneshot pattern** (from `src-tauri/src/commands/conversations.rs` lines 65-80):
```rust
#[tauri::command]
pub async fn get_messages(
    db: State<'_, DbWorker>,
    conversation_id: String,
) -> Result<Vec<Message>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadMessages {
        conversation_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {}", e))?
        .map_err(|e| e.to_string())
}
```

**KeyringStore spawn_blocking pattern** (from `src-tauri/src/commands/providers.rs` lines 31-34):
```rust
let has_key = tokio::task::spawn_blocking(move || {
    crate::secrets::KeyringStore::load_key(&pid).is_ok()
})
.await
.unwrap_or(false);
```

**start_chat command skeleton** (new — combines patterns above):
```rust
#[tauri::command]
pub async fn start_chat(
    db: State<'_, DbWorker>,
    cancel_state: State<'_, CancelState>,
    channel: Channel<StreamChunk>,
    conversation_id: String,
    content: String,
) -> Result<(), String> {
    // 1. Save user message via DbWorker (oneshot pattern)
    // 2. Load API key via spawn_blocking (providers.rs pattern)
    // 3. Load provider config + character + messages via DbWorker
    // 4. Build system prompt from character personality + scenario
    // 5. Set up CancellationToken in CancelState
    // 6. tokio::spawn the stream loop
    // 7. Return Ok(()) — channel continues sending events from spawned task
    Ok(())
}
```

**CancelState managed state** (new — follows DbWorker managed state pattern):
```rust
pub struct CancelState {
    pub token: std::sync::Mutex<Option<tokio_util::sync::CancellationToken>>,
}
```

---

### `src-tauri/src/commands/mod.rs` (MODIFY — config)

**Analog:** existing file at `src-tauri/src/commands/mod.rs`

**Current pattern** (lines 1-8):
```rust
pub mod avatars;
pub mod characters;
pub mod conversations;
pub mod keywords;
pub mod lorebooks;
pub mod personas;
pub mod presets;
pub mod providers;
```

**Change:** Add `pub mod chat;`

---

### `src-tauri/src/lib.rs` (MODIFY — config)

**Analog:** existing file at `src-tauri/src/lib.rs`

**Current command registration pattern** (lines 35-73):
```rust
.invoke_handler(tauri::generate_handler![
    commands::characters::get_characters,
    commands::characters::save_character,
    // ... all existing commands
    commands::providers::update_conversation_provider,
])
```

**Changes needed:**
1. Add `mod ai;` at top (alongside `mod commands;`, `mod db;`, `mod domain;`, `mod secrets;`)
2. Add `CancelState` to managed state: `.manage(CancelState { token: std::sync::Mutex::new(None) })`
3. Add new commands to `invoke_handler`:
   - `commands::chat::start_chat`
   - `commands::chat::stop_generation`
   - `commands::chat::regenerate`
   - `commands::chat::edit_and_resend`

---

### `src-tauri/Cargo.toml` (MODIFY — config)

**Analog:** existing file at `src-tauri/Cargo.toml`

**Current dependency pattern** (lines 15-34):
```toml
[dependencies]
tauri = { version = "2", features = ["protocol-asset"] }
# ... existing deps
tokio = { version = "1", features = ["sync", "rt-multi-thread"] }
```

**Changes needed:**
1. Add `rig-core = "0.37"` (NOT 0.36 — critical per RESEARCH.md Pitfall 1)
2. Add `futures = "0.3"` (required for `StreamExt`)
3. Add `tokio-util = { version = "0.7", features = ["rt"] }` (for `CancellationToken`)
4. Update tokio features: add `"macros"` to existing features list

---

### `src-tauri/src/domain/mod.rs` (MODIFY — config)

**Analog:** existing file at `src-tauri/src/domain/mod.rs`

**Current pattern** (lines 1-15):
```rust
pub mod character;
pub mod conversation;
pub mod generation;
pub mod lorebook;
pub mod persona;
pub mod provider;

pub use character::CharacterCardV2;
pub use conversation::{Conversation, Message};
// ... other re-exports
```

**Change:** No module addition needed here — the `ai` module lives at `src-tauri/src/ai/mod.rs` (separate from domain). If we want to re-export `StreamChunk`, add:
```rust
// No change needed — ai module is separate from domain
```

However, if the `ai` module is registered at `src-tauri/src/ai/mod.rs`, it's a top-level module (like `commands`, `db`, `secrets`) — not a domain submodule. No modification needed to `domain/mod.rs`.

---

### `src-tauri/src/db/mod.rs` (MODIFY — model, CRUD)

**Analog:** existing file at `src-tauri/src/db/mod.rs`

**Current DbCommand pattern** (lines 25-163):
```rust
pub enum DbCommand {
    // ... existing variants with oneshot reply channels
    SaveMessage {
        message: Message,
        reply: oneshot::Sender<anyhow::Result<()>>,
    },
    // ...
}
```

**Changes needed — add new DbCommand variants:**
```rust
// New variants for Phase 5 chat operations
DeleteMessage {
    id: String,
    reply: oneshot::Sender<anyhow::Result<()>>,
},
DeleteMessagesAfterSortOrder {
    conversation_id: String,
    sort_order: i32,
    reply: oneshot::Sender<anyhow::Result<()>>,
},
UpdateMessageContent {
    id: String,
    content: String,
    reply: oneshot::Sender<anyhow::Result<()>>,
},
```

**DbWorker handle_command match pattern** (lines 234-443): Add new match arms following the existing pattern of calling sub-module functions and sending the result via `reply.send()`.

---

### `src-tauri/src/db/conversations.rs` (MODIFY — model, CRUD)

**Analog:** existing file (already contains `save_message`, `load_messages`, etc.)

**New functions to add:**
- `delete_message(conn: &Connection, id: &str) -> Result<()>`
- `delete_messages_after_sort_order(conn: &Connection, conversation_id: &str, sort_order: i32) -> Result<()>` — `DELETE FROM messages WHERE conversation_id = ? AND sort_order > ?`
- `update_message_content(conn: &Connection, id: &str, content: &str) -> Result<()>` — `UPDATE messages SET content = ? WHERE id = ?`

---

### `src/routes/chat/[id]/+page.svelte` (MODIFY — component, streaming)

**Analog:** existing file

**Current mock streaming pattern** (lines 43-89 — TO BE REPLACED):
```typescript
function handleSend(text: string) {
    if (!app.active_conversation_id) return;
    const now = new Date().toISOString();
    const nextSortOrder = (app.messages.at(-1)?.sort_order ?? -1) + 1;

    const userMsg: Message = { /* ... */ };
    app.appendMessage(userMsg);

    const aiId = `msg_${Date.now()}_a`;
    const aiMsg: Message = { /* ... is_streaming: true ... */ };
    app.appendMessage(aiMsg);
    app.is_generating = true;

    // Simulated token stream — REPLACE THIS
    const sample = '...';
    const chunks = sample.match(/.{1,4}/g) ?? [sample];
    let acc = '';
    let i = 0;
    const streamTick = () => { /* setTimeout loop */ };
    setTimeout(streamTick, 200);
}
```

**New pattern** — delegate to AppState:
```typescript
function handleSend(text: string) {
    if (!app.active_conversation_id) return;
    app.startChat(text);  // All streaming logic moves to AppState
}

function handleCancel() {
    app.stopGeneration();  // Replaces direct updateStreamingMessage
}

function handleRegenerate() {
    app.regenerate();
}

function handleEditResend(messageId: string, newContent: string) {
    app.editResend(messageId, newContent);
}
```

**Preserved patterns:**
- `$effect` for conversation hydration (lines 19-26)
- `$effect` for auto-scroll on message changes (lines 29-36)
- `MessageBubble` rendering loop (lines 115-121)
- `MessageInput` with `onsend`/`oncancel` (line 126)

---

### `src/lib/stores/app.svelte.ts` (MODIFY — service, event-driven)

**Analog:** existing file

**Current action pattern** (e.g., `switchPersona` lines 228-251):
```typescript
async switchPersona(persona_id: string) {
    if (!this.active_conversation_id) return;
    // Optimistically update local state
    this.conversations = this.conversations.map((c) =>
        c.id === this.active_conversation_id
            ? { ...c, creator_persona_id: persona_id }
            : c,
    );
    // Persist to backend
    try {
        await apiUpdateConversationPersona(this.active_conversation_id, persona_id);
    } catch (err) {
        console.error("Failed to persist persona switch:", err);
        // Revert on failure
        this.conversations = this.conversations.map((c) =>
            c.id === this.active_conversation_id
                ? { ...c, creator_persona_id: undefined }
                : c,
        );
    }
}
```

**New actions to add — follow same optimistic-update + backend-persist pattern:**

```typescript
// startChat — the core streaming action
async startChat(content: string) {
    if (!this.active_conversation_id) return;
    // 1. Create user message, append, save to DB immediately (D-04)
    // 2. Create AI placeholder with is_streaming: true
    // 3. Set is_generating = true
    // 4. Create Channel, set onmessage handler for Token/Complete/Error
    // 5. invoke('start_chat', { conversationId, content, onEvent })
    // On Token: updateStreamingMessage(aiId, accumulated)
    // On Complete: updateStreamingMessage(aiId, content, done=true), saveMessage to DB
    // On Error: replace AI placeholder with error message, is_generating = false
}

async stopGeneration() {
    try {
        await apiStopGeneration();
    } catch (err) {
        console.error("Failed to stop generation:", err);
    }
}

async regenerate() {
    // Delete last AI message from UI and DB, then re-invoke start_chat
}

async editResend(messageId: string, newContent: string) {
    // Update message content, truncate subsequent, re-invoke start_chat
}
```

**Existing streaming support** (lines 213-222 — keep as-is):
```typescript
appendMessage(message: Message) {
    this.messages = [...this.messages, message];
}

updateStreamingMessage(id: string, content: string, done = false) {
    this.messages = this.messages.map((m) =>
        m.id === id ? { ...m, content, is_streaming: !done } : m,
    );
    if (done) this.is_generating = false;
}
```

---

### `src/lib/data/api.ts` (MODIFY — service, request-response)

**Analog:** existing file

**Current invoke pattern** (lines 24-26):
```typescript
export async function getCharacters(): Promise<CharacterCard[]> {
    return invoke<CharacterCard[]>("get_characters");
}
```

**Current parameterized invoke pattern** (lines 86-88):
```typescript
export async function getMessages(conversationId: string): Promise<Message[]> {
    return invoke<Message[]>("get_messages", { conversationId });
}
```

**New wrappers to add:**
```typescript
import { Channel } from "@tauri-apps/api/core";

// Channel-based streaming invoke (new pattern for Phase 5)
export async function startChat(
    conversationId: string,
    content: string,
    onEvent: Channel<StreamChunk>,
): Promise<void> {
    return invoke<void>("start_chat", { conversationId, content, onEvent });
}

export async function stopGeneration(): Promise<void> {
    return invoke<void>("stop_generation");
}

export async function regenerate(
    conversationId: string,
): Promise<void> {
    return invoke<void>("regenerate", { conversationId });
}

export async function editAndResend(
    messageId: string,
    newContent: string,
): Promise<void> {
    return invoke<void>("edit_and_resend", { messageId, newContent });
}
```

**Note:** The `Channel<T>` import from `@tauri-apps/api/core` is new for Phase 5. The `StreamChunk` type definition goes in `types/index.ts`.

---

### `src/lib/components/chat/MessageBubble.svelte` (MODIFY — component, event-driven)

**Analog:** existing file

**Current props pattern** (lines 7-12):
```svelte
interface Props {
    message: Message;
    character: CharacterCard | undefined;
    persona: UserPersona | undefined;
}
let { message, character, persona }: Props = $props();
```

**Current hover actions pattern** (lines 83-120):
```svelte
<div class={cn(
    'mt-0.5 flex gap-0.5 px-1 opacity-0 transition-opacity group-hover:opacity-100',
    isUser ? 'flex-row-reverse' : 'flex-row'
)}>
    <button type="button"
        class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
        aria-label="Copy">
        <Copy class="size-3" />
    </button>
    <!-- ... Edit, Regenerate, Delete buttons ... -->
</div>
```

**Changes needed:**
1. **Edit mode**: Add `$state` for `isEditing` boolean. When true, replace bubble `<p>` content with `<textarea>`. Show "Save & resend" / "Discard changes" buttons instead of hover actions.
2. **Conditional hover actions**: Edit button only on user messages. Regenerate only on last AI message. Error messages show Copy only.
3. **Error rendering**: When `message.metadata?.stop_reason === 'error'`, render with destructive-tinted styling:
   - `bg-(--color-destructive)/10` background
   - `border border-(--color-destructive)/20`
   - `AlertTriangle` icon + error text + Retry/Switch provider links
4. **Stop reason badge**: When `stop_reason` is "cancelled" or "length", render badge below bubble text.
5. **No avatar for error messages**: Error messages skip the Avatar column.

---

### `src/lib/components/chat/MessageInput.svelte` (MODIFY — component, event-driven)

**Analog:** existing file

**Current stop button pattern** (lines 96-104):
```svelte
{#if app.is_generating}
    <button type="button"
        onclick={() => oncancel?.()}
        class="inline-flex size-8 shrink-0 items-center justify-center rounded-(--radius-sm) bg-(--color-destructive) text-white hover:bg-(--color-destructive)/90"
        aria-label="Stop generation">
        <Square class="size-3.5" fill="currentColor" />
    </button>
{:else}
    <!-- Send button -->
{/if}
```

**Change:** Minimal — the `oncancel` prop already calls `handleCancel` in the page. The page's `handleCancel` now calls `app.stopGeneration()` instead of directly updating the streaming message. Also disable textarea during generation per UI-SPEC.

Add `disabled={app.is_generating}` to the textarea element (line 86-94).

---

### `src/lib/types/index.ts` (MODIFY — model)

**Analog:** existing file

**Current MessageMetadata pattern** (lines 113-119):
```typescript
export interface MessageMetadata {
    persona_id?: string;
    provider?: string;
    model?: string;
    stop_reason?: "end" | "cancelled" | "length" | "error";
    [key: string]: unknown;
}
```

**Changes needed:**
1. Add `error_type?: string` to `MessageMetadata` (for `context_length_exceeded`, `authentication_failed`, etc.)
2. Add `StreamChunk` discriminated union type:
```typescript
export type StreamChunk =
    | { event: "token"; data: { content: string } }
    | { event: "complete"; data: { messageId: string; content: string; stopReason: string } }
    | { event: "error"; data: { error: string; errorType: string } };
```

---

## Shared Patterns

### DbWorker Oneshot Request-Response
**Source:** `src-tauri/src/commands/conversations.rs` lines 7-15
**Apply to:** All new Tauri commands (`start_chat`, `regenerate`, `edit_and_resend`) that need to load/save data via DbWorker
```rust
let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
db.send(DbCommand::SomeCommand { /* params */, reply: reply_tx })
    .map_err(|e| e.to_string())?;
reply_rx
    .await
    .map_err(|e| format!("DB worker dropped: {}", e))?
    .map_err(|e| e.to_string())
```

### KeyringStore spawn_blocking
**Source:** `src-tauri/src/commands/providers.rs` lines 31-34
**Apply to:** `start_chat` command when loading API key
```rust
tokio::task::spawn_blocking(move || {
    crate::secrets::KeyringStore::load_key(&provider_id)
})
.await
.map_err(|e| format!("Keyring task failed: {}", e))?
.map_err(|e| format!("Failed to load API key: {}", e))
```

### Optimistic State Update + Backend Persist + Revert on Failure
**Source:** `src/lib/stores/app.svelte.ts` lines 228-251 (`switchPersona`)
**Apply to:** All new AppState actions (`editResend`, `regenerate`, `updateConversationProvider`)
```typescript
// 1. Optimistically update local state
this.messages = /* updated */;
// 2. Persist to backend
try {
    await apiSomeAction(/* ... */);
} catch (err) {
    console.error("Failed to ...:", err);
    // 3. Revert on failure
    this.messages = /* revert */;
}
```

### Tauri Command Registration Triangle
**Source:** `src-tauri/src/lib.rs` lines 35-73
**Apply to:** All new Tauri commands — must appear in three places:
1. `src-tauri/src/commands/mod.rs` — `pub mod chat;`
2. `src-tauri/src/commands/chat.rs` — `#[tauri::command] pub async fn start_chat(...)`
3. `src-tauri/src/lib.rs` — `commands::chat::start_chat` in `invoke_handler`

### TypeScript Type Alignment (Rust serde ↔ TS interface)
**Source:** `src/lib/types/index.ts` + `src-tauri/src/domain/conversation.rs`
**Apply to:** `StreamChunk` enum — Rust `#[serde(rename_all = "camelCase", tag = "event", content = "data")]` must produce JSON matching the TS discriminated union:
```typescript
// Rust produces: { "event": "token", "data": { "content": "Hello" } }
// TS matches: { event: "token"; data: { content: string } }
```

### Svelte 5 Component Props Pattern
**Source:** `src/lib/components/chat/MessageBubble.svelte` lines 7-13
**Apply to:** Any new component or modified component receiving new props
```svelte
interface Props {
    message: Message;
    character: CharacterCard | undefined;
    persona: UserPersona | undefined;
    isLastAiMessage?: boolean;  // New prop for regenerate visibility
}
let { message, character, persona, isLastAiMessage = false }: Props = $props();
```

### Hover Action Button Pattern
**Source:** `src/lib/components/chat/MessageBubble.svelte` lines 90-119
**Apply to:** New Edit, Regenerate, and Retry hover actions — same class structure
```svelte
<button type="button"
    class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
    aria-label="Edit"
    onclick={() => /* enter edit mode */}>
    <Pencil class="size-3" />
</button>
```

## No Analog Found

| File | Role | Data Flow | Reason |
|------|------|-----------|--------|
| `src-tauri/src/ai/mod.rs` | service | streaming | Partial analog in `gtk_ref/src/service/ai.rs` — but Tauri Channel + CancellationToken + rig-core 0.37 API are all new patterns not in codebase |
| `src-tauri/src/commands/chat.rs` | controller | streaming | No existing Tauri command uses `Channel<StreamChunk>` — this is a new pattern for the codebase |

**Planner guidance:** For the two files above, combine the closest analogs documented in Pattern Assignments with the concrete code examples from `05-RESEARCH.md` (Pattern 1: Tauri v2 Channel Streaming, Pattern 2: rig-core 0.37 Streaming, Pattern 3: Cancellation via CancellationToken).

## Metadata

**Analog search scope:** `src-tauri/src/`, `src/lib/`, `gtk_ref/src/service/`, `gtk_ref/src/domain/`
**Files scanned:** 20
**Pattern extraction date:** 2026-05-15
