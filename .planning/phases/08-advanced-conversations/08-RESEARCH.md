# Phase 8 Research: Advanced Conversations

**Researched:** 2026-05-18
**Question:** What do I need to know to PLAN this phase well?

---

## Summary

Phase 8 adds three capabilities to an already-mature chat system: (1) message tree branching, (2) branch navigation UI, and (3) FTS5 full-text search. The codebase is well-structured for extension — the gtk_ref has complete working implementations for all three, and the active codebase has clean seams at every integration point. The main complexity is coordinating the branch-aware changes across the backend pipeline (migration → domain → DB functions → DbCommand → Tauri commands → frontend types → state → components).

---

## Research Findings

### 1. Schema Migration — What Exists vs What's Needed

**Current state (3 migrations):**

- Migration 1 (Phase 2): Core schema. Messages table has `id, conversation_id, role, content, sort_order, timestamp` — **no `branch_id` column**. Test explicitly asserts `!columns.contains(&"branch_id")`.
- Migration 2 (Phase 3): Lorebooks refactor, persona avatar_path.
- Migration 3 (Phase 4): Provider models JSON, conversation model_name.

**Migration 4 (Phase 8) needs:**

```sql
-- New table
CREATE TABLE message_branches (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_message_id TEXT NOT NULL,  -- the message where the fork happens
    label TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_message_branches_parent ON message_branches(parent_message_id);

-- Add column to existing messages
ALTER TABLE messages ADD COLUMN branch_id TEXT REFERENCES message_branches(id) ON DELETE CASCADE;
CREATE INDEX idx_messages_conv_branch ON messages(conversation_id, branch_id);

-- FTS5 virtual table for messages + conversation titles
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    content=messages,
    content_rowid=rowid
);

-- FTS5 sync triggers for messages
CREATE TRIGGER messages_ai AFTER INSERT ON messages BEGIN
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;
CREATE TRIGGER messages_ad AFTER DELETE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
END;
CREATE TRIGGER messages_au AFTER UPDATE ON messages BEGIN
    INSERT INTO messages_fts(messages_fts, rowid, content) VALUES ('delete', old.rowid, old.content);
    INSERT INTO messages_fts(rowid, content) VALUES (new.rowid, new.content);
END;
```

**Key insight:** The existing test `test_messages_no_branch_id` must be updated/removed — it currently asserts that `branch_id` does NOT exist.

**FTS5 pattern reference:** The existing `characters_fts` in migration 1 is the exact template. It uses `content=characters, content_rowid=rowid` with INSERT/UPDATE/DELETE triggers. The `messages_fts` follows the same pattern, indexing only `content` (conversation title search can be handled by joining or a separate index — D-14 says "follow characters_fts pattern"). Also consider a separate `conversations_fts` for titles, or use a unified approach.

**sanitize_fts_query reuse:** `src-tauri/src/db/characters.rs` already has `sanitize_fts_query()` as a private function. It needs to be made `pub(crate)` or moved to a shared location for the message search to use.

### 2. Domain Model Changes

**Active `Message` struct** (`src-tauri/src/domain/conversation.rs`):

```rust
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sort_order: i32,
    pub timestamp: DateTime<Utc>,
}
```

Must add: `pub branch_id: Option<String>,`

**New `MessageBranch` struct** (port from gtk_ref, adapt `NaiveDateTime` → `DateTime<Utc>`):

```rust
pub struct MessageBranch {
    pub id: String,
    pub conversation_id: String,
    pub parent_message_id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}
```

**Row mappers** (`src-tauri/src/db/models.rs`):

- `row_to_message` must read `branch_id` column (nullable)
- New `row_to_message_branch` mapper needed
- `row_to_message` currently reads 6 columns; after migration it reads 7

### 3. Branch-Aware Message Loading

**Current `load_messages`** (active `src-tauri/src/db/conversations.rs`):

```rust
pub fn load_messages(conn: &Connection, conversation_id: &str) -> Result<Vec<Message>>
```

Returns ALL messages for a conversation ordered by sort_order.

**Phase 8 needs** (from gtk_ref pattern, D-13):

```rust
pub fn load_messages(conn: &Connection, conversation_id: &str, branch_id: Option<&str>) -> Result<Vec<Message>>
```

**Branch loading strategy:** Per D-06 (shared history preserved), loading branch-aware messages means:

1. Load all messages with `branch_id IS NULL` (shared root/Main path)
2. PLUS all messages with `branch_id = ?` (the active branch)
3. Both ordered by sort_order

This is a **union query**:

```sql
SELECT ... FROM messages
WHERE conversation_id = ?1 AND (branch_id IS NULL OR branch_id = ?2)
ORDER BY sort_order ASC
```

When `branch_id` is None (viewing Main), just filter `branch_id IS NULL`.

**Impact on callers:** Every call site that calls `load_messages` must be updated:

- `DbWorker::handle_command` for `DbCommand::LoadMessages` — needs new optional param
- `load_chat_context` in `commands/chat.rs` — needs to pass active branch_id
- Tests in `conversations.rs`

### 4. DbCommand Extensions

**New DbCommand variants needed:**

```rust
// Branch operations
CreateBranch {
    conversation_id: String,
    parent_message_id: String,
    label: String,
    reply: oneshot::Sender<anyhow::Result<MessageBranch>>,
},
LoadBranchesForMessage {
    parent_message_id: String,
    reply: oneshot::Sender<anyhow::Result<Vec<MessageBranch>>>,
},
LoadAllBranches {
    conversation_id: String,
    reply: oneshot::Sender<anyhow::Result<Vec<MessageBranch>>>,
},
DeleteBranch {
    branch_id: String,
    reply: oneshot::Sender<anyhow::Result<()>>,
},

// Search
SearchMessages {
    query: String,
    reply: oneshot::Sender<anyhow::Result<Vec<SearchResult>>>,
},
```

**Modified DbCommand variant:**

```rust
LoadMessages {
    conversation_id: String,
    branch_id: Option<String>,  // NEW
    reply: oneshot::Sender<anyhow::Result<Vec<Message>>>,
},
```

**New domain types for search results:**

```rust
pub struct SearchResult {
    pub conversation_id: String,
    pub conversation_title: String,
    pub character_id: String,
    pub match_type: String,  // "title" or "message"
    pub message_snippet: Option<String>,
    pub message_id: Option<String>,
    pub branch_id: Option<String>,
    pub rank: f64,
}
```

### 5. New DB Functions Needed

Port from gtk_ref's `conversations.rs`:

- `create_branch(conn, &MessageBranch)` — already in gtk_ref, port directly
- `load_branches_for_message(conn, parent_message_id)` — already in gtk_ref
- `load_all_branches_for_conversation(conn, conversation_id)` — new, needed for branch count
- `delete_branch_by_id(conn, branch_id)` — deletes `message_branches` row + cascades to messages
- `search_messages(conn, query)` — new FTS5 search using `messages_fts`

**Forking function** (D-05 through D-08):

```rust
pub fn fork_from_message(
    conn: &Connection,
    conversation_id: &str,
    parent_message_id: &str,
) -> Result<MessageBranch>
```

This is a compound operation:

1. Determine next branch label (count existing branches, "Branch N+1")
2. Create `MessageBranch` record
3. Return the branch (messages on the branch are created by the chat flow)

When forking from an AI message (D-08), the frontend triggers regeneration on the new branch.

**delete_messages_after_sort_order change:** Currently does `DELETE FROM messages WHERE conversation_id = ?1 AND sort_order > ?2`. With branches, this must also respect the active branch — only truncate messages on the same branch path.

### 6. Tauri Commands

**New commands** (following pattern in `commands/conversations.rs`):

```rust
fork_conversation(db, conversation_id, parent_message_id) -> Result<MessageBranch, String>
get_branches_for_message(db, parent_message_id) -> Result<Vec<MessageBranch>, String>
get_branches_for_conversation(db, conversation_id) -> Result<Vec<MessageBranch>, String>
delete_branch(db, branch_id) -> Result<(), String>
search_messages(db, query) -> Result<Vec<SearchResult>, String>
```

**Modified command:**

```rust
get_messages(db, conversation_id, branch_id: Option<String>) -> Result<Vec<Message>, String>
```

**Command registration:** `src-tauri/src/lib.rs` needs the new commands added to the `invoke_handler` list.

### 7. Frontend Type Changes

**`src/lib/types/index.ts`:**

Add `branch_id` to Message:

```typescript
export interface Message {
  // ... existing fields
  branch_id?: string | null; // null = Main/root path
}
```

New types:

```typescript
export interface MessageBranch {
  id: string;
  conversation_id: string;
  parent_message_id: string;
  label: string;
  created_at: string;
}

export interface SearchResult {
  conversation_id: string;
  conversation_title: string;
  character_id: string;
  match_type: "title" | "message";
  message_snippet?: string;
  message_id?: string;
  branch_id?: string;
  rank: number;
}
```

### 8. AppState Changes

**New state fields:**

```typescript
active_branch_id = $state<string | null>(null);
branch_search_results = $state<SearchResult[]>([]);
is_searching = $state<boolean>(false);
```

**New actions:**

- `searchMessages(query)` — calls `invoke('search_messages')`, sets `branch_search_results`
- `switchBranch(branchId)` — sets `active_branch_id`, reloads messages with branch filter
- `forkFromMessage(messageId)` — calls `invoke('fork_conversation')`, creates branch, switches to it
- `deleteBranch(branchId)` — calls `invoke('delete_branch')`, removes branch from state

**Modified actions:**

- `startChat()` — must include `branch_id` on new messages
- `regenerate()` — when on a branch, must pass branch context
- `editResend()` — branch-aware truncation
- `openConversation()` — must accept and set initial branch_id
- `hydrate()` — load branch info for active conversation

### 9. Component Changes

**MessageBubble.svelte:**

- Add GitBranch icon to hover actions row (all messages, not just last)
- Add BranchBadge component for messages with sibling branches
- Add BranchPopover (bits-ui Popover) anchored to BranchBadge
- Pass `branch_id` awareness through props

**ConversationSidebar.svelte:**

- Replace client-side `filtered` derived (current LIKE filter) with backend FTS5 search
- Add debounced search (300ms)
- When search returns message matches, show snippet line below conversation entry
- Handle loading state (`is_searching`)
- On result click: navigate + auto-switch branch if needed

**`chat/[id]/+page.svelte`:**

- Pass `branch_id` to `getMessages` call
- Handle branch switching (reload messages with different branch)
- Auto-scroll to fork point on branch switch

**New components:**

- `BranchBadge.svelte` — pill badge showing "1/3" position
- `BranchPopover.svelte` — bits-ui Popover listing branches with preview, delete action

### 10. Integration Points and Dependencies

**Streaming coexistence with branching:**

- `startChat()` creates new messages. When on a branch, the message must carry the active `branch_id`.
- The `start_chat` Tauri command currently saves messages without `branch_id`. It needs to accept and persist `branch_id`.
- `regenerate()` on a branch must delete only the branch's last AI message, not root messages.
- `editResend()` truncation must respect branch boundaries — only truncate messages on the same branch path.

**PromptEngine coexistence:**

- When assembling history for prompt, only include messages visible on the active branch path (shared root + branch messages).
- This requires `load_messages` to return the correct union (root + branch).

**Token budget + branching:**

- Budget display in MessageInput must reflect only active branch messages.
- `load_chat_context` in `commands/chat.rs` must pass branch_id.

### 11. Coexistence with Edit/Resend and Regenerate

**Current edit/resend flow** (`edit_and_resend` command):

1. Find message by id → get sort_order
2. Update message content
3. `delete_messages_after_sort_order` — deletes ALL subsequent messages regardless of branch
4. Re-stream new AI response

**Phase 8 complication:** Step 3 must only delete messages on the same branch path. Messages on other branches must be preserved. The `delete_messages_after_sort_order` SQL needs a branch-aware variant:

```sql
DELETE FROM messages
WHERE conversation_id = ?1
  AND sort_order > ?2
  AND (branch_id = ?3 OR (branch_id IS NULL AND ?3 IS NULL))
```

**Regenerate on a branch:** Currently deletes the last AI message globally. Must only delete on the active branch:

```sql
DELETE FROM messages WHERE id = ?1  -- still works since we have the specific message id
```

But finding the "last AI message" must be branch-scoped.

### 12. FTS5 Search Implementation

**search_messages SQL** (from D-14, D-16):

```sql
SELECT
    m.id as message_id,
    m.conversation_id,
    m.branch_id,
    c.title as conversation_title,
    c.character_id,
    'message' as match_type,
    snippet(messages_fts, 0, '<mark>', '</mark>', '...', 32) as message_snippet,
    f.rank
FROM messages_fts f
JOIN messages m ON m.rowid = f.rowid
JOIN conversations c ON c.id = m.conversation_id
WHERE messages_fts MATCH ?1
ORDER BY rank
LIMIT 50;
```

Also union with conversation title search for comprehensive results. The `sanitize_fts_query` function from `characters.rs` can be reused.

### 13. Files Touch Map

**Backend (Rust) — 10 files:**

| File                                      | Change                                                                                                                   |
| ----------------------------------------- | ------------------------------------------------------------------------------------------------------------------------ |
| `src-tauri/src/db/migrations.rs`          | Add migration 4 (message_branches, branch_id, messages_fts)                                                              |
| `src-tauri/src/domain/conversation.rs`    | Add `branch_id` to Message, add MessageBranch struct                                                                     |
| `src-tauri/src/db/models.rs`              | Update `row_to_message`, add `row_to_message_branch`, add `row_to_search_result`                                         |
| `src-tauri/src/db/conversations.rs`       | Add branch functions, modify `load_messages` signature, add `search_messages`, modify `delete_messages_after_sort_order` |
| `src-tauri/src/db/characters.rs`          | Make `sanitize_fts_query` `pub(crate)`                                                                                   |
| `src-tauri/src/db/mod.rs`                 | Add DbCommand variants, update LoadMessages, add handle_command match arms                                               |
| `src-tauri/src/commands/conversations.rs` | Add fork, get_branches, delete_branch, search_messages commands; modify get_messages                                     |
| `src-tauri/src/commands/chat.rs`          | Pass branch_id through load_chat_context and all chat commands                                                           |
| `src-tauri/src/ai/mod.rs`                 | No changes needed (stream_chat already generic)                                                                          |
| `src-tauri/src/lib.rs`                    | Register new Tauri commands                                                                                              |

**Frontend (TypeScript/Svelte) — 8 files:**

| File                                                   | Change                                                                   |
| ------------------------------------------------------ | ------------------------------------------------------------------------ |
| `src/lib/types/index.ts`                               | Add branch_id to Message, add MessageBranch, SearchResult types          |
| `src/lib/stores/app.svelte.ts`                         | Add branch state, actions, modify chat actions for branch awareness      |
| `src/lib/data/api.ts`                                  | Add forkConversation, getBranches, deleteBranch, searchMessages wrappers |
| `src/lib/components/chat/MessageBubble.svelte`         | Add GitBranch action, BranchBadge, BranchPopover                         |
| `src/lib/components/chat/BranchBadge.svelte`           | NEW — pill badge component                                               |
| `src/lib/components/chat/BranchPopover.svelte`         | NEW — branch list popover                                                |
| `src/lib/components/layout/ConversationSidebar.svelte` | Replace client search with backend FTS, add message snippets             |
| `src/routes/chat/[id]/+page.svelte`                    | Branch-aware message loading, branch switching                           |

### 14. Risks and Open Questions

**Risk 1: Branch-aware message loading complexity.**
The union query approach (shared root + branch messages) must handle the case where a branch starts at sort_order N — root messages with sort_order ≤ N plus branch messages with sort_order > N. If the query isn't precise, messages from wrong branches could leak into view.

**Risk 2: Edit/resend + branch boundary interaction.**
If user edits a message that's on a branch, truncation must not delete root messages. The SQL needs careful branch scoping. The current `delete_messages_after_sort_order` is branch-unaware and will break if not updated.

**Risk 3: FTS5 performance with large message histories.**
FTS5 on messages content could grow large. The sync triggers add write overhead. Mitigation: FTS5 is proven at scale in SQLite, the characters_fts pattern works, and the search is user-triggered (not continuous).

**Risk 4: getMessages signature change is a breaking change.**
Every call site of `getMessages` / `DbCommand::LoadMessages` must pass the new `branch_id` param. Missing a call site means messages load without branch filtering (showing all branches mixed together). All callers must be updated atomically.

**Risk 5: PromptEngine must receive branch-scoped messages.**
If `load_chat_context` doesn't pass the correct branch_id, the prompt will include messages from all branches, causing confusing AI behavior.

**Open question: How to determine "last active branch" per conversation.**
D-13 says "default to the last active branch or root (NULL)". This needs either:

- A persisted per-conversation `active_branch_id` (DB column or app_state table)
- In-memory only (AppState tracks it, resets on app restart)
  The CONTEXT says "in-memory state only, or persisted" is at agent's discretion. Recommendation: in-memory for simplicity (D-13 just says "track in AppState").

**Open question: Branch creation as single vs multi-step Tauri command.**
A single `fork_conversation` command that creates the branch record and returns it is simplest. The frontend then handles the "regenerate on new branch" logic by calling `regenerate` with the new branch context. This keeps fork and regenerate as separate concerns.

### 15. Plan Ordering Recommendation

Based on dependency analysis, the natural plan order is:

**Plan 08-01: Schema + Domain + DB Functions (backend foundation)**

- Migration 4 (message_branches, branch_id, messages_fts)
- MessageBranch domain struct
- branch_id on Message
- Row mappers update
- Branch DB functions (create, load, delete)
- search_messages DB function
- Modify load_messages signature
- sanitize_fts_query pub
- All backend tests

**Plan 08-02: Tauri Commands + DbCommand wiring (IPC layer)**

- New DbCommand variants
- New Tauri commands
- Modify get_messages to accept branch_id
- Modify chat commands (start_chat, regenerate, edit_and_resend) for branch awareness
- Frontend API wrappers
- Frontend types (MessageBranch, SearchResult)

**Plan 08-03: Frontend State + Branch-Aware Loading (state layer)**

- AppState branch fields and actions
- Modify startChat/regenerate/editResend for branch_id
- Branch-aware message loading in chat page
- openConversation branch support

**Plan 08-04: UI Components (visual layer)**

- BranchBadge + BranchPopover components
- MessageBubble modifications (GitBranch action, badge, popover)
- ConversationSidebar enhanced search
- End-to-end integration testing

This ordering ensures each plan builds on a stable foundation and can be tested independently before the next plan starts.

---

## Canonical References Verified

All canonical references from 08-CONTEXT.md have been read and verified:

| Reference                                              | Status                | Key Insight                                                                                        |
| ------------------------------------------------------ | --------------------- | -------------------------------------------------------------------------------------------------- |
| `gtk_ref/src/domain/conversation.rs`                   | Read                  | MessageBranch struct with NaiveDateTime; Message with branch_id                                    |
| `gtk_ref/src/db/conversations.rs`                      | Read                  | Complete branch functions (create_branch, load_branches_for_message, load_messages with branch_id) |
| `gtk_ref/src/db/migrations.rs`                         | Read                  | Migration 5 pattern for message_branches table                                                     |
| `gtk_ref/src/ui/chat/branch_nav.rs`                    | Referenced in CONTEXT | GTK branch navigator pattern — adapt to Svelte Popover                                             |
| `gtk_ref/src/ui/chat/message_row.rs`                   | Referenced in CONTEXT | Branch indicator pattern — adapt to Badge                                                          |
| `src-tauri/src/db/migrations.rs`                       | Read                  | Current 3 migrations, migration 4 needed                                                           |
| `src-tauri/src/db/conversations.rs`                    | Read                  | Current load_messages (no branch_id), needs signature change                                       |
| `src-tauri/src/domain/conversation.rs`                 | Read                  | Current Message struct (no branch_id)                                                              |
| `src-tauri/src/db/mod.rs`                              | Read                  | Current DbCommand enum, needs new variants                                                         |
| `src-tauri/src/db/characters.rs`                       | Read                  | sanitize_fts_query to make pub                                                                     |
| `src-tauri/src/commands/`                              | Read                  | Command pattern (oneshot, invoke, State)                                                           |
| `src/lib/types/index.ts`                               | Read                  | Current types, needs MessageBranch + SearchResult                                                  |
| `src/lib/stores/app.svelte.ts`                         | Read                  | Current AppState, needs branch state + actions                                                     |
| `src/lib/components/chat/MessageBubble.svelte`         | Read                  | Current hover actions pattern                                                                      |
| `src/lib/components/layout/ConversationSidebar.svelte` | Read                  | Current client-side search, needs FTS5                                                             |
| `src/routes/chat/[id]/+page.svelte`                    | Read                  | Current message loading, needs branch param                                                        |

---

_Research complete. Ready for planning._
