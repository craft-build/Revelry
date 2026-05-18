use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// A conversation between a user and a character.
/// Tracks which character, optional provider, and optional persona are active.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub character_id: String,
    /// Nullable — no providers in Phase 2.
    pub provider_id: Option<String>,
    /// Specific model within a provider. Paired with provider_id to identify the model.
    pub model_name: Option<String>,
    pub creator_persona_id: Option<String>,
    /// Per-conversation preset override per D-03. Resolved via hierarchy:
    /// conversation → character → global → none (provider defaults).
    pub generation_preset_id: Option<String>,
    /// Display title for the conversation (may be auto-generated or user-set).
    pub title: String,
    /// Truncated preview of the last message for list display.
    pub last_message_preview: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Conversation {
    /// Creates a new conversation with a generated UUID v4 id and current timestamps.
    /// provider_id is None (no providers until Phase 3).
    pub fn new(character_id: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            character_id,
            provider_id: None,
            model_name: None,
            creator_persona_id: None,
            generation_preset_id: None,
            title: String::new(),
            last_message_preview: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// The role of a message sender in a conversation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum MessageRole {
    User,
    Assistant,
    System,
}

impl fmt::Display for MessageRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageRole::User => write!(f, "user"),
            MessageRole::Assistant => write!(f, "assistant"),
            MessageRole::System => write!(f, "system"),
        }
    }
}

impl FromStr for MessageRole {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "user" => Ok(MessageRole::User),
            "assistant" => Ok(MessageRole::Assistant),
            "system" => Ok(MessageRole::System),
            _ => Err(format!("Unknown message role: {}", s)),
        }
    }
}

/// A single message within a conversation.
/// Phase 2 — no branch_id (deferred to Phase 8).
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sort_order: i32,
    pub timestamp: DateTime<Utc>,
}

impl Message {
    /// Creates a new message with a generated UUID v4 id and current timestamp.
    pub fn new(conversation_id: String, role: MessageRole, content: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            role,
            content,
            sort_order: 0,
            timestamp: now,
        }
    }

    /// Creates a new message with a specific sort_order.
    pub fn with_order(
        conversation_id: String,
        role: MessageRole,
        content: String,
        sort_order: i32,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            role,
            content,
            sort_order,
            timestamp: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_new_generates_uuid() {
        let conv = Conversation::new("char-1".to_string());

        // UUID v4 id should be valid and non-empty
        assert!(!conv.id.is_empty());
        let parsed = Uuid::parse_str(&conv.id);
        assert!(
            parsed.is_ok(),
            "id should be valid UUID v4, got: {}",
            conv.id
        );

        // Character ID should match
        assert_eq!(conv.character_id, "char-1");

        // Provider should be None (no providers in Phase 2)
        assert!(conv.provider_id.is_none());

        // Persona should be None by default
        assert!(conv.creator_persona_id.is_none());

        // Title and preview should be empty by default
        assert!(conv.title.is_empty());
        assert!(conv.last_message_preview.is_empty());

        // Timestamps should be set
        let now = Utc::now();
        assert!(conv.created_at <= now);
        assert!(conv.updated_at <= now);
    }

    #[test]
    fn test_message_new_defaults() {
        let msg = Message::new(
            "conv-1".to_string(),
            MessageRole::User,
            "Hello!".to_string(),
        );

        // UUID v4 id should be valid
        assert!(!msg.id.is_empty());
        assert!(Uuid::parse_str(&msg.id).is_ok());

        // Conversation ID should match
        assert_eq!(msg.conversation_id, "conv-1");

        // Role and content should match
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello!");

        // Sort order should default to 0
        assert_eq!(msg.sort_order, 0);

        // Timestamp should be set
        let now = Utc::now();
        assert!(msg.timestamp <= now);
    }

    #[test]
    fn test_message_with_order() {
        let msg = Message::with_order(
            "conv-1".to_string(),
            MessageRole::Assistant,
            "Hi there!".to_string(),
            5,
        );

        assert_eq!(msg.role, MessageRole::Assistant);
        assert_eq!(msg.sort_order, 5);
    }

    #[test]
    fn test_message_role_display() {
        assert_eq!(MessageRole::User.to_string(), "user");
        assert_eq!(MessageRole::Assistant.to_string(), "assistant");
        assert_eq!(MessageRole::System.to_string(), "system");
    }

    #[test]
    fn test_message_role_from_str() {
        assert_eq!(MessageRole::from_str("user").unwrap(), MessageRole::User);
        assert_eq!(MessageRole::from_str("USER").unwrap(), MessageRole::User);
        assert_eq!(MessageRole::from_str("User").unwrap(), MessageRole::User);
        assert_eq!(
            MessageRole::from_str("assistant").unwrap(),
            MessageRole::Assistant
        );
        assert_eq!(
            MessageRole::from_str("system").unwrap(),
            MessageRole::System
        );
        assert!(MessageRole::from_str("unknown").is_err());
    }

    #[test]
    fn test_serde_roundtrip_conversation() {
        let conv = Conversation::new("char-1".to_string());
        let json = serde_json::to_string(&conv).expect("serialize failed");
        let deserialized: Conversation = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(conv, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_message() {
        let msg = Message::new(
            "conv-1".to_string(),
            MessageRole::Assistant,
            "Hello, world!".to_string(),
        );
        let json = serde_json::to_string(&msg).expect("serialize failed");
        let deserialized: Message = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(msg, deserialized);
    }
}
