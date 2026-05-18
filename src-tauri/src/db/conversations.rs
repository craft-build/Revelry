use crate::db::characters::{load_character, sanitize_fts_query};
use crate::db::models::{row_to_conversation, row_to_message, row_to_message_branch};
use crate::domain::conversation::{
    Conversation, Message, MessageBranch, MessageRole, SearchResult,
};
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};

/// Starts a new conversation with a character.
/// Loads the character, creates a Conversation with title="Chat with {char.name}",
/// sets last_message_preview to the character's first_mes, inserts the conversation,
/// and creates an initial assistant message if first_mes is non-empty.
pub fn start_conversation(conn: &Connection, character_id: &str) -> Result<Conversation> {
    let character = load_character(conn, character_id)?
        .ok_or_else(|| anyhow::anyhow!("Character not found: {}", character_id))?;

    let mut conversation = Conversation::new(character_id.to_string());
    conversation.title = format!("Chat with {}", character.name);
    conversation.last_message_preview = character.first_mes.clone();

    // Insert the conversation
    conn.prepare_cached(
        "INSERT INTO conversations (id, character_id, provider_id, model_name, creator_persona_id,
         generation_preset_id, title, last_message_preview, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
    )?
    .execute(params![
        &conversation.id,
        &conversation.character_id,
        &conversation.provider_id,
        &conversation.model_name,
        &conversation.creator_persona_id,
        &conversation.generation_preset_id,
        &conversation.title,
        &conversation.last_message_preview,
        conversation.created_at.to_rfc3339(),
        conversation.updated_at.to_rfc3339(),
    ])?;

    // Create initial assistant message from character's first_mes (if not empty)
    if !character.first_mes.is_empty() {
        let initial_message = Message::with_order(
            conversation.id.clone(),
            MessageRole::Assistant,
            character.first_mes.clone(),
            0,
        );
        save_message(conn, &initial_message)?;
    }

    Ok(conversation)
}

/// Loads all conversations, ordered by updated_at descending (most recent first).
pub fn load_conversations(conn: &Connection) -> Result<Vec<Conversation>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, provider_id, model_name, creator_persona_id, generation_preset_id,
         title, last_message_preview, created_at, updated_at
         FROM conversations ORDER BY updated_at DESC",
    )?;
    let conversations = stmt.query_map([], |row| row_to_conversation(row))?;
    let mut result = Vec::new();
    for conv in conversations {
        result.push(conv?);
    }
    Ok(result)
}

/// Deletes a conversation and all its messages (cascades via foreign key).
pub fn delete_conversation(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM conversations WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

/// Loads messages for a conversation, ordered by sort_order ascending.
/// Phase 8: branch-aware filtering.
/// When branch_id is None: loads root messages (branch_id IS NULL).
/// When branch_id is Some: loads root messages + messages in that branch.
pub fn load_messages(
    conn: &Connection,
    conversation_id: &str,
    branch_id: Option<&str>,
) -> Result<Vec<Message>> {
    let mut result = Vec::new();
    match branch_id {
        Some(bid) => {
            let mut stmt = conn.prepare_cached(
                "SELECT id, conversation_id, role, content, sort_order, timestamp, branch_id
                 FROM messages WHERE conversation_id = ?1 AND (branch_id IS NULL OR branch_id = ?2) ORDER BY sort_order ASC",
            )?;
            for msg in stmt.query_map(params![conversation_id, bid], |row| row_to_message(row))? {
                result.push(msg?);
            }
        }
        None => {
            let mut stmt = conn.prepare_cached(
                "SELECT id, conversation_id, role, content, sort_order, timestamp, branch_id
                 FROM messages WHERE conversation_id = ?1 AND branch_id IS NULL ORDER BY sort_order ASC",
            )?;
            for msg in stmt.query_map(params![conversation_id], |row| row_to_message(row))? {
                result.push(msg?);
            }
        }
    }
    Ok(result)
}

/// Saves a new message to the database.
/// Also updates the conversation's updated_at timestamp and last_message_preview.
/// Phase 8: includes branch_id column.
pub fn save_message(conn: &Connection, msg: &Message) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO messages (id, conversation_id, role, content, sort_order, timestamp, branch_id)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
    )?
    .execute(params![
        &msg.id,
        &msg.conversation_id,
        msg.role.to_string(),
        &msg.content,
        msg.sort_order,
        msg.timestamp.to_rfc3339(),
        &msg.branch_id,
    ])?;

    // Update conversation's updated_at and last_message_preview
    let now = Utc::now();
    let preview: String = msg.content.chars().take(100).collect();
    conn.prepare_cached(
        "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3",
    )?
    .execute(params![now.to_rfc3339(), preview, &msg.conversation_id])?;

    Ok(())
}

/// Deletes a single message by ID.
/// Used by regenerate to delete the last AI message before re-streaming.
pub fn delete_message(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM messages WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

/// Deletes all messages with sort_order greater than the given value.
/// Used by edit/resend to truncate the conversation from the edit point.
/// Also updates the conversation's last_message_preview to the latest remaining message.
pub fn delete_messages_after_sort_order(
    conn: &Connection,
    conversation_id: &str,
    sort_order: i32,
) -> Result<()> {
    conn.prepare_cached("DELETE FROM messages WHERE conversation_id = ?1 AND sort_order > ?2")?
        .execute(params![conversation_id, sort_order])?;

    // Update conversation's last_message_preview to the latest remaining message
    let now = Utc::now();
    let preview: Option<String> = conn
        .prepare_cached(
            "SELECT content FROM messages WHERE conversation_id = ?1 ORDER BY sort_order DESC LIMIT 1",
        )?
        .query_row(params![conversation_id], |row| row.get(0))
        .ok();

    conn.prepare_cached(
        "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3",
    )?
    .execute(params![
        now.to_rfc3339(),
        preview.unwrap_or_default(),
        conversation_id
    ])?;

    Ok(())
}

/// Updates a message's content by ID.
/// Used by edit/resend to update the edited message before re-streaming.
pub fn update_message_content(conn: &Connection, id: &str, content: &str) -> Result<()> {
    conn.prepare_cached("UPDATE messages SET content = ?1 WHERE id = ?2")?
        .execute(params![content, id])?;
    Ok(())
}

/// Searches conversations using SQL LIKE on title and last_message_preview.
pub fn search_conversations(conn: &Connection, query: &str) -> Result<Vec<Conversation>> {
    let pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, provider_id, model_name, creator_persona_id, generation_preset_id,
         title, last_message_preview, created_at, updated_at
         FROM conversations
         WHERE title LIKE ?1 ESCAPE '\\' OR last_message_preview LIKE ?1 ESCAPE '\\'
         ORDER BY updated_at DESC",
    )?;
    let conversations = stmt.query_map(params![pattern], |row| row_to_conversation(row))?;
    let mut result = Vec::new();
    for conv in conversations {
        result.push(conv?);
    }
    Ok(result)
}

/// Updates the creator_persona_id and updated_at for a conversation.
pub fn update_conversation_persona(
    conn: &Connection,
    conversation_id: &str,
    persona_id: Option<&str>,
) -> Result<()> {
    let now = Utc::now();
    conn.prepare_cached(
        "UPDATE conversations SET creator_persona_id = ?1, updated_at = ?2 WHERE id = ?3",
    )?
    .execute(params![persona_id, now.to_rfc3339(), conversation_id])?;
    Ok(())
}

/// Updates the provider_id, model_name, and updated_at for a conversation.
/// This is used by the ChatHeader provider/model selector (Plan 02 Task 3).
pub fn update_conversation_provider(
    conn: &Connection,
    conversation_id: &str,
    provider_id: Option<&str>,
    model_name: Option<&str>,
) -> Result<()> {
    let now = Utc::now();
    conn.prepare_cached(
        "UPDATE conversations SET provider_id = ?1, model_name = ?2, updated_at = ?3 WHERE id = ?4",
    )?
    .execute(params![
        provider_id,
        model_name,
        now.to_rfc3339(),
        conversation_id
    ])?;
    Ok(())
}

/// Creates a new branch in the database.
pub fn create_branch(conn: &Connection, branch: &MessageBranch) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO message_branches (id, conversation_id, parent_message_id, label, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
    )?
    .execute(params![
        &branch.id,
        &branch.conversation_id,
        &branch.parent_message_id,
        &branch.label,
        branch.created_at.to_rfc3339(),
    ])?;
    Ok(())
}

/// Loads all branches that fork from a specific parent message.
pub fn load_branches_for_message(
    conn: &Connection,
    parent_message_id: &str,
) -> Result<Vec<MessageBranch>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, parent_message_id, label, created_at
         FROM message_branches WHERE parent_message_id = ?1 ORDER BY created_at ASC",
    )?;
    let mut result = Vec::new();
    for b in stmt.query_map(params![parent_message_id], |row| row_to_message_branch(row))? {
        result.push(b?);
    }
    Ok(result)
}

/// Loads all branches for a conversation.
pub fn load_all_branches_for_conversation(
    conn: &Connection,
    conversation_id: &str,
) -> Result<Vec<MessageBranch>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, parent_message_id, label, created_at
         FROM message_branches WHERE conversation_id = ?1 ORDER BY created_at ASC",
    )?;
    let mut result = Vec::new();
    for b in stmt.query_map(params![conversation_id], |row| row_to_message_branch(row))? {
        result.push(b?);
    }
    Ok(result)
}

/// Deletes a branch by ID. Associated messages are cascaded via FK.
pub fn delete_branch_by_id(conn: &Connection, branch_id: &str) -> Result<()> {
    // Get conversation_id BEFORE deleting (needed for preview update)
    let conv_id: Option<String> = conn
        .prepare_cached("SELECT conversation_id FROM message_branches WHERE id = ?1")?
        .query_row(params![branch_id], |row| row.get(0))
        .ok();

    conn.prepare_cached("DELETE FROM message_branches WHERE id = ?1")?
        .execute(params![branch_id])?;

    if let Some(cid) = conv_id {
        let now = Utc::now();
        let preview: Option<String> = conn
            .prepare_cached(
                "SELECT content FROM messages WHERE conversation_id = ?1 ORDER BY sort_order DESC LIMIT 1",
            )?
            .query_row(params![cid], |row| row.get(0))
            .ok();
        conn.prepare_cached(
            "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3",
        )?
        .execute(params![now.to_rfc3339(), preview.unwrap_or_default(), cid])?;
    }
    Ok(())
}

/// Counts the number of branches for a conversation (for auto-labeling).
pub fn count_branches_for_conversation(conn: &Connection, conversation_id: &str) -> Result<i64> {
    let count: i64 = conn
        .prepare_cached("SELECT COUNT(*) FROM message_branches WHERE conversation_id = ?1")?
        .query_row(params![conversation_id], |row| row.get(0))?;
    Ok(count)
}

/// Searches across all messages (via FTS5) and conversation titles (via LIKE).
/// Returns deduplicated results grouped by conversation.
pub fn search_messages(conn: &Connection, query: &str) -> Result<Vec<SearchResult>> {
    let sanitized = sanitize_fts_query(query);
    if sanitized.is_empty() {
        return Ok(Vec::new());
    }

    let mut result = Vec::new();

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
         LIMIT 50",
    )?;
    for r in msg_stmt.query_map(params![sanitized], |row| {
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
    })? {
        result.push(r?);
    }

    // Also search conversation titles via LIKE
    let like_pattern = format!("%{}%", query.replace('%', "\\%").replace('_', "\\_"));
    let mut title_stmt = conn.prepare_cached(
        "SELECT id as conversation_id, title as conversation_title,
                character_id, 'title' as match_type,
                NULL as message_snippet, NULL as message_id,
                NULL as branch_id, 0.0 as rank
         FROM conversations
         WHERE title LIKE ?1
         LIMIT 20",
    )?;
    for r in title_stmt.query_map(params![like_pattern], |row| {
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
    })? {
        result.push(r?);
    }

    // Deduplicate: if a conversation already matched via message FTS, skip the title match
    let mut seen_conv_ids = std::collections::HashSet::new();
    result.retain(|r| seen_conv_ids.insert(r.conversation_id.clone()));

    Ok(result)
}

/// Branch-aware version of delete_messages_after_sort_order.
/// Only deletes messages matching the given branch_id (or NULL for root).
pub fn delete_messages_after_sort_order_branch(
    conn: &Connection,
    conversation_id: &str,
    sort_order: i32,
    branch_id: Option<&str>,
) -> Result<()> {
    match branch_id {
        Some(bid) => {
            conn.prepare_cached(
                "DELETE FROM messages WHERE conversation_id = ?1 AND sort_order > ?2 AND branch_id = ?3",
            )?
            .execute(params![conversation_id, sort_order, bid])?;
        }
        None => {
            conn.prepare_cached(
                "DELETE FROM messages WHERE conversation_id = ?1 AND sort_order > ?2 AND branch_id IS NULL",
            )?
            .execute(params![conversation_id, sort_order])?;
        }
    }

    // Update conversation preview
    let now = Utc::now();
    let preview: Option<String> = conn
        .prepare_cached(
            "SELECT content FROM messages WHERE conversation_id = ?1 ORDER BY sort_order DESC LIMIT 1",
        )?
        .query_row(params![conversation_id], |row| row.get(0))
        .ok();
    conn.prepare_cached(
        "UPDATE conversations SET updated_at = ?1, last_message_preview = ?2 WHERE id = ?3",
    )?
    .execute(params![
        now.to_rfc3339(),
        preview.unwrap_or_default(),
        conversation_id
    ])?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::characters::insert_character;
    use crate::db::migrations::run_migrations;
    use crate::domain::character::CharacterCardV2;
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    fn make_test_character(name: &str) -> CharacterCardV2 {
        let mut character = CharacterCardV2::new(name.to_string());
        character.first_mes = format!("Hello, I am {}!", name);
        character
    }

    #[test]
    fn test_start_conversation() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        assert_eq!(
            conversation.title, "Chat with Aria",
            "Title should be auto-generated"
        );
        assert_eq!(
            conversation.last_message_preview, "Hello, I am Aria!",
            "Preview should be first_mes"
        );
        assert_eq!(conversation.character_id, character.id);

        // Should have created an initial assistant message
        let messages =
            load_messages(&conn, &conversation.id, None).expect("Load messages should succeed");
        assert_eq!(messages.len(), 1, "Should have 1 initial message");
        assert_eq!(messages[0].role, MessageRole::Assistant);
        assert_eq!(messages[0].content, "Hello, I am Aria!");
        assert_eq!(messages[0].sort_order, 0);
    }

    #[test]
    fn test_start_conversation_no_first_mes() {
        let conn = setup_test_db();
        let mut character = CharacterCardV2::new("Silent".to_string());
        character.first_mes = String::new();
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        assert_eq!(conversation.title, "Chat with Silent");
        assert!(conversation.last_message_preview.is_empty());

        // No initial message when first_mes is empty
        let messages =
            load_messages(&conn, &conversation.id, None).expect("Load messages should succeed");
        assert!(
            messages.is_empty(),
            "Should have no messages for empty first_mes"
        );
    }

    #[test]
    fn test_start_conversation_character_not_found() {
        let conn = setup_test_db();
        let result = start_conversation(&conn, "nonexistent-id");
        assert!(result.is_err(), "Should error when character not found");
    }

    #[test]
    fn test_load_conversations_ordered() {
        let conn = setup_test_db();
        let c1 = make_test_character("Aria");
        let c2 = make_test_character("Bob");
        insert_character(&conn, &c1).expect("Insert c1");
        insert_character(&conn, &c2).expect("Insert c2");

        let conv1 = start_conversation(&conn, &c1.id).expect("Start conv1");
        let conv2 = start_conversation(&conn, &c2.id).expect("Start conv2");

        let loaded = load_conversations(&conn).expect("Load should succeed");
        assert_eq!(loaded.len(), 2, "Should have 2 conversations");

        // Most recent first (conv2 was created after conv1)
        assert_eq!(loaded[0].id, conv2.id);
        assert_eq!(loaded[1].id, conv1.id);
    }

    #[test]
    fn test_delete_conversation_cascades() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        // Verify messages exist
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        assert!(!messages.is_empty(), "Should have messages");

        // Delete conversation
        delete_conversation(&conn, &conversation.id).expect("Delete should succeed");

        // Messages should be gone (cascaded)
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        assert!(messages.is_empty(), "Messages should be cascaded");
    }

    #[test]
    fn test_save_and_load_messages() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        // The initial message from start_conversation
        let initial_messages =
            load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        let next_sort_order = initial_messages.len() as i32;

        // Save a user message
        let user_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "Tell me about yourself".to_string(),
            next_sort_order,
        );
        save_message(&conn, &user_msg).expect("Save should succeed");

        // Save an assistant message
        let assistant_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::Assistant,
            "I am Aria, a fantasy character!".to_string(),
            next_sort_order + 1,
        );
        save_message(&conn, &assistant_msg).expect("Save should succeed");

        // Load all messages
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        assert_eq!(
            messages.len(),
            3,
            "Should have 3 messages (initial + user + assistant)"
        );

        // Verify ordering
        assert_eq!(messages[0].sort_order, 0);
        assert_eq!(messages[0].role, MessageRole::Assistant); // initial first_mes
        assert_eq!(messages[1].sort_order, 1);
        assert_eq!(messages[1].role, MessageRole::User);
        assert_eq!(messages[2].sort_order, 2);
        assert_eq!(messages[2].role, MessageRole::Assistant);
    }

    #[test]
    fn test_save_message_updates_conversation() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        // Save a new user message
        let user_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "This is a long message that should be previewed".to_string(),
            10,
        );
        save_message(&conn, &user_msg).expect("Save should succeed");

        // Verify conversation updated_at and last_message_preview were updated
        let convs = load_conversations(&conn).expect("Load should succeed");
        let updated_conv = convs
            .iter()
            .find(|c| c.id == conversation.id)
            .expect("Should find conversation");
        assert_eq!(
            updated_conv.last_message_preview,
            "This is a long message that should be previewed"
        );
    }

    #[test]
    fn test_search_conversations_by_title() {
        let conn = setup_test_db();
        let c1 = make_test_character("Fantasy Hero");
        let c2 = make_test_character("Sci-Fi Pilot");
        insert_character(&conn, &c1).expect("Insert c1");
        insert_character(&conn, &c2).expect("Insert c2");

        start_conversation(&conn, &c1.id).expect("Start conv1");
        start_conversation(&conn, &c2.id).expect("Start conv2");

        let results = search_conversations(&conn, "Fantasy").expect("Search should succeed");
        assert_eq!(
            results.len(),
            1,
            "Should find 1 conversation matching 'Fantasy'"
        );
        assert!(results[0].title.contains("Fantasy Hero"));
    }

    #[test]
    fn test_search_conversations_by_preview() {
        let conn = setup_test_db();
        let mut c1 = CharacterCardV2::new("Aria".to_string());
        c1.first_mes = "I am a mystical sorceress".to_string();
        insert_character(&conn, &c1).expect("Insert c1");

        start_conversation(&conn, &c1.id).expect("Start conv1");

        let results = search_conversations(&conn, "sorceress").expect("Search should succeed");
        assert_eq!(
            results.len(),
            1,
            "Should find 1 conversation matching 'sorceress'"
        );
    }

    #[test]
    fn test_search_conversations_no_match() {
        let conn = setup_test_db();
        let c1 = make_test_character("Aria");
        insert_character(&conn, &c1).expect("Insert c1");
        start_conversation(&conn, &c1.id).expect("Start conv1");

        let results = search_conversations(&conn, "nonexistent").expect("Search should succeed");
        assert!(results.is_empty(), "Should find no conversations");
    }

    #[test]
    fn test_delete_message() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        // Save a user message
        let user_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "Hello".to_string(),
            1,
        );
        save_message(&conn, &user_msg).expect("Save should succeed");

        // Verify it was saved
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        assert_eq!(messages.len(), 2); // initial + user

        // Delete the user message
        delete_message(&conn, &user_msg.id).expect("Delete should succeed");

        // Verify it was deleted
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        assert_eq!(messages.len(), 1); // only initial remains
        assert_eq!(messages[0].role, MessageRole::Assistant);
    }

    #[test]
    fn test_delete_messages_after_sort_order() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        // Save several messages
        let user_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "Hello".to_string(),
            1,
        );
        save_message(&conn, &user_msg).expect("Save should succeed");

        let ai_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::Assistant,
            "Hi there!".to_string(),
            2,
        );
        save_message(&conn, &ai_msg).expect("Save should succeed");

        let user_msg2 = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "How are you?".to_string(),
            3,
        );
        save_message(&conn, &user_msg2).expect("Save should succeed");

        // Verify 4 messages total (initial + 3)
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        assert_eq!(messages.len(), 4);

        // Delete messages after sort_order 1 (the user message at position 1)
        delete_messages_after_sort_order(&conn, &conversation.id, 1)
            .expect("Delete should succeed");

        // Verify only initial + user_msg remain (sort_order 0 and 1)
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0].sort_order, 0);
        assert_eq!(messages[1].sort_order, 1);

        // Verify conversation preview was updated to the latest remaining message
        let convs = load_conversations(&conn).expect("Load should succeed");
        let conv = convs.iter().find(|c| c.id == conversation.id).unwrap();
        assert_eq!(conv.last_message_preview, "Hello");
    }

    #[test]
    fn test_update_message_content() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        let user_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "Original content".to_string(),
            1,
        );
        save_message(&conn, &user_msg).expect("Save should succeed");

        // Update the content
        update_message_content(&conn, &user_msg.id, "Updated content")
            .expect("Update should succeed");

        // Verify the content was updated
        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        let updated = messages.iter().find(|m| m.id == user_msg.id).unwrap();
        assert_eq!(updated.content, "Updated content");
    }

    #[test]
    fn test_update_conversation_persona() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        // Set a persona
        update_conversation_persona(&conn, &conversation.id, Some("persona-123"))
            .expect("Update should succeed");

        let convs = load_conversations(&conn).expect("Load should succeed");
        let conv = convs
            .iter()
            .find(|c| c.id == conversation.id)
            .expect("Should find conversation");
        assert_eq!(conv.creator_persona_id, Some("persona-123".to_string()));

        // Clear the persona
        update_conversation_persona(&conn, &conversation.id, None).expect("Clear should succeed");

        let convs = load_conversations(&conn).expect("Load should succeed");
        let conv = convs
            .iter()
            .find(|c| c.id == conversation.id)
            .expect("Should find conversation");
        assert!(conv.creator_persona_id.is_none());
    }

    #[test]
    fn test_create_and_load_branch() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        let initial_msg_id = &messages[0].id;

        let branch = MessageBranch::new(
            conversation.id.clone(),
            initial_msg_id.clone(),
            "Branch 1".to_string(),
        );
        create_branch(&conn, &branch).expect("Create branch should succeed");

        let loaded =
            load_branches_for_message(&conn, initial_msg_id).expect("Load branches should succeed");
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].id, branch.id);
        assert_eq!(loaded[0].label, "Branch 1");
        assert_eq!(loaded[0].parent_message_id, *initial_msg_id);
    }

    #[test]
    fn test_load_messages_branch_aware() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        let messages = load_messages(&conn, &conversation.id, None).expect("Load should succeed");
        let initial_msg_id = &messages[0].id;

        // Create a branch from the initial message
        let branch = MessageBranch::new(
            conversation.id.clone(),
            initial_msg_id.clone(),
            "Branch 1".to_string(),
        );
        create_branch(&conn, &branch).expect("Create branch should succeed");

        // Save a message on the branch
        let branch_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "Message on branch".to_string(),
            1,
        )
        .with_branch_id(Some(branch.id.clone()));
        save_message(&conn, &branch_msg).expect("Save should succeed");

        // Save a message on root
        let root_msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "Message on root".to_string(),
            1,
        );
        save_message(&conn, &root_msg).expect("Save should succeed");

        // Load root messages: should see initial + root_msg, NOT branch_msg
        let root_messages =
            load_messages(&conn, &conversation.id, None).expect("Load root should succeed");
        assert_eq!(root_messages.len(), 2); // initial + root_msg
        assert!(root_messages.iter().all(|m| m.branch_id.is_none()));

        // Load branch messages: sees shared root (NULL) + branch messages
        // = initial (NULL) + root_msg (NULL) + branch_msg (branch)
        let branch_messages = load_messages(&conn, &conversation.id, Some(&branch.id))
            .expect("Load branch should succeed");
        assert_eq!(branch_messages.len(), 3); // shared root + branch_msg
        assert!(branch_messages
            .iter()
            .any(|m| m.branch_id.as_deref() == Some(branch.id.as_str())));
    }

    #[test]
    fn test_search_messages_fts() {
        let conn = setup_test_db();
        let character = make_test_character("Aria");
        insert_character(&conn, &character).expect("Insert character should succeed");

        let conversation =
            start_conversation(&conn, &character.id).expect("Start conversation should succeed");

        // Save an extra message with searchable content
        let msg = Message::with_order(
            conversation.id.clone(),
            MessageRole::User,
            "Tell me about the mystical forest".to_string(),
            1,
        );
        save_message(&conn, &msg).expect("Save should succeed");

        // Search for "mystical"
        let results = search_messages(&conn, "mystical").expect("Search should succeed");
        assert!(!results.is_empty(), "Should find at least one result");
        assert_eq!(results[0].match_type, "message");
        assert!(results[0].message_snippet.is_some());
        assert!(results[0]
            .message_snippet
            .as_ref()
            .unwrap()
            .contains("<mark>"));
    }
}
