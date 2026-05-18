# Phase 8: Advanced Conversations — Implementation Plan

**Requirements:** CONV-06, CONV-07, CONV-08

---

## Plan 08-01: Schema + Domain + DB Functions (Backend Foundation)

```yaml
wave: 1
depends_on: []
files_modified:
  - src-tauri/src/db/migrations.rs
  - src-tauri/src/domain/conversation.rs
  - src-tauri/src/db/models.rs
  - src-tauri/src/db/conversations.rs
  - src-tauri/src/db/characters.rs
autonomous: true
```

### Task 1: Add Migration 4 for branching + FTS5

<read_first>

- `src-tauri/src/db/migrations.rs` (current 3 migrations, add migration 4 after Phase 4)
- `src-tauri/src/db/characters.rs` (FTS5 trigger pattern reference)
  </read_first>

<action>
Add a new `M::up()` / `M::down()` pair as the 4th entry in the `get_migrations()` vector (after the Phase 4 migration). The migration adds:

1. `message_branches` table:

```sql
CREATE TABLE message_branches (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    parent_message_id TEXT NOT NULL,
    label TEXT NOT NULL,
    created_at TEXT NOT NULL
);
CREATE INDEX idx_message_branches_parent ON message_branches(parent_message_id);
CREATE INDEX idx_message_branches_conv ON message_branches(conversation_id);
```

2. `branch_id` column on `messages`:

```sql
ALTER TABLE messages ADD COLUMN branch_id TEXT REFERENCES message_branches(id) ON DELETE CASCADE;
CREATE INDEX idx_messages_conv_branch ON messages(conversation_id, branch_id);
```

3. `messages_fts` FTS5 virtual table + sync triggers (identical pattern to `characters_fts`):

```sql
CREATE VIRTUAL TABLE messages_fts USING fts5(
    content,
    content=messages,
    content_rowid=rowid
);
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

The `.down()` should drop all new objects in reverse order: triggers, fts table, indexes, column (via `CREATE TABLE` temp-copy since SQLite doesn't support DROP COLUMN easily — but rusqlite_migration requires down, so use `DROP TRIGGER IF EXISTS` + `DROP TABLE IF EXISTS messages_fts` + `DROP INDEX IF EXISTS idx_messages_conv_branch` + `DROP TABLE IF EXISTS message_branches`. For the `branch_id` column, since this is a development migration, just drop and recreate the messages table without it — OR accept that down migration won't perfectly remove the column and just drop the indexes/tables/triggers).

Update the `test_messages_no_branch_id` test to now assert that `branch_id` DOES exist (flip the assertion). Add new tests:

- `test_message_branches_table_exists` — asserts `message_branches` table exists
- `test_messages_fts_exists` — asserts `messages_fts` virtual table exists
- `test_messages_has_branch_id` — asserts `messages` has `branch_id` column
  </action>

<acceptance_criteria>

- `src-tauri/src/db/migrations.rs` contains a 4th `M::up()` entry with `CREATE TABLE message_branches`
- `src-tauri/src/db/migrations.rs` contains `ALTER TABLE messages ADD COLUMN branch_id`
- `src-tauri/src/db/migrations.rs` contains `CREATE VIRTUAL TABLE messages_fts USING fts5`
- `src-tauri/src/db/migrations.rs` contains 3 FTS sync triggers: `messages_ai`, `messages_ad`, `messages_au`
- `test_messages_no_branch_id` test removed or renamed to `test_messages_has_branch_id` asserting column exists
- `cargo test -p revelry-ui` exits 0
  </acceptance_criteria>

### Task 2: Add MessageBranch domain struct + branch_id to Message

<read_first>

- `src-tauri/src/domain/conversation.rs` (current Message struct)
- `gtk_ref/src/domain/conversation.rs` (MessageBranch reference pattern)
  </read_first>

<action>
In `src-tauri/src/domain/conversation.rs`:

1. Add `branch_id` field to `Message` struct:

```rust
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sort_order: i32,
    pub timestamp: DateTime<Utc>,
    pub branch_id: Option<String>,
}
```

2. Update `Message::new()` to set `branch_id: None`.
3. Update `Message::with_order()` to set `branch_id: None`.
4. Add new `MessageBranch` struct after `Message`:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessageBranch {
    pub id: String,
    pub conversation_id: String,
    pub parent_message_id: String,
    pub label: String,
    pub created_at: DateTime<Utc>,
}
```

5. Add `MessageBranch::new()` constructor that generates UUID v4 id, takes `conversation_id`, `parent_message_id`, `label` as params, and sets `created_at` to `Utc::now()`.

6. Add a new `SearchResult` struct:

```rust
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub conversation_id: String,
    pub conversation_title: String,
    pub character_id: String,
    pub match_type: String, // "title" or "message"
    pub message_snippet: Option<String>,
    pub message_id: Option<String>,
    pub branch_id: Option<String>,
    pub rank: f64,
}
```

7. Update `test_message_new_defaults` and `test_message_with_order` to verify `branch_id` is `None`.
8. Add serde roundtrip test for `MessageBranch`.
9. Add serde roundtrip test for `SearchResult`.
   </action>

<acceptance_criteria>

- `src-tauri/src/domain/conversation.rs` contains `pub branch_id: Option<String>` in `Message` struct
- `src-tauri/src/domain/conversation.rs` contains `pub struct MessageBranch` with fields: `id, conversation_id, parent_message_id, label, created_at`
- `src-tauri/src/domain/conversation.rs` contains `pub struct SearchResult` with fields: `conversation_id, conversation_title, character_id, match_type, message_snippet, message_id, branch_id, rank`
- `Message::new()` and `Message::with_order()` set `branch_id: None`
- `cargo test -p revelry-ui` exits 0
  </acceptance_criteria>

### Task 3: Update row mappers + add new mappers

<read_first>

- `src-tauri/src/db/models.rs` (current `row_to_message` mapper reads 6 columns)
  </read_first>

<action>
In `src-tauri/src/db/models.rs`:

1. Update `row_to_message` to read the new `branch_id` column (7th column, nullable):

```rust
pub fn row_to_message(row: &Row) -> rusqlite::Result<Message> {
    let timestamp_str: String = row.get("timestamp")?;
    let timestamp = parse_timestamp(&timestamp_str)?;
    let role_str: String = row.get("role")?;
    let role = MessageRole::from_str(&role_str).map_err(|e| {
        rusqlite::Error::FromSqlConversionFailure(0, rusqlite::types::Type::Text, Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, e)))
    })?;
    Ok(Message {
        id: row.get("id")?,
        conversation_id: row.get("conversation_id")?,
        role,
        content: row.get("content")?,
        sort_order: row.get("sort_order")?,
        timestamp,
        branch_id: row.get("branch_id")?,
    })
}
```

Note: `row.get("branch_id")` on a nullable column returns `Option<String>` via rusqlite's `FromSql` impl.

2. Add `row_to_message_branch` mapper:

```rust
pub fn row_to_message_branch(row: &Row) -> rusqlite::Result<MessageBranch> {
    let created_at_str: String = row.get("created_at")?;
    let created_at = parse_timestamp(&created_at_str)?;
    Ok(MessageBranch {
        id: row.get("id")?,
        conversation_id: row.get("conversation_id")?,
        parent_message_id: row.get("parent_message_id")?,
        label: row.get("label")?,
        created_at,
    })
}
```

3. Add the import for `MessageBranch` in models.rs (it's already in `crate::domain::conversation`).
   </action>

<acceptance_criteria>

- `row_to_message` reads `branch_id` via `row.get("branch_id")?`
- `row_to_message_branch` function exists returning `rusqlite::Result<MessageBranch>`
- `cargo test -p revelry-ui` exits 0
  </acceptance_criteria>

### Task 4: Add branch DB functions + branch-aware load_messages + search_messages

<read_first>

- `src-tauri/src/db/conversations.rs` (current `load_messages`, `save_message`, `delete_messages_after_sort_order`)
- `src-tauri/src/db/characters.rs` (reference `sanitize_fts_query` function — lines 1-50)
- `gtk_ref/src/db/conversations.rs` (branch function reference patterns)
  </read_first>

<action>
In `src-tauri/src/db/characters.rs`:
1. Change `fn sanitize_fts_query` visibility to `pub(crate) fn sanitize_fts_query` so it can be reused from conversations.rs.

In `src-tauri/src/db/conversations.rs`:

1. Add `use crate::db::characters::sanitize_fts_query;` import.
2. Add `use crate::domain::conversation::{MessageBranch, SearchResult};` import.
3. Add `use crate::db::models::row_to_message_branch;` import.

4. Modify `load_messages` signature to accept optional `branch_id`:

```rust
pub fn load_messages(conn: &Connection, conversation_id: &str, branch_id: Option<&str>) -> Result<Vec<Message>> {
```

When `branch_id` is `None` (Main/root): `WHERE conversation_id = ?1 AND branch_id IS NULL ORDER BY sort_order ASC`
When `branch_id` is `Some(id)`: `WHERE conversation_id = ?1 AND (branch_id IS NULL OR branch_id = ?2) ORDER BY sort_order ASC`
The union query for branches includes the shared root (NULL branch_id) + the active branch messages. This preserves D-06 (shared history).

5. Update `save_message` to persist `branch_id`:
   Add `branch_id` to the INSERT statement as column 7: `INSERT INTO messages (id, conversation_id, role, content, sort_order, timestamp, branch_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)` with `params![&msg.id, &msg.conversation_id, msg.role.to_string(), &msg.content, msg.sort_order, msg.timestamp.to_rfc3339(), &msg.branch_id]`.

6. Add `delete_messages_after_sort_order_branch` function:

```rust
pub fn delete_messages_after_sort_order_branch(
    conn: &Connection,
    conversation_id: &str,
    sort_order: i32,
    branch_id: Option<&str>,
) -> Result<()> {
    conn.prepare_cached(
        "DELETE FROM messages WHERE conversation_id = ?1 AND sort_order > ?2 AND (branch_id = ?3 OR (branch_id IS NULL AND ?3 IS NULL))"
    )?.execute(params![conversation_id, sort_order, branch_id])?;
    // Update conversation preview (same as delete_messages_after_sort_order)
    let now = Utc::now();
    let preview: Option<String> = conn.prepare_cached(
        "SELECT content FROM messages WHERE conversation_id = ?1 ORDER BY sort_order DESC LIMIT 1"
    )?.query_row(params![conversation_id], |row| row.get(0)).ok();
    conn.prepare_cached(
        "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3"
    )?.execute(params![now.to_rfc3339(), preview.unwrap_or_default(), conversation_id])?;
    Ok(())
}
```

7. Add `create_branch` function:

```rust
pub fn create_branch(conn: &Connection, branch: &MessageBranch) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO message_branches (id, conversation_id, parent_message_id, label, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?.execute(params![
        &branch.id,
        &branch.conversation_id,
        &branch.parent_message_id,
        &branch.label,
        branch.created_at.to_rfc3339(),
    ])?;
    Ok(())
}
```

8. Add `load_branches_for_message` function:

```rust
pub fn load_branches_for_message(conn: &Connection, parent_message_id: &str) -> Result<Vec<MessageBranch>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, parent_message_id, label, created_at
         FROM message_branches WHERE parent_message_id = ?1 ORDER BY created_at ASC"
    )?;
    let branches = stmt.query_map(params![parent_message_id], |row| row_to_message_branch(row))?;
    let mut result = Vec::new();
    for b in branches { result.push(b?); }
    Ok(result)
}
```

9. Add `load_all_branches_for_conversation` function:

```rust
pub fn load_all_branches_for_conversation(conn: &Connection, conversation_id: &str) -> Result<Vec<MessageBranch>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, parent_message_id, label, created_at
         FROM message_branches WHERE conversation_id = ?1 ORDER BY created_at ASC"
    )?;
    let branches = stmt.query_map(params![conversation_id], |row| row_to_message_branch(row))?;
    let mut result = Vec::new();
    for b in branches { result.push(b?); }
    Ok(result)
}
```

10. Add `delete_branch_by_id` function:

```rust
pub fn delete_branch_by_id(conn: &Connection, branch_id: &str) -> Result<()> {
    // CASCADE on FK will delete associated messages
    conn.prepare_cached("DELETE FROM message_branches WHERE id = ?1")?
        .execute(params![branch_id])?;
    // Update conversation preview
    // Find the conversation for this branch
    let conv_id: Option<String> = conn.prepare_cached(
        "SELECT conversation_id FROM message_branches WHERE id = ?1"
    )?.query_row(params![branch_id], |row| row.get(0)).ok();
    // Note: after deletion, can't query. Should get conv_id before deletion.
    Ok(())
}
```

Actually, fix this — get the conversation_id BEFORE deleting:

```rust
pub fn delete_branch_by_id(conn: &Connection, branch_id: &str) -> Result<()> {
    let conv_id: Option<String> = conn.prepare_cached(
        "SELECT conversation_id FROM message_branches WHERE id = ?1"
    )?.query_row(params![branch_id], |row| row.get(0)).ok();
    conn.prepare_cached("DELETE FROM message_branches WHERE id = ?1")?
        .execute(params![branch_id])?;
    if let Some(cid) = conv_id {
        let now = Utc::now();
        let preview: Option<String> = conn.prepare_cached(
            "SELECT content FROM messages WHERE conversation_id = ?1 ORDER BY sort_order DESC LIMIT 1"
        )?.query_row(params![cid], |row| row.get(0)).ok();
        conn.prepare_cached(
            "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3"
        )?.execute(params![now.to_rfc3339(), preview.unwrap_or_default(), cid])?;
    }
    Ok(())
}
```

11. Add `count_branches_for_conversation` function (for auto-labeling):

```rust
pub fn count_branches_for_conversation(conn: &Connection, conversation_id: &str) -> Result<i64> {
    let count: i64 = conn.prepare_cached(
        "SELECT COUNT(*) FROM message_branches WHERE conversation_id = ?1"
    )?.query_row(params![conversation_id], |row| row.get(0))?;
    Ok(count)
}
```

12. Add `search_messages` function (searches both message content via FTS5 and conversation titles via LIKE, per D-14):

```rust
pub fn search_messages(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let sanitized = sanitize_fts_query(query);
    if sanitized.is_empty() {
        return Ok(Vec::new());
    }

    // Search messages via FTS5
    let mut msg_stmt = conn.prepare_cached(
        "SELECT m.id as message_id, m.conversation_id, m.branch_id,
                c.title as conversation_title, c.character_id,
                'message' as match_type,
                snippet(messages_fts, 0, '<mark>', '</mark>', '...', 32) as message_snippet,
                f.rank
         FROM messages_fts f
         JOIN messages m ON m.rowid = f.rowid
         JOIN conversations c ON c.id = m.conversation_id
         WHERE messages_fts MATCH ?1
         ORDER BY rank
         LIMIT 50"
    )?;
    let msg_results = msg_stmt.query_map(params![sanitized], |row| {
        Ok(SearchResult {
            conversation_id: row.get("conversation_id")?,
            conversation_title: row.get("conversation_title")?,
            character_id: row.get("character_id")?,
            match_type: "message".to_string(),
            message_snippet: row.get("message_snippet")?,
            message_id: row.get("message_id")?,
            branch_id: row.get("branch_id")?,
            rank: row.get("rank")?,
        })
    })?;
    let mut result = Vec::new();
    for r in msg_results { result.push(r?); }

    // Also search conversation titles via LIKE (D-14: FTS5 covers both)
    let like_pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
    let mut title_stmt = conn.prepare_cached(
        "SELECT id as conversation_id, title as conversation_title,
                character_id, 'title' as match_type,
                NULL as message_snippet, NULL as message_id,
                NULL as branch_id, 0.0 as rank
         FROM conversations
         WHERE title LIKE ?1
         LIMIT 20"
    )?;
    let title_results = title_stmt.query_map(params![like_pattern], |row| {
        Ok(SearchResult {
            conversation_id: row.get("conversation_id")?,
            conversation_title: row.get("conversation_title")?,
            character_id: row.get("character_id")?,
            match_type: row.get("match_type")?,
            message_snippet: row.get("message_snippet")?,
            message_id: row.get("message_id")?,
            branch_id: row.get("branch_id")?,
            rank: row.get("rank")?,
        })
    })?;
    for r in title_results { result.push(r?); }

    // Deduplicate: if a conversation already matched via message FTS, skip the title match
    let mut seen_conv_ids = std::collections::HashSet::new();
    result.retain(|r| seen_conv_ids.insert(r.conversation_id.clone()));

    Ok(result)
}
```

13. Update ALL callers of `load_messages` in this file's tests to pass the new `branch_id` parameter (use `None`). This includes every test that calls `load_messages(&conn, ...)` — change to `load_messages(&conn, ..., None)`.

14. Update `save_message` tests to verify `branch_id` is persisted (insert a message with `branch_id: None`, load it back, assert `branch_id` is `None`).

15. Add test for branch functions:

- `test_create_and_load_branch` — creates a branch, loads it back, asserts fields match
- `test_load_messages_branch_aware` — creates messages with and without branch_id, loads with branch filter, asserts correct subset
- `test_search_messages_fts` — saves messages, searches, asserts results contain snippet with `<mark>` tags
  </action>

<acceptance_criteria>

- `src-tauri/src/db/characters.rs` contains `pub(crate) fn sanitize_fts_query`
- `src-tauri/src/db/conversations.rs` has `load_messages` with signature `(conn, conversation_id: &str, branch_id: Option<&str>)`
- `src-tauri/src/db/conversations.rs` has `save_message` INSERT that includes `branch_id` column
- `src-tauri/src/db/conversations.rs` has `fn create_branch`, `fn load_branches_for_message`, `fn load_all_branches_for_conversation`, `fn delete_branch_by_id`, `fn count_branches_for_conversation`, `fn search_messages`
- `src-tauri/src/db/conversations.rs` has `fn delete_messages_after_sort_order_branch`
- All existing `load_messages` call sites in test code pass `None` as 3rd argument
- `cargo test -p revelry-ui` exits 0
  </acceptance_criteria>

---

## Plan 08-02: DbCommand Wiring + Tauri Commands + Frontend Types + API (IPC Layer)

```yaml
wave: 2
depends_on: ["08-01"]
files_modified:
  - src-tauri/src/db/mod.rs
  - src-tauri/src/commands/conversations.rs
  - src-tauri/src/commands/chat.rs
  - src-tauri/src/lib.rs
  - src/lib/types/index.ts
  - src/lib/data/api.ts
autonomous: true
```

### Task 1: Add new DbCommand variants + update LoadMessages

<read_first>

- `src-tauri/src/db/mod.rs` (current DbCommand enum, all variants + handle_command match arms)
  </read_first>

<action>
In `src-tauri/src/db/mod.rs`:

1. Add imports for new types:

```rust
use crate::domain::conversation::{MessageBranch, SearchResult};
```

Import `row_to_message_branch` from models in the conversations handler closures.

2. Modify `DbCommand::LoadMessages` to include `branch_id`:

```rust
LoadMessages {
    conversation_id: String,
    branch_id: Option<String>,  // NEW
    reply: oneshot::Sender<anyhow::Result<Vec<Message>>>,
},
```

3. Modify `DbCommand::DeleteMessagesAfterSortOrder` to include `branch_id`:

```rust
DeleteMessagesAfterSortOrder {
    conversation_id: String,
    sort_order: i32,
    branch_id: Option<String>,  // NEW
    reply: oneshot::Sender<anyhow::Result<()>>,
},
```

4. Add new DbCommand variants:

```rust
// Branch operations
CreateBranch {
    branch: MessageBranch,
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
CountBranches {
    conversation_id: String,
    reply: oneshot::Sender<anyhow::Result<i64>>,
},
SearchMessages {
    query: String,
    reply: oneshot::Sender<anyhow::Result<Vec<SearchResult>>>,
},
```

5. Update `handle_command` match arm for `LoadMessages` to pass `branch_id.as_deref()`:

```rust
DbCommand::LoadMessages { conversation_id, branch_id, reply } => {
    let result = conversations::load_messages(conn, &conversation_id, branch_id.as_deref());
    let _ = reply.send(result);
}
```

6. Update `handle_command` match arm for `DeleteMessagesAfterSortOrder` to use branch-aware version:

```rust
DbCommand::DeleteMessagesAfterSortOrder { conversation_id, sort_order, branch_id, reply } => {
    let result = conversations::delete_messages_after_sort_order_branch(
        conn, &conversation_id, sort_order, branch_id.as_deref(),
    );
    let _ = reply.send(result);
}
```

7. Add `handle_command` match arms for new variants:

```rust
DbCommand::CreateBranch { branch, reply } => {
    let result = conversations::create_branch(conn, &branch).map(|_| branch.clone());
    let _ = reply.send(result);
}
DbCommand::LoadBranchesForMessage { parent_message_id, reply } => {
    let result = conversations::load_branches_for_message(conn, &parent_message_id);
    let _ = reply.send(result);
}
DbCommand::LoadAllBranches { conversation_id, reply } => {
    let result = conversations::load_all_branches_for_conversation(conn, &conversation_id);
    let _ = reply.send(result);
}
DbCommand::DeleteBranch { branch_id, reply } => {
    let result = conversations::delete_branch_by_id(conn, &branch_id);
    let _ = reply.send(result);
}
DbCommand::CountBranches { conversation_id, reply } => {
    let result = conversations::count_branches_for_conversation(conn, &conversation_id);
    let _ = reply.send(result);
}
DbCommand::SearchMessages { query, reply } => {
    let result = conversations::search_messages(conn, &query);
    let _ = reply.send(result);
}
```

8. Update ALL other `handle_command` match arms that call `conversations::load_messages` (in tests or elsewhere within mod.rs) to pass `None` for branch_id.
   </action>

<acceptance_criteria>

- `DbCommand::LoadMessages` has `branch_id: Option<String>` field
- `DbCommand::DeleteMessagesAfterSortOrder` has `branch_id: Option<String>` field
- `DbCommand::CreateBranch`, `LoadBranchesForMessage`, `LoadAllBranches`, `DeleteBranch`, `CountBranches`, `SearchMessages` variants exist
- `handle_command` has match arms for all new variants
- `LoadMessages` match arm passes `branch_id.as_deref()` to `conversations::load_messages`
- `cargo test -p revelry-ui` exits 0
  </acceptance_criteria>

### Task 2: Add Tauri commands + register in lib.rs

<read_first>

- `src-tauri/src/commands/conversations.rs` (current command pattern)
- `src-tauri/src/commands/chat.rs` (load_chat_context calls DbCommand::LoadMessages)
- `src-tauri/src/lib.rs` (invoke_handler registration)
  </read_first>

<action>
In `src-tauri/src/commands/conversations.rs`:

1. Add imports:

```rust
use crate::domain::conversation::{MessageBranch, SearchResult};
```

2. Update `get_messages` to accept optional `branch_id`:

```rust
#[tauri::command]
pub async fn get_messages(
    db: State<'_, DbWorker>,
    conversation_id: String,
    branch_id: Option<String>,
) -> Result<Vec<Message>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadMessages {
        conversation_id,
        branch_id,
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}
```

3. Add new Tauri commands:

```rust
#[tauri::command]
pub async fn fork_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
    parent_message_id: String,
) -> Result<MessageBranch, String> {
    // Count existing branches for auto-labeling
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::CountBranches {
        conversation_id: conversation_id.clone(),
        reply: reply_tx,
    }).map_err(|e| e.to_string())?;
    let count = reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())?;
    let label = format!("Branch {}", count + 1);

    let branch = MessageBranch::new(conversation_id, parent_message_id, label);
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::CreateBranch { branch: branch.clone(), reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_branches_for_message(
    db: State<'_, DbWorker>,
    parent_message_id: String,
) -> Result<Vec<MessageBranch>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadBranchesForMessage { parent_message_id, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_branches_for_conversation(
    db: State<'_, DbWorker>,
    conversation_id: String,
) -> Result<Vec<MessageBranch>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::LoadAllBranches { conversation_id, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_branch(
    db: State<'_, DbWorker>,
    branch_id: String,
) -> Result<(), String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::DeleteBranch { branch_id, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_messages(
    db: State<'_, DbWorker>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    let (reply_tx, reply_rx) = tokio::sync::oneshot::channel();
    db.send(DbCommand::SearchMessages { query, reply: reply_tx })
        .map_err(|e| e.to_string())?;
    reply_rx.await.map_err(|e| format!("DB worker dropped: {}", e))?.map_err(|e| e.to_string())
}
```

In `src-tauri/src/commands/chat.rs`:

1. Add `branch_id: Option<String>` parameter to `load_chat_context` and propagate it to `DbCommand::LoadMessages`. This ensures AI on branches sees branch-specific context (ISSUE 4 fix):

```rust
#[tauri::command]
pub async fn load_chat_context(
    db: State<'_, DbWorker>,
    conversation_id: String,
    branch_id: Option<String>,  // NEW — branch-aware context loading
    // ... existing params ...
) -> Result<ChatContext, String> {
    // ...
    db.send(DbCommand::LoadMessages {
        conversation_id: conversation_id.to_string(),
        branch_id,  // Propagated from frontend
        reply: reply_tx,
    })
    // ...
}
```

2. Add `branch_id: Option<String>` parameter to `start_chat` and propagate it when saving the user and AI messages. The `branch_id` is included in the `Message` objects sent to `DbCommand::SaveMessage`:

```rust
#[tauri::command]
pub async fn start_chat(
    db: State<'_, DbWorker>,
    conversation_id: String,
    content: String,
    branch_id: Option<String>,  // NEW
    // ... existing params ...
) -> Result<(), String> {
    // ...
    let user_msg = Message::new(/* ... */).with_branch_id(branch_id.clone());
    let ai_msg = Message::new(/* ... */).with_branch_id(branch_id);
    // ...
}
```

3. Add `branch_id: Option<String>` parameter to `regenerate` and propagate it to the AI placeholder message:

```rust
#[tauri::command]
pub async fn regenerate(
    db: State<'_, DbWorker>,
    conversation_id: String,
    branch_id: Option<String>,  // NEW
    // ... existing params ...
) -> Result<(), String> {
    // ...
    let ai_msg = Message::new(/* ... */).with_branch_id(branch_id);
    // ...
}
```

4. Add `branch_id: Option<String>` parameter to `edit_and_resend` and propagate it to BOTH `DbCommand::LoadMessages` and `DbCommand::DeleteMessagesAfterSortOrder`:

```rust
#[tauri::command]
pub async fn edit_and_resend(
    db: State<'_, DbWorker>,
    conversation_id: String,
    message_id: String,
    new_content: String,
    sort_order: i32,
    branch_id: Option<String>,  // NEW
    // ... existing params ...
) -> Result<(), String> {
    // ...
    db.send(DbCommand::DeleteMessagesAfterSortOrder {
        conversation_id: conversation_id.clone(),
        sort_order,
        branch_id: branch_id.clone(),  // Branch-aware truncation
        reply: reply_tx,
    })
    // ...
    db.send(DbCommand::LoadMessages {
        conversation_id: conversation_id.clone(),
        branch_id: branch_id.clone(),  // Branch-aware reload
        reply: reply_tx,
    })
    // ...
}
```

5. Add a `with_branch_id` helper method to the `Message` domain struct (in `src-tauri/src/domain/conversation.rs`) for ergonomic construction:

```rust
impl Message {
    pub fn with_branch_id(mut self, branch_id: Option<String>) -> Self {
        self.branch_id = branch_id;
        self
    }
}
```

In `src-tauri/src/lib.rs`:

1. Add new commands to `invoke_handler`:

```rust
commands::conversations::fork_conversation,
commands::conversations::get_branches_for_message,
commands::conversations::get_branches_for_conversation,
commands::conversations::delete_branch,
commands::conversations::search_messages,
```

</action>

<acceptance_criteria>

- `get_messages` command accepts `branch_id: Option<String>` parameter
- `fork_conversation`, `get_branches_for_message`, `get_branches_for_conversation`, `delete_branch`, `search_messages` Tauri commands exist
- All new commands registered in `lib.rs` `invoke_handler`
- `load_chat_context` in `chat.rs` accepts `branch_id: Option<String>` parameter and propagates it to `DbCommand::LoadMessages`
- `start_chat` in `chat.rs` accepts `branch_id: Option<String>` and passes it to saved messages
- `regenerate` in `chat.rs` accepts `branch_id: Option<String>` and passes it to AI placeholder
- `edit_and_resend` in `chat.rs` accepts `branch_id: Option<String>` and propagates it to both `DbCommand::LoadMessages` and `DbCommand::DeleteMessagesAfterSortOrder`
- `Message` domain struct has `with_branch_id()` builder method
- `cargo build` exits 0
  </acceptance_criteria>

### Task 3: Add frontend types + API wrappers

<read_first>

- `src/lib/types/index.ts` (current type definitions)
- `src/lib/data/api.ts` (current API wrapper pattern)
  </read_first>

<action>
In `src/lib/types/index.ts`:

1. Add `branch_id` field to `Message` interface (after `timestamp`):

```typescript
/** Branch this message belongs to. null = Main/root path. */
branch_id?: string | null;
```

2. Add `MessageBranch` interface (after `MessageMetadata`):

```typescript
export interface MessageBranch {
  id: string;
  conversation_id: string;
  parent_message_id: string;
  label: string;
  created_at: string;
}
```

3. Add `SearchResult` interface (after `MessageBranch`):

```typescript
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

In `src/lib/data/api.ts`:

1. Add imports for new types:

```typescript
import type { ..., MessageBranch, SearchResult } from "$lib/types";
```

2. Update `getMessages` to accept optional `branchId`:

```typescript
export async function getMessages(
  conversationId: string,
  branchId?: string | null,
): Promise<Message[]> {
  return invoke<Message[]>("get_messages", {
    conversationId,
    branchId: branchId ?? null,
  });
}
```

3. Add new API wrappers:

```typescript
export async function forkConversation(
  conversationId: string,
  parentMessageId: string,
): Promise<MessageBranch> {
  return invoke<MessageBranch>("fork_conversation", {
    conversationId,
    parentMessageId,
  });
}

export async function getBranchesForMessage(
  parentMessageId: string,
): Promise<MessageBranch[]> {
  return invoke<MessageBranch[]>("get_branches_for_message", {
    parentMessageId,
  });
}

export async function getBranchesForConversation(
  conversationId: string,
): Promise<MessageBranch[]> {
  return invoke<MessageBranch[]>("get_branches_for_conversation", {
    conversationId,
  });
}

export async function deleteBranch(branchId: string): Promise<void> {
  return invoke<void>("delete_branch", { branchId });
}

export async function searchMessages(query: string): Promise<SearchResult[]> {
  return invoke<SearchResult[]>("search_messages", { query });
}
```

</action>

<acceptance_criteria>

- `src/lib/types/index.ts` has `branch_id?: string | null` on `Message` interface
- `src/lib/types/index.ts` has `MessageBranch` interface with fields: `id, conversation_id, parent_message_id, label, created_at`
- `src/lib/types/index.ts` has `SearchResult` interface with fields: `conversation_id, conversation_title, character_id, match_type, message_snippet, message_id, branch_id, rank`
- `src/lib/data/api.ts` has `forkConversation`, `getBranchesForMessage`, `getBranchesForConversation`, `deleteBranch`, `searchMessages` functions
- `getMessages` in `api.ts` accepts optional `branchId` parameter
- `bun run check` exits 0
  </acceptance_criteria>

---

## Plan 08-03: Frontend State + Branch-Aware Loading (State Layer)

```yaml
wave: 3
depends_on: ["08-02"]
files_modified:
  - src/lib/stores/app.svelte.ts
  - src/routes/chat/[id]/+page.svelte
  - src/routes/chat/[id]/+page.ts
autonomous: true
```

### Task 1: Add branch state and actions to AppState

<read_first>

- `src/lib/stores/app.svelte.ts` (current AppState class, all state fields + actions)
- `src/lib/data/api.ts` (new API wrappers: forkConversation, getBranchesForMessage, etc.)
  </read_first>

<action>
In `src/lib/stores/app.svelte.ts`:

1. Add new imports at the top:

```typescript
import type { ..., MessageBranch, SearchResult } from "$lib/types";
```

Add these new API imports:

```typescript
import {
  // ... existing imports ...
  forkConversation as apiForkConversation,
  getBranchesForMessage as apiGetBranchesForMessage,
  getBranchesForConversation as apiGetBranchesForConversation,
  deleteBranch as apiDeleteBranch,
  searchMessages as apiSearchMessages,
  getMessages as apiGetMessages,
} from "$lib/data/api";
```

2. Add new state fields after `focusProviderSelector`:

```typescript
// ── Branch state ───────────────────────────────────────────────────────
active_branch_id = $state<string | null>(null);
branches = $state<MessageBranch[]>([]);
search_results = $state<SearchResult[]>([]);
is_searching = $state<boolean>(false);
```

3. Add new derived getter for active branch:

```typescript
get active_branch(): MessageBranch | undefined {
  if (!this.active_branch_id) return undefined;
  return this.branches.find((b) => b.id === this.active_branch_id);
}
```

4. Modify `openConversation` to accept optional `branchId` and reset branch state:

```typescript
openConversation(id: string, messages: Message[], branchId?: string | null) {
  this.active_conversation_id = id;
  this.messages = messages;
  this.active_branch_id = branchId ?? null;
  // Load branches for this conversation
  this.loadBranchesForConversation(id);
}
```

5. Modify `closeConversation` to reset branch state:

```typescript
closeConversation() {
  this.active_conversation_id = null;
  this.messages = [];
  this.context_budget = null;
  this.active_branch_id = null;
  this.branches = [];
}
```

6. Add new actions:

```typescript
/** Load all branches for the active conversation. */
async loadBranchesForConversation(conversationId: string): Promise<void> {
  try {
    this.branches = await apiGetBranchesForConversation(conversationId);
  } catch (err) {
    console.error("Failed to load branches:", err);
    this.branches = [];
  }
}

/** Switch to a different branch. Reloads messages for that branch. */
async switchBranch(branchId: string | null): Promise<void> {
  if (!this.active_conversation_id) return;
  this.active_branch_id = branchId;
  try {
    const messages = await apiGetMessages(this.active_conversation_id, branchId);
    this.messages = messages;
  } catch (err) {
    console.error("Failed to switch branch:", err);
  }
}

/** Fork from a specific message, creating a new branch. */
async forkFromMessage(messageId: string, isAiMessage: boolean): Promise<void> {
  if (!this.active_conversation_id) return;
  try {
    const branch = await apiForkConversation(this.active_conversation_id, messageId);
    this.branches = [...this.branches, branch];
    this.active_branch_id = branch.id;
    // Load messages for the new branch
    const messages = await apiGetMessages(this.active_conversation_id, branch.id);
    this.messages = messages;

    // If forking from an AI message, auto-regenerate on the new branch
    if (isAiMessage) {
      // The messages up to and including the fork point are shared.
      // We need to regenerate the AI response on this branch.
      await this.regenerate();
    }
  } catch (err) {
    console.error("Failed to create branch:", err);
    throw err;
  }
}

/** Delete a branch. If it's the active branch, switch to Main. */
async deleteBranchAction(branchId: string): Promise<void> {
  try {
    await apiDeleteBranch(branchId);
    this.branches = this.branches.filter((b) => b.id !== branchId);
    if (this.active_branch_id === branchId) {
      this.active_branch_id = null;
      // Reload root messages
      if (this.active_conversation_id) {
        const messages = await apiGetMessages(this.active_conversation_id, null);
        this.messages = messages;
      }
    }
  } catch (err) {
    console.error("Failed to delete branch:", err);
    throw err;
  }
}

/** Search across all conversations and messages via backend FTS5. */
async searchMessagesAction(query: string): Promise<void> {
  if (!query.trim()) {
    this.search_results = [];
    return;
  }
  this.is_searching = true;
  try {
    this.search_results = await apiSearchMessages(query.trim());
  } catch (err) {
    console.error("Search failed:", err);
    this.search_results = [];
  } finally {
    this.is_searching = false;
  }
}

/** Navigate to a search result's conversation and optionally switch branch. */
async navigateToSearchResult(result: SearchResult): Promise<void> {
  // Navigate to the conversation
  const messages = await apiGetMessages(result.conversation_id, result.branch_id ?? null);
  this.openConversation(result.conversation_id, messages, result.branch_id ?? null);
}
```

7. Update `startChat` method — include `branch_id` on messages AND pass it in the `invoke()` call to the Tauri backend:

```typescript
// In the invoke() call to the Tauri backend:
await invoke("start_chat", {
  conversationId: this.active_conversation_id,
  content: text,
  branchId: this.active_branch_id, // NEW — propagate to backend
  // ... existing params ...
});
```

Also set `branch_id` on local message objects:

```typescript
const userMsg: Message = {
  // ... existing fields ...
  branch_id: this.active_branch_id,
};
const aiMsg: Message = {
  // ... existing fields ...
  branch_id: this.active_branch_id,
};
```

8. Update `regenerate` method — include `branch_id` on AI placeholder AND pass it in the `invoke()` call:

```typescript
await invoke("regenerate", {
  conversationId: this.active_conversation_id,
  branchId: this.active_branch_id, // NEW — propagate to backend
  // ... existing params ...
});
```

```typescript
const aiMsg: Message = {
  // ... existing fields ...
  branch_id: this.active_branch_id,
};
```

9. Update `editResend` method — include `branch_id` on AI placeholder AND pass it in the `invoke()` call:

```typescript
await invoke("edit_and_resend", {
  conversationId: this.active_conversation_id,
  messageId,
  newContent,
  sortOrder,
  branchId: this.active_branch_id, // NEW — propagate to backend
  // ... existing params ...
});
```

```typescript
const aiMsg: Message = {
  // ... existing fields ...
  branch_id: this.active_branch_id,
};
```

</action>

<acceptance_criteria>

- `app.svelte.ts` has `active_branch_id`, `branches`, `search_results`, `is_searching` state fields
- `app.svelte.ts` has `switchBranch`, `forkFromMessage`, `deleteBranchAction`, `searchMessagesAction`, `navigateToSearchResult`, `loadBranchesForConversation` methods
- `openConversation` accepts 3rd parameter `branchId` and calls `loadBranchesForConversation`
- `startChat` includes `branch_id: this.active_branch_id` on user and AI messages, AND passes `branchId` in invoke call to `start_chat`
- `regenerate` includes `branch_id: this.active_branch_id` on AI placeholder, AND passes `branchId` in invoke call to `regenerate`
- `editResend` includes `branch_id: this.active_branch_id` on AI placeholder, AND passes `branchId` in invoke call to `edit_and_resend`
- `bun run check` exits 0
  </acceptance_criteria>

### Task 2: Update chat page for branch-aware loading

<read_first>

- `src/routes/chat/[id]/+page.svelte` (current message loading + MessageBubble rendering)
- `src/routes/chat/[id]/+page.ts` (current load function)
  </read_first>

<action>
In `src/routes/chat/[id]/+page.svelte`:

1. Update the `$effect` that hydrates conversation to use the new `openConversation` signature:

```typescript
$effect(() => {
  apiGetMessages(data.id)
    .then((msgs) => {
      app.openConversation(data.id, msgs);
    })
    .catch((err) => {
      console.error("Failed to load messages:", err);
      app.openConversation(data.id, []);
    });
});
```

This now calls `openConversation(id, msgs)` which internally loads branches.

2. Import `BranchBadge` component (will be created in Plan 08-04 — for now just verify the import path):

```typescript
import BranchBadge from "$lib/components/chat/BranchBadge.svelte";
```

Note: This component doesn't exist yet. We'll add it in Plan 08-04. For now, skip the import and rendering — just ensure the state is wired.

In `src/routes/chat/[id]/+page.ts`:
No changes needed — it just returns `{ id: params.id }`.
</action>

<acceptance_criteria>

- `+page.svelte` calls `app.openConversation(data.id, msgs)` which triggers branch loading
- `bun run check` exits 0
  </acceptance_criteria>

---

## Plan 08-04: UI Components (Visual Layer)

```yaml
wave: 4
depends_on: ["08-03"]
files_modified:
  - src/lib/components/chat/BranchBadge.svelte (NEW)
  - src/lib/components/chat/BranchPopover.svelte (NEW)
  - src/lib/components/chat/MessageBubble.svelte
  - src/lib/components/layout/ConversationSidebar.svelte
  - src/routes/chat/[id]/+page.svelte
  - src/lib/components/ui/index.ts
autonomous: true
```

### Task 1: Create BranchBadge component

<read_first>

- `src/lib/components/chat/MessageBubble.svelte` (badge positioning context)
- 08-UI-SPEC.md (Branch Indicator Badge interaction contract)
  </read_first>

<action>
Create `src/lib/components/chat/BranchBadge.svelte`:

```svelte
<script lang="ts">
  import type { MessageBranch } from '$lib/types';
  import { cn } from '$lib/utils';

  interface Props {
    branches: MessageBranch[];
    activeBranchId: string | null;
    class?: string;
    onclick?: () => void;
  }

  let { branches, activeBranchId, class: className, onclick }: Props = $props();

  // Calculate position: 1-based index of active branch among all branches at this fork point
  // "Main" is included as position when branch_id is null
  let totalBranches = $derived(branches.length + 1); // +1 for Main
  let activeIndex = $derived(
    activeBranchId
      ? branches.findIndex((b) => b.id === activeBranchId) + 2 // +2 because Main=1, branches start at 2
      : 1 // Main is position 1
  );

  let label = $derived(`${activeIndex}/${totalBranches}`);
</script>

<button
  type="button"
  class={cn(
    'text-[10px] font-medium px-1.5 py-0.5 rounded-(--radius-sm)',
    'border border-(--color-accent)/30 text-(--color-accent) bg-(--color-accent)/10',
    'hover:bg-(--color-accent)/20 transition-colors cursor-pointer',
    className,
  )}
  aria-label="View branches ({activeIndex} of {totalBranches})"
  {onclick}
>
  {label}
</button>
```

</action>

<acceptance_criteria>

- `src/lib/components/chat/BranchBadge.svelte` exists
- Component renders a pill badge with format "N/M"
- Has `onclick` prop for opening popover
- Uses `text-(--color-accent)` and `bg-(--color-accent)/10` styling per UI-SPEC
- `bun run check` exits 0
  </acceptance_criteria>

### Task 2: Create BranchPopover component

<read_first>

- `src/lib/components/chat/ChatHeader.svelte` (PersonaSwitcher pattern — bits-ui Popover usage)
- `08-UI-SPEC.md` (Branch Popover interaction contract)
  </read_first>

<action>
Create `src/lib/components/chat/BranchPopover.svelte`:

```svelte
<script lang="ts">
  import type { MessageBranch, Message } from '$lib/types';
  import { Popover } from 'bits-ui';
  import { Check, Trash2 } from '@lucide/svelte';
  import { cn } from '$lib/utils';
  import { app } from '$lib/stores/app.svelte';

  interface Props {
    branches: MessageBranch[];
    activeBranchId: string | null;
    messages: Message[];
    open: boolean;
    onopenchange: (open: boolean) => void;
    children?: Snippet;
  }

  let { branches, activeBranchId, messages, open, onopenchange, children }: Props = $props();

  let deletingBranchId = $state<string | null>(null);

  function getPreviewSnippet(branch: MessageBranch): string {
    const branchMsg = messages.find((m) => m.branch_id === branch.id);
    if (branchMsg) {
      return branchMsg.content.slice(0, 50) + (branchMsg.content.length > 50 ? '...' : '');
    }
    return 'No messages yet';
  }

  function getMainPreview(): string {
    const mainMsg = messages.find((m) => !m.branch_id);
    if (mainMsg) {
      return mainMsg.content.slice(0, 50) + (mainMsg.content.length > 50 ? '...' : '');
    }
    return 'No messages';
  }

  function handleSelect(branchId: string | null) {
    app.switchBranch(branchId);
    onopenchange(false);
  }

  async function handleDelete(branchId: string) {
    if (deletingBranchId) return; // Already confirming
    deletingBranchId = branchId;
  }

  async function confirmDelete(branchId: string) {
    try {
      await app.deleteBranchAction(branchId);
      deletingBranchId = null;
      onopenchange(false);
    } catch (err) {
      console.error('Failed to delete branch:', err);
      deletingBranchId = null;
    }
  }

  function cancelDelete() {
    deletingBranchId = null;
  }
</script>

<Popover.Root bind:open onOpenChange={onopenchange}>
  <Popover.Trigger>
    {@render children?.()}
  </Popover.Trigger>
  <Popover.Portal>
    <Popover.Content
      class="w-72 rounded-(--radius-lg) border border-(--color-border) bg-(--color-surface-elevated) p-1 shadow-xl z-50"
      sideOffset={4}
      align="start"
    >
      <!-- Header -->
      <div class="px-2 pb-1.5 pt-1 text-[10px] font-semibold uppercase tracking-wider text-(--color-subtle)">
        Branches
      </div>

      <!-- Main branch -->
      <button
        type="button"
        onclick={() => handleSelect(null)}
        class={cn(
          'group flex w-full items-center gap-2 rounded-(--radius-md) px-2 py-1.5 text-left',
          activeBranchId === null
            ? 'bg-(--color-surface) border-l-2 border-(--color-accent)'
            : 'hover:bg-(--color-surface-hover)',
        )}
      >
        <div class="flex-1 min-w-0">
          <div class="text-sm font-medium">Main</div>
          <div class="text-[11px] text-(--color-muted) truncate">{getMainPreview()}</div>
        </div>
        {#if activeBranchId === null}
          <Check class="size-3.5 shrink-0 text-(--color-accent)" />
        {/if}
      </button>

      <!-- Other branches -->
      {#each branches as branch (branch.id)}
        {#if deletingBranchId === branch.id}
          <!-- Inline delete confirmation -->
          <div class="rounded-(--radius-md) px-2 py-1.5">
            <p class="text-xs text-(--color-foreground)">Delete this branch?</p>
            <p class="text-[10px] text-(--color-muted)">This cannot be undone.</p>
            <div class="mt-1.5 flex gap-2">
              <button
                type="button"
                onclick={() => confirmDelete(branch.id)}
                class="text-xs font-medium text-(--color-destructive) hover:underline"
              >
                Delete
              </button>
              <button
                type="button"
                onclick={cancelDelete}
                class="text-xs font-medium text-(--color-subtle) hover:text-(--color-foreground)"
              >
                Cancel
              </button>
            </div>
          </div>
        {:else}
          <button
            type="button"
            onclick={() => handleSelect(branch.id)}
            class={cn(
              'group flex w-full items-center gap-2 rounded-(--radius-md) px-2 py-1.5 text-left',
              activeBranchId === branch.id
                ? 'bg-(--color-surface) border-l-2 border-(--color-accent)'
                : 'hover:bg-(--color-surface-hover)',
            )}
          >
            <div class="flex-1 min-w-0">
              <div class="text-sm font-medium">{branch.label}</div>
              <div class="text-[11px] text-(--color-muted) truncate">{getPreviewSnippet(branch)}</div>
            </div>
            {#if activeBranchId === branch.id}
              <Check class="size-3.5 shrink-0 text-(--color-accent)" />
            {:else}
              <button
                type="button"
                onclick={(e) => { e.stopPropagation(); handleDelete(branch.id); }}
                class="size-5 shrink-0 inline-flex items-center justify-center rounded-(--radius-sm) text-(--color-subtle) opacity-0 group-hover:opacity-100 hover:text-(--color-destructive) hover:bg-(--color-surface-hover)"
                aria-label="Delete branch"
              >
                <Trash2 class="size-3" />
              </button>
            {/if}
          </button>
        {/if}
      {/each}
    </Popover.Content>
  </Popover.Portal>
</Popover.Root>
```

Note: The BranchPopover uses `children?: Snippet` + `{@render children?.()}` per Svelte 5 convention (no `<slot />`). The `onclick` handler uses `(e) => { e.stopPropagation(); ... }` pattern (not `onclick|stopPropagation` which is Svelte 4 syntax).
</action>

<acceptance_criteria>

- `src/lib/components/chat/BranchPopover.svelte` exists
- Uses bits-ui `Popover` component
- Shows "Main" branch + all sibling branches
- Active branch has Check icon and accent left border
- Non-active branches show Trash2 icon on hover
- Inline delete confirmation with "Delete" / "Cancel" buttons
- Clicking a branch calls `app.switchBranch(branchId)`
- `bun run check` exits 0
  </acceptance_criteria>

### Task 3: Update MessageBubble with branch actions + badge

<read_first>

- `src/lib/components/chat/MessageBubble.svelte` (current hover actions row + bubble rendering)
- `08-UI-SPEC.md` (Branch from here action + Branch badge interaction contracts)
  </read_first>

<action>
In `src/lib/components/chat/MessageBubble.svelte`:

1. Add new imports:

```typescript
import { GitBranch } from "@lucide/svelte";
import BranchBadge from "./BranchBadge.svelte";
import BranchPopover from "./BranchPopover.svelte";
```

2. Extend `Props` interface to include branch data:

```typescript
interface Props {
  message: Message;
  character: CharacterCard | undefined;
  persona: UserPersona | undefined;
  isLastAiMessage?: boolean;
  branchesAtMessage?: MessageBranch[];
}
```

3. Destructure new prop:

```typescript
let {
  message,
  character,
  persona,
  isLastAiMessage = false,
  branchesAtMessage = [],
}: Props = $props();
```

4. Add state for popover:

```typescript
let branchPopoverOpen = $state(false);
```

5. Add "Branch from here" button in the hover actions row, after Copy and before Edit:

```svelte
<!-- Branch from here: on all messages -->
<button
  type="button"
  onclick={() => app.forkFromMessage(message.id, !isUser)}
  class="inline-flex size-6 items-center justify-center rounded-(--radius-sm) text-(--color-subtle) hover:bg-(--color-surface-hover) hover:text-(--color-foreground)"
  aria-label="Branch from here"
>
  <GitBranch class="size-3" />
</button>
```

6. Add BranchBadge + BranchPopover on messages that have sibling branches. This goes just before the bubble div, positioned to the left of AI bubbles or right of user bubbles. Add it inside the `<!-- Bubble + meta -->` section, between the speaker name and the bubble:

```svelte
{#if branchesAtMessage.length > 0}
  <div class={cn('flex items-center gap-1 px-1', isUser ? 'flex-row-reverse' : 'flex-row')}>
    <BranchPopover
      branches={branchesAtMessage}
      activeBranchId={app.active_branch_id}
      messages={app.messages}
      bind:open={branchPopoverOpen}
    >
      <BranchBadge
        branches={branchesAtMessage}
        activeBranchId={app.active_branch_id}
      />
    </BranchPopover>
  </div>
{/if}
```

7. Import `MessageBranch` type at the top:

```typescript
import type {
  Message,
  CharacterCard,
  UserPersona,
  MessageBranch,
} from "$lib/types";
```

</action>

<acceptance_criteria>

- `MessageBubble.svelte` imports `GitBranch` from `@lucide/svelte`
- `MessageBubble.svelte` imports `BranchBadge` and `BranchPopover`
- `Props` interface includes `branchesAtMessage?: MessageBranch[]`
- Hover actions row has a GitBranch icon button with `aria-label="Branch from here"`
- Clicking GitBranch calls `app.forkFromMessage(message.id, !isUser)`
- Messages with `branchesAtMessage.length > 0` show BranchBadge + BranchPopover
- `bun run check` exits 0
  </acceptance_criteria>

### Task 4: Update chat page to pass branch data to MessageBubble

<read_first>

- `src/routes/chat/[id]/+page.svelte` (current message rendering loop)
  </read_first>

<action>
In `src/routes/chat/[id]/+page.svelte`:

1. Compute which messages have branches by checking `app.branches`:
   Add a derived value that maps parent_message_id to branches:

```typescript
let branchesByParentId = $derived(
  (() => {
    const map = new Map<string, MessageBranch[]>();
    for (const branch of app.branches) {
      const existing = map.get(branch.parent_message_id) ?? [];
      existing.push(branch);
      map.set(branch.parent_message_id, existing);
    }
    return map;
  })(),
);
```

2. Pass `branchesAtMessage` to each `MessageBubble` in the `{#each}` loop. Also add a `data-message-id` attribute wrapper for scroll-to-message support (needed by search navigation in Task 5):

```svelte
{#each app.messages as message (message.id)}
  <div id="message-{message.id}" data-message-id={message.id}>
    <MessageBubble
      {message}
      character={app.active_character}
      persona={app.active_persona}
      isLastAiMessage={message.id === lastAiMessageId}
      branchesAtMessage={branchesByParentId.get(message.id) ?? []}
    />
  </div>
{/each}
```

3. Import `MessageBranch` type at top:

```typescript
import type { MessageBranch } from "$lib/types";
```

(Actually, this import may not be needed directly since it's inferred from the Map type.)
</action>

<acceptance_criteria>

- `+page.svelte` computes `branchesByParentId` from `app.branches`
- Each `MessageBubble` receives `branchesAtMessage` prop
- `bun run check` exits 0
  </acceptance_criteria>

### Task 5: Update ConversationSidebar with backend FTS5 search

<read_first>

- `src/lib/components/layout/ConversationSidebar.svelte` (current client-side search)
- `08-UI-SPEC.md` (Enhanced Sidebar Search interaction contract)
  </read_first>

<action>
In `src/lib/components/layout/ConversationSidebar.svelte`:

1. Add debounce utility and new imports:

```typescript
import { app } from "$lib/stores/app.svelte";
import type { SearchResult } from "$lib/types";
```

2. Replace the `filtered` derived with a dual-mode approach:

```typescript
let search = $state("");
let searchDebounceTimer: ReturnType<typeof setTimeout> | null = null;

// When search is empty, show all conversations (no backend call needed)
// When search has text, results come from app.search_results (populated by backend FTS5)
let filtered = $derived(
  (() => {
    const q = search.trim();
    if (!q)
      return {
        conversations: app.conversations,
        searchMode: false,
        results: [] as SearchResult[],
      };
    return { conversations: [], searchMode: true, results: app.search_results };
  })(),
);
```

3. Add a derived `groupedSearchResults` and reactive effect to trigger search with debounce:

```typescript
// Computed as $derived to avoid template mutation anti-pattern (ISSUE 8 fix)
let groupedSearchResults = $derived(
  (() => {
    const map = new Map<string, SearchResult[]>();
    for (const result of filtered.results) {
      const existing = map.get(result.conversation_id) ?? [];
      existing.push(result);
      map.set(result.conversation_id, existing);
    }
    return map;
  })(),
);

$effect(() => {
  const q = search.trim();
  if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
  if (!q) {
    app.search_results = [];
    return;
  }
  searchDebounceTimer = setTimeout(() => {
    app.searchMessagesAction(q);
  }, 300);
  return () => {
    if (searchDebounceTimer) clearTimeout(searchDebounceTimer);
  };
});
```

4. Update the conversation list rendering to handle both modes:

When `searchMode` is false (no search query): render the existing conversation list (unchanged).

When `searchMode` is true (search active): render search results grouped by conversation:

```svelte
{:else}
  {#if filtered.searchMode}
    {#if app.is_searching}
      <div class="flex h-full items-center justify-center">
        <p class="text-xs text-(--color-muted)">Searching...</p>
      </div>
    {:else if filtered.results.length === 0}
      <div class="flex h-full flex-col items-center justify-center gap-2 px-4 text-center">
        <MessageSquareDashed class="size-6 text-(--color-subtle)" />
        <p class="text-xs text-(--color-muted)">No conversations match.</p>
      </div>
    {:else}
      <ul class="flex flex-col gap-0.5">
        {#each groupedSearchResults as [convId, results] (convId)}
          {@const firstResult = results[0]}
          {@const char = characterFor(firstResult.character_id)}
          {@const active = convId === app.active_conversation_id}
          <li class="group/conv relative">
            <button
              type="button"
              onclick={() => {
                app.navigateToSearchResult(firstResult);
                search = '';
                // Scroll to matching message after navigation (ISSUE 6 fix)
                // Use requestAnimationFrame to wait for DOM update after navigation
                requestAnimationFrame(() => {
                  if (firstResult.message_id) {
                    const el = document.getElementById(`message-${firstResult.message_id}`);
                    el?.scrollIntoView({ behavior: 'smooth', block: 'center' });
                  }
                });
              }}
              class={cn(
                'group flex w-full gap-2.5 rounded-(--radius-md) px-2 py-2 text-left transition-colors',
                active ? 'bg-(--color-surface-elevated)' : 'hover:bg-(--color-surface-hover)'
              )}
            >
              <Avatar name={char?.name ?? '?'} src={char?.avatar_path} size="sm" class="mt-0.5" />
              <div class="min-w-0 flex-1">
                <div class="flex items-baseline justify-between gap-2">
                  <span class={cn('truncate text-xs font-medium', active ? 'text-(--color-foreground)' : 'text-(--color-foreground)/90')}>
                    {firstResult.conversation_title}
                  </span>
                </div>
                {#each results.slice(0, 1) as r}
                  {#if r.message_snippet}
                    <p class="mt-0.5 line-clamp-2 text-[11px] leading-tight text-(--color-muted)">
                      {@html r.message_snippet}
                    </p>
                  {/if}
                {/each}
              </div>
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  {:else}
    <!-- Original conversation list (unchanged) -->
    <ul class="flex flex-col gap-0.5">
      ...
    </ul>
  {/if}
```

5. The `<mark>` tag styling for search highlights should be added inline or via global CSS. Add a `<style>` block or use Tailwind arbitrary:
   Since FTS5 returns `<mark>` tags in the snippet, add a global style rule in `src/app.css`:

```css
mark {
  background: oklch(from var(--color-accent) l c h / 0.2);
  color: var(--color-accent);
  border-radius: 2px;
  padding: 0 2px;
}
```

Or alternatively, add it as a scoped style in ConversationSidebar. Prefer the `src/app.css` approach since it's a global utility.
</action>

<acceptance_criteria>

- `ConversationSidebar.svelte` uses debounced backend search (300ms) via `app.searchMessagesAction`
- Empty search shows all conversations (client-side, no backend call)
- Non-empty search shows FTS5 results grouped by conversation
- Message snippets with `<mark>` tags rendered as HTML
- Clicking a search result calls `app.navigateToSearchResult`, clears search input, and scrolls to matching message
- `groupedSearchResults` is computed as `$derived` (not mutated in template)
- `src/app.css` contains a `mark` style rule with accent background at 20% opacity
- `bun run check` exits 0
  </acceptance_criteria>

### Task 6: Add barrel exports + final verification

<read_first>

- `src/lib/components/ui/index.ts` (barrel exports)
  </read_first>

<action>
1. In `src/lib/components/ui/index.ts`: No changes needed — BranchBadge and BranchPopover are chat components, not UI primitives.

2. Run `bun run check` to verify no type errors.

3. Run `cargo test -p revelry-ui` to verify all Rust tests pass.

4. Run `cargo build` to verify the binary compiles.
   </action>

<acceptance_criteria>

- `bun run check` exits 0
- `cargo test -p revelry-ui` exits 0
- `cargo build` exits 0
  </acceptance_criteria>

---

## Verification

After all plans complete, verify:

1. **CONV-06**: User can fork a conversation from any message → GitBranch icon on every MessageBubble, calling `forkFromMessage` creates a branch in the DB and switches the view
2. **CONV-07**: User can navigate between branches → BranchBadge on forked messages, BranchPopover lists all branches, clicking switches the view
3. **CONV-08**: User can search across all conversations and messages → Sidebar search uses backend FTS5, results show message snippets with highlights

## must_haves

1. `message_branches` table exists in SQLite with correct schema (id, conversation_id, parent_message_id, label, created_at)
2. `messages` table has `branch_id` column (nullable FK to `message_branches`)
3. `messages_fts` FTS5 virtual table exists with sync triggers
4. `load_messages` supports branch-aware filtering (NULL = root, specific branch_id = root + branch)
5. `search_messages` returns FTS5 results with `<mark>` highlighted snippets
6. Frontend `Message` type includes `branch_id` field
7. `MessageBranch` and `SearchResult` types exist in both Rust and TypeScript
8. AppState tracks `active_branch_id`, `branches`, and `search_results`
9. BranchBadge shows "N/M" position on forked messages
10. BranchPopover lists all branches with preview, active indicator, and delete action
11. "Branch from here" action (GitBranch icon) on every message
12. Sidebar search uses backend FTS5 with debounced input and message preview snippets
13. All existing tests pass (`cargo test` + `bun run check`)
