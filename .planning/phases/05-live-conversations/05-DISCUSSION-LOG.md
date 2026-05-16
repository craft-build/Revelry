# Phase 5: Live Conversations - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-05-15
**Phase:** 05-Live Conversations
**Areas discussed:** Streaming transport, Prompt scope for Phase 5, Edit/resend behavior, Regenerate flow

---

## Streaming transport

| Option | Description | Selected |
|--------|-------------|----------|
| Tauri Channels (Recommended) | Request-scoped, typed channels. start_chat returns Channel<StreamChunk>. No cross-talk between streams. Closest to gtk_ref GLib channel pattern. | ✓ |
| Tauri Events (broadcast) | app-wide emit(). All conversations receive all tokens. Need client-side filtering. Risk of cross-talk. | |
| Hybrid: Channels + Events | Channels for streaming + global events for stream-complete. Best of both but more code. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Single start_chat command (Recommended) | One call: conversation_id + content → returns Channel. Simple flow. | ✓ |
| Two-step: prepare + send | Split into prepare_stream + send_message. More flexible but unnecessary complexity. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Explicit stop command (Recommended) | stop_generation Tauri command drops CancellationSender. tokio task detects via select!. Partial response finalized. | ✓ |
| Channel drop auto-cancel | Frontend drops Channel → Rust detects via Channel::closed(). Can't stop while staying on page. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Save on stream complete (Recommended) | User msg saved immediately. AI msg in memory only, saved when stream completes. Crash = lose AI response, keep user msg. | ✓ |
| Save both immediately | Both messages persisted right away. DB gets hammered with updates. Partial messages linger on failure. | |

**User's choice:** Tauri Channels, single start_chat, explicit stop, save on stream complete
**Notes:** Clean separation of concerns. Request-scoped channels prevent cross-talk. Explicit stop gives full control. Persist-on-complete avoids DB write amplification.

---

## Prompt scope for Phase 5

| Option | Description | Selected |
|--------|-------------|----------|
| Minimal: character only (Recommended) | System prompt = character personality + scenario. History = all messages. Persona/lorebook/budget = Phase 6. | ✓ |
| Medium: character + persona | Add active persona description to system prompt. More context but Phase 6 will refactor it. | |
| Full: port PromptEngine now | Port entire budget-aware engine. Violates phase boundary, creates rework. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Send all messages (Recommended) | No budget enforcement. API error on overflow shown inline. Phase 6 adds proper budget. | ✓ |
| Hard cap: last 20 | Arbitrary safety cutoff. Prevents some overflows but is a hack that Phase 6 replaces. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Inline error in chat (Recommended) | Error rendered as system-style message in conversation. User can retry or switch provider. | ✓ |
| Toast notification | Transient notification that may scroll away. Less visible. | |

**User's choice:** Minimal character-only prompt, all messages, inline error display
**Notes:** Clean Phase 5/6 boundary. Phase 5 focuses on the streaming pipeline; Phase 6 adds prompt intelligence.

---

## Edit/resend behavior

| Option | Description | Selected |
|--------|-------------|----------|
| Truncate from edit point (Recommended) | Delete all subsequent messages from DB and UI. Clean state, standard pattern. | ✓ |
| Soft delete (keep, mark hidden) | Mark as superseded, don't delete. Supports future branching. More complex state. | |
| Defer to Phase 8 | Only implement CHAT-01/02/03. Leave CHAT-04 unmet. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hover action on message (Recommended) | Pencil icon in hover actions on MessageBubble. Inline edit mode. Consistent pattern. | ✓ |
| Double-click to edit | Faster but less discoverable. Conflicts with text selection. | |

| Option | Description | Selected |
|--------|-------------|----------|
| User messages only (Recommended) | Edit only for user messages. AI messages corrected via Regenerate. Standard pattern. | ✓ |
| Both user and AI | Edit both. Editing AI message doesn't resend, just changes stored text. Confusing. | |

**User's choice:** Truncate, hover action trigger, user messages only
**Notes:** Truncation is clean and matches mainstream chat apps. Hover action consistent with existing patterns. User/AI split follows convention.

---

## Regenerate flow

| Option | Description | Selected |
|--------|-------------|----------|
| Replace in-place (Recommended) | Delete old AI message, stream new one. Conversation stays clean. Standard pattern. | ✓ |
| Append new, keep old | Keep both. Branch comparison possible but clutters conversation. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Hover action on last AI msg (Recommended) | Refresh icon on last AI MessageBubble hover. Only on most recent AI message. | ✓ |
| Button in input area | Dedicated regenerate button near MessageInput. More prominent but takes space. | |

| Option | Description | Selected |
|--------|-------------|----------|
| Same provider/model (Recommended) | Always use conversation's current provider. Switch via ChatHeader first. Simpler. | ✓ |
| Allow switching on regen | Quick picker in regenerate flow. More flexible but adds UI complexity. | |

**User's choice:** Replace in-place, hover action, same provider/model
**Notes:** Consistent with edit pattern. Replace keeps conversations clean. Same provider avoids a secondary picker UI.

---

## Claude's Discretion

- StreamChunk serialization format for Tauri Channel
- Whether start_chat also saves user message or separate call
- Inline error message rendering style
- Edit mode UI (textarea overlay vs. replacement component)
- Generation params resolution (provider defaults only)
- API key retrieval approach inside start_chat

## Deferred Ideas

None — discussion stayed within phase scope.
