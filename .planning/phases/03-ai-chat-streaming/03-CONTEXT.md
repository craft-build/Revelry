# Phase 3: AI Chat & Streaming - Context

**Gathered:** 2026-04-30
**Status:** Ready for planning

## Phase Boundary

Users can have streaming AI conversations with characters and manage conversation history with branching — the core product experience. This phase delivers the chat window with token-by-token streaming responses, markdown rendering, message editing with branch creation, and conversation management (create, resume, delete).

## Decisions

### Conversation Data Model

- **D-01:** Normalized 3-table schema for conversations, messages, and message_branches
  - `conversations` table: id, character_id, provider_id, creator_persona_id, created_at, updated_at
  - `messages` table: id, conversation_id, branch_id, role (user/assistant/system), content, timestamp, sort_order
  - `message_branches` table: id, conversation_id, parent_message_id, label, created_at — tracks fork points
  - Messages store content in TEXT columns (not JSON blobs), role as TEXT ('user', 'assistant', 'system')
  - Branch walk: messages with same branch_id form a branch; branch_id NULL means root

### Chat UI Architecture

- **D-02:** AdwToolbarView layout for main chat area
  - AdwHeaderBar at top with character avatar, character name, and action buttons (stop generation, send)
  - GtkScrolledWindow containing a vertical box of message widgets
  - Bottom input area with multi-line text entry and send button
  - Character's first_mes auto-sent as initial assistant message when starting a new conversation

- **D-03:** Message rendering as custom widgets
  - Character messages: left-aligned with small avatar, character name, and Pango markup body
  - User messages: right-aligned with accent color background, user name, Pango markup body
  - No message bubbles — uses clean row-style widgets for GNOME Consistency

- **D-04:** Sidebar conversation list
  - Each row: character avatar (circular), character name, last message preview (truncated), relative timestamp
  - Top entry: "New Conversation" button
  - AdwPreferencesHeader at top accepts keyboard search for filtering conversations

### Streaming & Rig Integration

- **D-05:** Glib::MainContext::channel() bridge for streaming
  - tokio async stream from rig-core → channel → GTK main loop
  - Each token chunk appended to the active message widget via GtkTextBuffer
  - Streaming runs in a background tokio task; channel dispatches to main thread

- **D-06:** Markdown rendering via pulldown-cmark → Pango
  - Full Markdown converted to Pango markup on each token (not incremental/lighter parser)
  - Supports bold, italic, headers, lists, code blocks via Pango
  - Column-based rendering (Pango markup in GtkLabel attributes)

- **D-07:** Cancellation behavior
  - Stop button or Escape cancels in-flight generation
  - Partial AI response is preserved as a complete message in history
  - Next user message continues from the partial response naturally

- **D-08:** Provider selection per conversation
  - Each conversation records which provider was active when created
  - Provider_id stored in conversations table
  - All streaming for that conversation uses the same provider

- **D-09:** Prompt assembly
  - System prompt from character's system_prompt field (prepended)
  - Conversation history: all messages in current branch up to fork point
  - No persona injection or lorebook activation (respectively handled in Phases 4 and 5)
  - Conversation prompt assembled client-side; no separate service layer needed

### Message Branching Model

- **D-10:** Branch indicator with popover switcher
  - Messages belonging to a branch show a colored left border indicator
  - Context menu or hover popover lists active branch + sibling branches
  - Branch switching displays an optimistic preview with load spinner

### Edit Data Flow

- **D-11:** Per-message edit popover
  - Right-click or context button on a message → Edit/Regenerate/Delete popover
  - Edit: opens input dialog with current message text
  - Confirming edit: wipes branch history after the edit point, sends modified message + history to provider, regenerates responses for new branch
  - A cancel breadcrumb shown during in-flight regeneration

### Error Handling

- **D-12:** Toast + message state for errors
  - Error shown via toast notification (AdwToast)
  - Partial message stays in conversation with small error indicator badge
  - User can retry generation or edit the failing message

## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Phase Scope & Requirements
- `.planning/REQUIREMENTS.md` — PROV-03, PROV-04, CONV-01 through CONV-10
- `.planning/ROADMAP.md` — Phase 3 definition and success criteria
- `.planning/PROJECT.md` — Core value and constraints

### Existing Domain & Database
- `src/domain/character.rs` — CharacterCardV2 domain model (first_mes, system_prompt, personality, scenario are used in prompt assembly)
- `src/domain/provider.rs` — ProviderConfig, ProviderType (provider selection for conversations)
- `src/db/mod.rs` — DbService Worker, DbInput/DbOutput enums (already established pattern)
- `src/db/migrations.rs` — Existing migration pattern (4 migrations so far: providers, app_settings, characters+fts, character extras)
- `src/db/models.rs` — Row conversion helpers (pattern to follow for new models)

### UI Framework
- `src/ui/chat.rs` — Current stub: AdwStatusPage placeholder (replace entirely in Phase 3)
- `src/app.rs` — AppModel with sidebar, ChatPanel placeholder, view switching
- `src/ui/settings/mod.rs` — SettingsPanel pattern (Component trait, output messages, worker forwarding via AppModel)
- `src/ui/characters.rs` — CharactersPanel pattern (list architecture for conversation list reuse)

### AI Inference
- `.planning/research/STACK.md` — rig-core 0.36.0 for provider abstraction, tokio for async streaming

### Design Specs
- `docs/Revelry_Design_Specification_v2.0.docx` — §10.4 Chat View, §10.5 Message Management, §10.6 Streaming

## Existing Code Insights

### Reusable Assets
- **DbService Worker** (`src/db/mod.rs`): Serial background thread pattern for DB operations. Add conversation/message CRUD to the existing worker.
- **AppModel** (`src/app.rs`): `ActiveView` enum, sidebar toggle, toast overlay, child panel controller pattern. ChatPanel controller already exists (currently just placeholder).
- **CharacterCardV2** (`src/domain/character.rs`): `system_prompt`, `first_mes`, `alternate_greetings` are used for prompt assembly and auto-starting conversations.
- **ProviderConfig** (`src/domain/provider.rs`): Provider selection for conversations.

### Established Patterns
- **Database:** NaiveDateTime as TEXT in SQLite, serde JSON in TEXT columns, prepare_cached() for CRUD, upsert via exists-check-then-insert-or-update.
- **UI Components:** Component trait (not SimpleComponent) for components with async operations and background channel output. AppModel bridges between UI components and DbInput via output messages.
- **Async:** tokio on background threads, glib::MainContext::channel() for UI thread dispatch.
- **Error handling:** RunResult + toast notifications. DB errors routed through DbOutput::Error → AppMsg::ShowToast.

### Integration Points
- **DbService** needs new DbInput variants: StartConversation, LoadConversations, LoadMessages, SendMessage, RegenerateBranch, EditMessage, DeleteMessage, DeleteConversation
- **DbService** needs new DbOutput variants: ConversationsLoaded, MessagesLoaded, ConversationCreated, MessageSent, etc.
- **AppModel** needs to initialize the active conversation when user selects a sidebar item
- **ChatPanel** replaces StatusPage with full chat UI layout
- **Sidebar** replaces GtkListBox placeholder with conversation list

## Specific Ideas

- Character selection flow: User clicks a character card in the Characters view to start a new conversation with it (not discussed in detail this session — planner should design natural UX).
- Conversation resume: Selecting a conversation from sidebar loads history and scrolls to bottom.

## Deferred Ideas

- Token counting/context budget indicator — belongs in Phase 5 (Context Engineering)
- Persona injection into prompts — belongs in Phase 4 (Personas & Template Resolution)
- Lorebook keyword activation — belongs in Phase 5
- Generation parameter presets — belongs in Phase 6
- Regex post-processing — belongs in Phase 6
- FTS5 search across conversations — belongs in Phase 4 or later (CONV-A01 is v2 detailed)
- Markdown toolbars/formatting in input — may revisit for polish
- Avatar row on message — lightweight avatar (circular, small) decided for GNOME simplicity

---

*Phase: 03-ai-chat-streaming*
*Context gathered: 2026-04-30*
