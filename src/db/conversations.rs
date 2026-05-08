use crate::db::models::{row_to_conversation, row_to_message, row_to_message_branch};
use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Conversation, Message, MessageBranch, MessageRole};
use anyhow::Result;
use rusqlite::{params, Connection};

/// Starts a new conversation with an initial assistant message from the character's first_mes.
/// Returns the created Conversation with the initial message saved.
pub fn start_conversation(
    conn: &Connection,
    character: &CharacterCardV2,
    provider_id: &str,
    creator_persona_id: Option<&str>,
) -> Result<Conversation> {
    let mut conversation = Conversation::new(character.id.clone(), provider_id.to_string());
    conversation.creator_persona_id = creator_persona_id.map(|s| s.to_string());

    // Insert the conversation
    conn.prepare_cached(
        "INSERT INTO conversations (id, character_id, provider_id, creator_persona_id, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)"
    )?
    .execute(params![
        &conversation.id,
        &conversation.character_id,
        &conversation.provider_id,
        &conversation.creator_persona_id,
        conversation.created_at.to_string(),
        conversation.updated_at.to_string(),
    ])?;

    // Create initial assistant message from character's first_mes (if not empty)
    if !character.first_mes.is_empty() {
        let initial_message = Message::new(
            conversation.id.clone(),
            None, // root branch
            MessageRole::Assistant,
            character.first_mes.clone(),
        );
        save_message(conn, &initial_message)?;
    }

    Ok(conversation)
}

/// Loads all conversations, ordered by updated_at descending (most recent first).
pub fn load_conversations(conn: &Connection) -> Result<Vec<Conversation>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, provider_id, creator_persona_id, created_at, updated_at
         FROM conversations ORDER BY updated_at DESC"
    )?;
    let conversations = stmt.query_map([], |row| row_to_conversation(row))?;
    let mut result = Vec::new();
    for conv in conversations {
        result.push(conv?);
    }
    Ok(result)
}

/// Loads a single conversation by ID.
pub fn load_conversation(conn: &Connection, id: &str) -> Result<Option<Conversation>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, character_id, provider_id, creator_persona_id, created_at, updated_at
         FROM conversations WHERE id = ?1"
    )?;
    let mut conversations = stmt.query_map(params![id], |row| row_to_conversation(row))?;
    match conversations.next() {
        Some(c) => Ok(Some(c?)),
        None => Ok(None),
    }
}

/// Loads messages for a conversation, optionally filtered by branch.
/// Messages are ordered by sort_order ascending.
pub fn load_messages(
    conn: &Connection,
    conversation_id: &str,
    branch_id: Option<&str>,
) -> Result<Vec<Message>> {
    let mut result = Vec::new();
    
    if let Some(bid) = branch_id {
        let mut stmt = conn.prepare_cached(
            "SELECT id, conversation_id, branch_id, role, content, timestamp, sort_order
             FROM messages WHERE conversation_id = ?1 AND branch_id = ?2 ORDER BY sort_order ASC"
        )?;
        let messages = stmt.query_map(params![conversation_id, bid], |row| row_to_message(row))?;
        for msg in messages {
            result.push(msg?);
        }
    } else {
        let mut stmt = conn.prepare_cached(
            "SELECT id, conversation_id, branch_id, role, content, timestamp, sort_order
             FROM messages WHERE conversation_id = ?1 AND branch_id IS NULL ORDER BY sort_order ASC"
        )?;
        let messages = stmt.query_map(params![conversation_id], |row| row_to_message(row))?;
        for msg in messages {
            result.push(msg?);
        }
    }
    
    Ok(result)
}

/// Saves a new message to the database.
/// Also updates the conversation's updated_at timestamp.
pub fn save_message(conn: &Connection, msg: &Message) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO messages (id, conversation_id, branch_id, role, content, timestamp, sort_order)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)"
    )?
    .execute(params![
        &msg.id,
        &msg.conversation_id,
        &msg.branch_id,
        msg.role.to_string(),
        &msg.content,
        msg.timestamp.to_string(),
        msg.sort_order,
    ])?;

    // Update conversation's updated_at
    let now = chrono::Local::now().naive_local();
    conn.prepare_cached(
        "UPDATE conversations SET updated_at = ?1 WHERE id = ?2"
    )?
    .execute(params![now.to_string(), &msg.conversation_id])?;

    Ok(())
}

/// Updates the content of an existing message.
pub fn update_message_content(conn: &Connection, id: &str, content: &str) -> Result<()> {
    conn.prepare_cached(
        "UPDATE messages SET content = ?1 WHERE id = ?2"
    )?
    .execute(params![content, id])?;
    Ok(())
}

/// Deletes a message by ID.
pub fn delete_message(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM messages WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

/// Deletes a conversation and all its messages (cascades via foreign key).
pub fn delete_conversation(conn: &Connection, id: &str) -> Result<()> {
    conn.prepare_cached("DELETE FROM conversations WHERE id = ?1")?
        .execute(params![id])?;
    Ok(())
}

/// Loads all branches that fork from a specific parent message.
pub fn load_branches_for_message(conn: &Connection, parent_message_id: &str) -> Result<Vec<MessageBranch>> {
    let mut stmt = conn.prepare_cached(
        "SELECT id, conversation_id, parent_message_id, label, created_at
         FROM message_branches WHERE parent_message_id = ?1"
    )?;
    let branches = stmt.query_map(params![parent_message_id], |row| row_to_message_branch(row))?;
    let mut result = Vec::new();
    for branch in branches {
        result.push(branch?);
    }
    Ok(result)
}

/// Creates a new branch in the database.
pub fn create_branch(conn: &Connection, branch: &MessageBranch) -> Result<()> {
    conn.prepare_cached(
        "INSERT INTO message_branches (id, conversation_id, parent_message_id, label, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)"
    )?
    .execute(params![
        &branch.id,
        &branch.conversation_id,
        &branch.parent_message_id,
        &branch.label,
        branch.created_at.to_string(),
    ])?;
    Ok(())
}

/// Counts the number of messages in a conversation.
pub fn count_messages_in_conversation(conn: &Connection, conversation_id: &str) -> Result<i32> {
    let count: i32 = conn.query_row(
        "SELECT COUNT(*) FROM messages WHERE conversation_id = ?1",
        params![conversation_id],
        |row| row.get(0),
    )?;
    Ok(count)
}

/// Updates the creator_persona_id for a conversation.
pub fn update_conversation_persona(
    conn: &Connection,
    conversation_id: &str,
    persona_id: Option<&str>,
) -> Result<()> {
    conn.prepare_cached(
        "UPDATE conversations SET creator_persona_id = ?1 WHERE id = ?2"
    )?
    .execute(params![persona_id, conversation_id])?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::migrations::run_migrations;
    use crate::db::providers::insert_provider;
    use crate::domain::provider::{ProviderConfig, ProviderType};
    use rusqlite::Connection;

    fn setup_test_db() -> Connection {
        let mut conn = Connection::open_in_memory().expect("Failed to open in-memory database");
        run_migrations(&mut conn).expect("Failed to run migrations");
        conn
    }

    fn make_test_character(name: &str) -> CharacterCardV2 {
        let mut char = CharacterCardV2::new(name.to_string());
        char.first_mes = format!("Hello, I am {}!", name);
        char
    }

    fn make_test_provider(name: &str) -> ProviderConfig {
        use chrono::NaiveDateTime;
        ProviderConfig {
            id: format!("prov-{}", name),
            name: name.to_string(),
            provider_type: ProviderType::Ollama,
            base_url: Some("http://localhost:11434".to_string()),
            model_name: "llama3.2".to_string(),
            context_window_size: None,
            enabled: true,
            created_at: NaiveDateTime::parse_from_str("2024-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
            updated_at: NaiveDateTime::parse_from_str("2024-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap(),
        }
    }

    #[test]
    fn test_conversations_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='conversations')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "conversations table should exist after migration 005");
    }

    #[test]
    fn test_messages_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='messages')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "messages table should exist after migration 005");
    }

    #[test]
    fn test_message_branches_table_exists() {
        let conn = setup_test_db();
        let result: bool = conn
            .query_row(
                "SELECT EXISTS (SELECT 1 FROM sqlite_master WHERE type='table' AND name='message_branches')",
                [],
                |row| row.get(0),
            )
            .expect("Failed to query sqlite_master");
        assert!(result, "message_branches table should exist after migration 005");
    }

    #[test]
    fn test_start_conversation() {
        let conn = setup_test_db();
        let character = make_test_character("TestChar");
        let provider = make_test_provider("Ollama");

        // Need to insert character and provider first due to FK constraints
        crate::db::characters::insert_character(&conn, &character).expect("Insert character");
        insert_provider(&conn, &provider).expect("Insert provider");

        let conversation = start_conversation(&conn, &character, &provider.id, None).expect("Start conversation");

        // Conversation should have UUID and correct IDs
        assert!(!conversation.id.is_empty());
        assert_eq!(conversation.character_id, character.id);
        assert_eq!(conversation.provider_id, provider.id);
        assert!(conversation.creator_persona_id.is_none());

        // Initial message should exist
        let messages = load_messages(&conn, &conversation.id, None).expect("Load messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, MessageRole::Assistant);
        assert_eq!(messages[0].content, character.first_mes);
    }

    #[test]
    fn test_save_and_load_messages() {
        let conn = setup_test_db();
        let character = make_test_character("TestChar");
        let provider = make_test_provider("Ollama");

        crate::db::characters::insert_character(&conn, &character).expect("Insert character");
        insert_provider(&conn, &provider).expect("Insert provider");

        let conversation = start_conversation(&conn, &character, &provider.id, None).expect("Start conversation");

        // Save additional messages
        let user_msg = Message::new(
            conversation.id.clone(),
            None,
            MessageRole::User,
            "How are you?".to_string(),
        );
        save_message(&conn, &user_msg).expect("Save user message");

        let assistant_msg = Message::new(
            conversation.id.clone(),
            None,
            MessageRole::Assistant,
            "I'm doing well, thank you!".to_string(),
        );
        save_message(&conn, &assistant_msg).expect("Save assistant message");

        // Load messages - should include initial + user + assistant
        let messages = load_messages(&conn, &conversation.id, None).expect("Load messages");
        assert_eq!(messages.len(), 3);
    }

    #[test]
    fn test_load_conversations_ordered() {
        let conn = setup_test_db();
        let character = make_test_character("TestChar");
        let provider = make_test_provider("Ollama");

        crate::db::characters::insert_character(&conn, &character).expect("Insert character");
        insert_provider(&conn, &provider).expect("Insert provider");

        // Create two conversations with a small delay
        let conv1 = start_conversation(&conn, &character, &provider.id, None).expect("Start conv1");
        std::thread::sleep(std::time::Duration::from_millis(10));
        let conv2 = start_conversation(&conn, &character, &provider.id, None).expect("Start conv2");

        // Load conversations - should be ordered by updated_at DESC (most recent first)
        let conversations = load_conversations(&conn).expect("Load conversations");
        assert_eq!(conversations.len(), 2);
        assert_eq!(conversations[0].id, conv2.id); // Most recent first
        assert_eq!(conversations[1].id, conv1.id);
    }

    #[test]
    fn test_delete_conversation_cascades() {
        let conn = setup_test_db();
        let character = make_test_character("TestChar");
        let provider = make_test_provider("Ollama");

        crate::db::characters::insert_character(&conn, &character).expect("Insert character");
        insert_provider(&conn, &provider).expect("Insert provider");

        let conversation = start_conversation(&conn, &character, &provider.id, None).expect("Start conversation");

        // Save additional message
        let msg = Message::new(
            conversation.id.clone(),
            None,
            MessageRole::User,
            "Test".to_string(),
        );
        save_message(&conn, &msg).expect("Save message");

        let count_before = count_messages_in_conversation(&conn, &conversation.id).expect("Count messages");
        assert_eq!(count_before, 2); // initial + user

        // Delete conversation
        delete_conversation(&conn, &conversation.id).expect("Delete conversation");

        // Conversation and messages should be gone (cascade)
        let loaded = load_conversation(&conn, &conversation.id).expect("Load conversation");
        assert!(loaded.is_none());

        let messages = load_messages(&conn, &conversation.id, None).expect("Load messages");
        assert!(messages.is_empty());
    }

    #[test]
    fn test_update_message_content() {
        let conn = setup_test_db();
        let character = make_test_character("TestChar");
        let provider = make_test_provider("Ollama");

        crate::db::characters::insert_character(&conn, &character).expect("Insert character");
        insert_provider(&conn, &provider).expect("Insert provider");

        let conversation = start_conversation(&conn, &character, &provider.id, None).expect("Start conversation");
        let messages = load_messages(&conn, &conversation.id, None).expect("Load messages");
        let initial_msg = &messages[0];

        // Update the message
        update_message_content(&conn, &initial_msg.id, "Updated content").expect("Update message");

        // Reload and verify
        let updated_messages = load_messages(&conn, &conversation.id, None).expect("Load messages");
        assert_eq!(updated_messages[0].content, "Updated content");
    }

    #[test]
    fn test_create_and_load_branches() {
        let conn = setup_test_db();
        let character = make_test_character("TestChar");
        let provider = make_test_provider("Ollama");

        crate::db::characters::insert_character(&conn, &character).expect("Insert character");
        insert_provider(&conn, &provider).expect("Insert provider");

        let conversation = start_conversation(&conn, &character, &provider.id, None).expect("Start conversation");
        let messages = load_messages(&conn, &conversation.id, None).expect("Load messages");
        let parent_msg = &messages[0];

        // Create a branch
        let branch = MessageBranch::new(
            conversation.id.clone(),
            Some(parent_msg.id.clone()),
            "Alternative response".to_string(),
        );
        create_branch(&conn, &branch).expect("Create branch");

        // Load branches for the parent message
        let branches = load_branches_for_message(&conn, &parent_msg.id).expect("Load branches");
        assert_eq!(branches.len(), 1);
        assert_eq!(branches[0].label, "Alternative response");
    }

    #[test]
    fn test_messages_with_branch() {
        let conn = setup_test_db();
        let character = make_test_character("TestChar");
        let provider = make_test_provider("Ollama");

        crate::db::characters::insert_character(&conn, &character).expect("Insert character");
        insert_provider(&conn, &provider).expect("Insert provider");

        let conversation = start_conversation(&conn, &character, &provider.id, None).expect("Start conversation");

        // Create a branch
        let branch = MessageBranch::new(
            conversation.id.clone(),
            None,
            "Test branch".to_string(),
        );
        create_branch(&conn, &branch).expect("Create branch");

        // Save message with branch
        let msg = Message::new(
            conversation.id.clone(),
            Some(branch.id.clone()),
            MessageRole::User,
            "Branched message".to_string(),
        );
        save_message(&conn, &msg).expect("Save branched message");

        // Load messages with branch filter
        let branch_messages = load_messages(&conn, &conversation.id, Some(&branch.id)).expect("Load branch messages");
        assert_eq!(branch_messages.len(), 1);
        assert_eq!(branch_messages[0].content, "Branched message");

        // Load root messages (should not include branched message)
        let root_messages = load_messages(&conn, &conversation.id, None).expect("Load root messages");
        assert_eq!(root_messages.len(), 1); // Just the initial first_mes
    }
}
