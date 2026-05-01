# Phase 3: AI Chat & Streaming - Research

**Domain:** Streaming AI conversations with branching, message CRUD, and markdown rendering
**Researched:** 2026-04-30
**Confidence:** HIGH

## Standard Stack

All technologies already in Cargo.toml from Phase 1 scaffold:
- rig-core 0.36.0 — StreamingPrompt + StreamingChat traits, agent builder with preamble
- tokio 1.52.1 — Async runtime for rig streaming
- futures 0.3.32 — StreamExt for chunk-by-chunk processing
- pulldown-cmark 0.13.3 — Markdown → event parsing
- pango 0.22.6 — Pango markup for formatted rendering
- gtk4 0.11.3 / libadwaita 0.9.1 / relm4 0.11.0 — UI framework

## Key API Patterns

### rig-core Streaming (verified via Context7 + docs.rs)

**Agent construction per provider:**
```rust
// OpenAI
let client = openai::Client::new(&api_key);
let agent = client.agent("gpt-4o")
    .preamble("System prompt here")
    .temperature(0.8)
    .build();

// Anthropic
let client = anthropic::Client::new(&api_key);
let agent = client.agent(anthropic::CLAUDE_3_5_SONNET)
    .preamble("System prompt here")
    .build();

// Ollama (no API key needed)
let client = ollama::Client::new();
let agent = client.agent("llama3:8b")
    .preamble("System prompt here")
    .build();
```

**Streaming with chat history:**
```rust
use rig::streaming::StreamedAssistantContent;
use rig::agent::prompt_request::streaming::MultiTurnStreamItem;
use futures::StreamExt;

let mut stream = agent.stream_chat(user_message, &chat_history).await?;
while let Some(chunk) = stream.next().await {
    match chunk? {
        MultiTurnStreamItem::StreamAssistantItem(
            StreamedAssistantContent::Text(text)
        ) => { /* token chunk: text.text */ }
        MultiTurnStreamItem::FinalResponse(resp) => {
            /* resp.response() for full text, resp.history() for updated history */
        }
        _ => {}
    }
}
```

**Key findings:**
- `stream_chat()` takes prompt + history iterator; returns `StreamingPromptRequest`
- `stream_prompt()` for single-turn (no history)
- `MultiTurnStreamItem::FinalResponse` contains `response()` and `history()` for budget tracking
- rig constructs `Message` enum with System/User/Assistant variants per provider
- `preamble()` on agent builder = system prompt (not a chat message)
- Each provider client is constructed with `Client::new(api_key)` or `Client::from_env()`

### Relm4 Command Pattern for Streaming

**Per ARCHITECTURE.md and verified Relm4 0.11 API:**
- `sender.command(|out, shutdown| { ... })` — provides `Sender<CommandOutput>` for multi-message streaming
- `shutdown.register(async { ... }).drop_on_shutdown().boxed()` — cancellation via component drop
- `update_cmd()` handles `CommandOutput` messages on GLib main loop
- This replaces the manual `glib::MainContext::channel()` approach

### pulldown-cmark → Pango Pipeline

```rust
use pulldown_cmark::{Parser, Options, Event, Tag, TagEnd};

fn markdown_to_pango(input: &str) -> String {
    let mut pango = String::new();
    let parser = Parser::new_ext(input, Options::all());
    for event in parser {
        match event {
            Event::Start(Tag::Strong) => pango.push_str("<b>"),
            Event::End(TagEnd::Strong) => pango.push_str("</b>"),
            Event::Start(Tag::Emphasis) => pango.push_str("<i>"),
            Event::End(TagEnd::Emphasis) => pango.push_str("</i>"),
            Event::Start(Tag::Heading { level, .. }) => {
                let size = match level {
                    1 => "xx-large", 2 => "x-large", 3 => "large",
                    _ => "medium",
                };
                pango.push_str(&format!("<span size=\"{}\"><b>", size));
            }
            Event::End(TagEnd::Heading(_)) => pango.push_str("</b></span>\n"),
            Event::Start(Tag::List(_)) => { /* track list depth */ }
            Event::Start(Tag::Item) => pango.push_str("• "),
            Event::End(TagEnd::Item) => pango.push('\n'),
            Event::Start(Tag::CodeBlock(_)) => pango.push_str("<tt>"),
            Event::End(TagEnd::CodeBlock) => pango.push_str("</tt>\n"),
            Event::Code(code) => pango.push_str(&format!("<tt>{}</tt>", escape_pango(&code))),
            Event::Text(text) => pango.push_str(&escape_pango(&text)),
            Event::SoftBreak | Event::HardBreak => pango.push('\n'),
            _ => {}
        }
    }
    pango
}

fn escape_pango(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}
```

**Critical: Pango markup must be escaped** — raw `<` `>` `&` in AI output will break Pango parsing. Must escape text content before wrapping in Pango tags.

### Conversation Branching Model (per CONTEXT.md D-01)

**3-table schema:**
- `conversations`: id, character_id, provider_id, creator_persona_id, created_at, updated_at
- `messages`: id, conversation_id, branch_id, role, content, timestamp, sort_order
- `message_branches`: id, conversation_id, parent_message_id, label, created_at

**Branch walk:** Messages with same branch_id form a branch. branch_id NULL = root branch.
**Branch navigation:** Given a message, siblings = all messages with same parent_message_id. Use colored left border indicator + popover to switch.

## Architecture Patterns

### Pattern: Streaming Bridge (sender.command + rig stream)

Use `sender.command()` with `ShutdownReceiver` for cancellation. Each token chunk sends a `CommandOutput` to `update_cmd()`. The streaming loop runs on tokio; `update_cmd()` runs on GLib main loop.

**Anti-pattern to avoid (PITFALLS.md #1):** Do NOT re-render full markdown→Pango on every token. Accumulate tokens in a buffer, re-render on debounce (every 100ms or N tokens). During streaming, show plain text with minimal formatting.

### Pattern: DB Worker Extensions

Add new `DbInput`/`DbOutput` variants to the existing `DbService` Worker:
- StartConversation, LoadConversations, LoadMessages, SendMessage, RegenerateBranch, EditMessage, DeleteMessage, DeleteConversation
- New `src/db/conversations.rs` module for conversation/message CRUD
- Migration 004: conversations, messages, message_branches tables

### Pattern: ChatPanel Upgrade (SimpleComponent → Component)

Current `ChatPanel` is a `SimpleComponent` with just has_providers. Must upgrade to full `Component` with `CommandOutput` for streaming. This is the most significant structural change.

## Common Pitfalls (Phase 3 Specific)

### Pitfall: Streaming Cancellation Without Cleanup (PITFALLS.md #6)
- Use `shutdown.register(...).drop_on_shutdown()` for automatic cancellation
- Save partial response on cancel: preserve as complete message in history
- Track streaming state: `StreamingState` enum (Idle, Streaming, Cancelling)

### Pitfall: Markdown Re-rendering on Every Token (PITFALLS.md #1)
- Accumulate tokens in buffer, debounce Pango re-rendering (100ms or every 5 tokens)
- During streaming: show accumulated text as-is (or with minimal formatting)
- After streaming complete: full markdown→Pango conversion

### Pitfall: Conversation Tree Queries (PITFALLS.md #10)
- Use recursive CTEs for branch traversal: `WITH RECURSIVE branch AS (...)`
- Branch navigation: siblings = same parent_message_id
- Active branch per conversation: track in conversations table or in-memory

## Integration Points

| Boundary | Communication | Notes |
|----------|---------------|-------|
| ChatPanel → rig agent | `sender.command()` with tokio async | Stream tokens back as CommandOutput |
| ChatPanel → DbWorker | `DbInput` variants via `db_worker.sender()` | Save messages, load history |
| ChatPanel → AppModel | `ChatPanelOutput` messages | Route character selection, provider access |
| AppModel → ChatPanel | `ChatPanelMsg` input messages | Set active conversation, forward data |
| rig → GLib main loop | Relm4 `Sender<CommandOutput>` | Automatic GLib dispatch from command() |

## Out of Scope

- Token counting / context budget indicator — Phase 5
- Persona injection — Phase 4
- Lorebook keyword activation — Phase 5
- Generation parameter presets — Phase 6
- Regex post-processing — Phase 6
- FTS5 search across conversations — Phase 4+
- Group chat — v2

---
*Phase 3 research: 2026-04-30*
