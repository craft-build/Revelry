# Phase 3: AI Chat & Streaming - Discussion Log

**Gathered:** 2026-04-30

## Conversation data model

**Question:** How should conversations, messages, and branches map to the SQLite schema?
- Options: Normalized 3 tables, 3 tables + JSON messages, You decide
- **Selected:** Normalized 3 tables (Recommended)

**Follow-up:** When a message is edited or regenerated, what does the store look like?
- Options: Branch table with fork point, Every branch message is a new row, You decide
- **Selected:** Branch table with fork point (Recommended)

## Chat UI architecture

**Question:** How should the main chat view be structured?
- Options: AdwToolbarView with message ScrollView, Custom message view from scratch, You decide
- **Selected:** AdwToolbarView with message ScrollView

**Follow-up:** How should messages be rendered as widgets?
- Options: AdwActionRow with text child, Bubble-style cards, You decide
- **Selected:** AdwActionRow with text child

**Follow-up:** How should the conversation list in the sidebar look and behave?
- Options: Custom AdwListBoxRow list, Searchable filter row, You decide
- **Selected:** Searchable filter row

## Streaming & rig integration

**Question:** How should streaming responses be delivered from rig-core's async stream to the GTK UI thread?
- Options: GStreamChannel on main loop, Relm4 Command for streaming, You decide
- **Selected:** GStreamChannel on main loop (Recommended)

**Follow-up:** How should streamed messages be rendered as markdown?
- Options: Convert to Pango on each token, Pango-compatible Markdown, You decide
- **Selected:** Convert to Pango on each token

**Follow-up:** When the user cancels generation, what happens to the partial AI response?
- Options: Preserve partial as complete message, Commit as partial but editable, You decide
- **Selected:** Preserve partial as complete message

**Follow-up:** How is the AI prompt assembled for each new message in a conversation?
- Options: System prompt + conversation history, System prompt + all history, You decide
- **Selected:** System prompt + conversation history

## Message branching model

**Question:** How should the UI convey and allow branching with a tab popover?
- Options: Indicator with popover switcher, Full sidebar is branch-aware, You decide
- **Selected:** Branch indicator with popover switcher

## Edit data flow

**Question:** How should editing a message work in the conversation?
- Options: Inline edit on hover, Per-message edit popover, You decide
- **Selected:** Per-message edit popover

## Provider selection

**Question:** How does the chat layer know which AI provider to use for responses?
- Options: Per-conversation provider selection, Per-message provider selection, You decide
- **Selected:** Per-conversation provider selection

## Starting conversations

**Question:** When starting a new conversation, what happens automatically?
- Options: Auto-generate first message, Empty conversation, You decide
- **Selected:** Auto-generate first message (recommended)

## Chat input

**Question:** How should the chat input (text entry) behave?
- Options: Enter = send, Ctrl+Enter = newline, Enter = send, Shift+Enter = newline, Multi-line by default
- **Selected:** Multi-line by default

## Error handling

**Question:** How should errors (provider failure, rate limit, connection drop) be handled?
- Options: Toast + message state, Inline error message, You decide
- **Selected:** Toast + message state

---

*Phase: 03-ai-chat-streaming*
*Discussion log captured: 2026-04-30*
