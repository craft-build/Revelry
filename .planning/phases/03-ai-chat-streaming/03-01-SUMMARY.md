---
phase: 03-ai-chat-streaming
plan: 01
type: execute
subsystem: data-layer
requires: [PROV-03, CONV-08]
provides: [conversation-domain, db-migration-005, conversation-crud, ai-service, markdown-rendering]
affects: [src/domain, src/db, src/service]
tech_stack:
  added: []
  patterns: [db-migration, domain-types, service-layer]
duration_minutes: 45
completed: 2026-04-30T22:50:00Z
---

# Phase 03 Plan 01: Conversation Data Layer & AI Service Summary

**One-liner:** Built the conversation data layer (domain types, DB schema, CRUD) and AI service layer (streaming foundation, prompt assembly, markdown rendering) — the foundation for chat UI.

## What Was Built

### Task 1: Conversation Domain & Database (30 min)

Created conversation persistence infrastructure:

| Component | Purpose | Key Features |
|-----------|---------|--------------|
| `Conversation` | Chat session model | UUID v4 id, character_id, provider_id, creator_persona_id, timestamps |
| `Message` | Individual message model | UUID v4 id, conversation_id, branch_id, role (User/Assistant/System), content, sort_order |
| `MessageBranch` | Branching model | UUID v4 id, parent_message_id, label, created_at |
| `MessageRole` | Role enum | Display + FromStr traits for DB serialization |
| DB Migration 005 | Schema tables | conversations, messages, message_branches with foreign keys + indexes |
| `conversations.rs` | CRUD operations | start_conversation, load_conversations, load_messages, save_message, delete_conversation, etc. |
| Row converters | DB ↔ Domain | row_to_conversation, row_to_message, row_to_message_branch |
| DbInput/DbOutput | Worker messages | All conversation operations as enum variants |

**Key decisions:**
- NaiveDateTime as TEXT columns per Phase 1 convention
- Branch_id NULL means root branch
- CASCADE deletes on foreign key constraints
- sort_order ASC for message ordering

### Task 2: AI Service Layer (15 min)

Created foundation for AI streaming:

| Component | Purpose | Key Features |
|-----------|---------|--------------|
| `StreamChunk` | Streaming enum | Token(String), Complete(String), Error(String) |
| `AiService` | Provider utilities | requires_api_key(), provider_type() helpers |
| `PromptEngine` | Prompt assembly | assemble(), assemble_initial() methods |
| `AssembledPrompt` | Prompt container | preamble + chat_history as rig messages |
| `markdown_to_pango()` | Text rendering | bold → `<b>`, italic → `<i>`, headers → `<span size="..."><b>`, code → `<tt>` |
| `escape_pango()` | Security | & → `&amp;`, < → `&lt;`, > → `&gt;` (T-03-01 mitigation) |

**Supported markdown:** bold (`**text**`), italic (`*text*`), headers (#/##/###), inline code (`` `code` ``), code blocks (```), lists (• bullet), horizontal rules.

## Files Created/Modified

```
src/
├── domain/
│   ├── mod.rs              (+1 line: pub mod conversation)
│   └── conversation.rs     (new: 272 lines - domain types + tests)
├── db/
│   ├── mod.rs              (+42 lines: conversation DbInput/DbOutput variants)
│   ├── conversations.rs    (new: 459 lines - CRUD + tests)
│   ├── migrations.rs       (+45 lines: migration 005)
│   └── models.rs           (+50 lines: row converters)
└── service/
    ├── mod.rs              (+3 lines: pub mod ai, markdown, prompt)
    ├── ai.rs               (new: 82 lines - StreamChunk, AiService)
    ├── markdown.rs         (new: 143 lines - Pango converter)
    └── prompt.rs           (new: 271 lines - prompt assembly)
```

## Test Results

**Total tests:** 130 passed

**New tests added:**
- 10 conversation domain tests (constructors, serialization)
- 9 conversation CRUD tests (start, load, save, delete, branches)
- 12 markdown tests (escape, bold, italic, headers, code, lists)
- 8 prompt assembly tests (assemble, system prompt, chat history)
- 3 AI service tests (provider type, api key check, StreamChunk)

**All existing tests:** 71 still passing (0 regressions)

## Deviations from Plan

None - plan executed exactly as written.

## Success Criteria Verification

| Criterion | Status | Evidence |
|-----------|--------|----------|
| Conversation, Message, MessageBranch types | ✓ | src/domain/conversation.rs lines 1-130 |
| DB migration 005 creates tables | ✓ | src/db/migrations.rs lines 77-108 |
| Conversation CRUD works | ✓ | 9 tests in conversations.rs, all pass |
| PromptEngine::assemble | ✓ | src/service/prompt.rs lines 38-63 |
| markdown_to_pango with escaping | ✓ | src/service/markdown.rs lines 10-76 |
| StreamChunk enum defined | ✓ | src/service/ai.rs lines 7-15 |
| All tests pass | ✓ | 130/130 passed |

## Key Links Verified

| From | To | Via | Status |
|------|-----|-----|--------|
| conversations.rs | db/mod.rs | DbInput/DbOutput enums | ✓ All match arms added |
| markdown.rs | Threat Model | escape_pango() security | ✓ T-03-01 mitigation |
| prompt.rs | conversation.rs | Message → rig::message::Message | ✓ build_chat_history() |

## Next Steps

Ready for **03-02 ChatPanel** — this foundation provides:
- Domain types for conversation state
- DB operations for persistence
- Markdown rendering for AI responses
- Prompt assembly for API calls
- StreamChunk enum for token streaming

The ChatPanel (Plan 02) will consume these through:
- `DbInput::StartConversation`, `LoadConversations`, `LoadMessages`, `SaveMessage`
- `PromptEngine::assemble()` for constructing prompts
- `StreamChunk` for displaying streaming tokens
- `markdown_to_pango()` for rendering responses

---

**Commit hashes:**
- d769f1a feat(03-01): add conversation domain types, DB migration 005, and CRUD operations
- 1f9f133 feat(03-01): add AI service, prompt assembly, and Pango markdown converter
