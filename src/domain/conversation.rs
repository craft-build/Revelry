use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// A conversation between a user and a character.
/// Tracks which character, provider, and optional persona are active.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub character_id: String,
    pub provider_id: String,
    pub creator_persona_id: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl Conversation {
    /// Creates a new conversation with a generated UUID v4 id and current timestamps.
    pub fn new(character_id: String, provider_id: String) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            character_id,
            provider_id,
            creator_persona_id: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// The role of a message sender in a conversation.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
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
/// Messages can belong to a branch for alternative conversation paths.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub branch_id: Option<String>,
    pub role: MessageRole,
    pub content: String,
    pub timestamp: NaiveDateTime,
    pub sort_order: i32,
}

impl Message {
    /// Creates a new message with a generated UUID v4 id and current timestamp.
    pub fn new(
        conversation_id: String,
        branch_id: Option<String>,
        role: MessageRole,
        content: String,
    ) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            branch_id,
            role,
            content,
            timestamp: now,
            sort_order: 0,
        }
    }

    /// Creates a new message with a specific sort_order.
    pub fn with_order(
        conversation_id: String,
        branch_id: Option<String>,
        role: MessageRole,
        content: String,
        sort_order: i32,
    ) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            branch_id,
            role,
            content,
            timestamp: now,
            sort_order,
        }
    }
}

/// A branch represents an alternative path in a conversation.
/// Branches fork from a specific parent message.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessageBranch {
    pub id: String,
    pub conversation_id: String,
    pub parent_message_id: Option<String>,
    pub label: String,
    pub created_at: NaiveDateTime,
}

impl MessageBranch {
    /// Creates a new branch with a generated UUID v4 id and current timestamp.
    pub fn new(
        conversation_id: String,
        parent_message_id: Option<String>,
        label: String,
    ) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            parent_message_id,
            label,
            created_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conversation_new_generates_uuid() {
        let conv = Conversation::new("char-1".to_string(), "prov-1".to_string());

        // UUID v4 id should be valid and non-empty
        assert!(!conv.id.is_empty());
        let parsed = Uuid::parse_str(&conv.id);
        assert!(parsed.is_ok(), "id should be valid UUID v4, got: {}", conv.id);

        // Character and provider IDs should match
        assert_eq!(conv.character_id, "char-1");
        assert_eq!(conv.provider_id, "prov-1");

        // Persona should be None by default
        assert!(conv.creator_persona_id.is_none());

        // Timestamps should be set
        let now = chrono::Local::now().naive_local();
        assert!(conv.created_at <= now);
        assert!(conv.updated_at <= now);
    }

    #[test]
    fn test_message_new_defaults() {
        let msg = Message::new(
            "conv-1".to_string(),
            Some("branch-1".to_string()),
            MessageRole::User,
            "Hello!".to_string(),
        );

        // UUID v4 id should be valid
        assert!(!msg.id.is_empty());
        assert!(Uuid::parse_str(&msg.id).is_ok());

        // Conversation and branch IDs should match
        assert_eq!(msg.conversation_id, "conv-1");
        assert_eq!(msg.branch_id, Some("branch-1".to_string()));

        // Role and content should match
        assert_eq!(msg.role, MessageRole::User);
        assert_eq!(msg.content, "Hello!");

        // Sort order should default to 0
        assert_eq!(msg.sort_order, 0);

        // Timestamp should be set
        let now = chrono::Local::now().naive_local();
        assert!(msg.timestamp <= now);
    }

    #[test]
    fn test_message_with_order() {
        let msg = Message::with_order(
            "conv-1".to_string(),
            None,
            MessageRole::Assistant,
            "Hi there!".to_string(),
            5,
        );

        assert_eq!(msg.branch_id, None);
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
        assert_eq!(MessageRole::from_str("assistant").unwrap(), MessageRole::Assistant);
        assert_eq!(MessageRole::from_str("system").unwrap(), MessageRole::System);
        assert!(MessageRole::from_str("unknown").is_err());
    }

    #[test]
    fn test_message_branch_new() {
        let branch = MessageBranch::new(
            "conv-1".to_string(),
            Some("msg-1".to_string()),
            "Alternative path".to_string(),
        );

        // UUID v4 id should be valid
        assert!(!branch.id.is_empty());
        assert!(Uuid::parse_str(&branch.id).is_ok());

        assert_eq!(branch.conversation_id, "conv-1");
        assert_eq!(branch.parent_message_id, Some("msg-1".to_string()));
        assert_eq!(branch.label, "Alternative path");

        let now = chrono::Local::now().naive_local();
        assert!(branch.created_at <= now);
    }

    #[test]
    fn test_message_branch_root() {
        // Branch with no parent (root branch)
        let branch = MessageBranch::new(
            "conv-1".to_string(),
            None,
            "Main storyline".to_string(),
        );

        assert_eq!(branch.parent_message_id, None);
    }

    #[test]
    fn test_serde_roundtrip_conversation() {
        let conv = Conversation::new("char-1".to_string(), "prov-1".to_string());
        let json = serde_json::to_string(&conv).expect("serialize failed");
        let deserialized: Conversation = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(conv, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_message() {
        let msg = Message::new(
            "conv-1".to_string(),
            Some("branch-1".to_string()),
            MessageRole::Assistant,
            "Hello, world!".to_string(),
        );
        let json = serde_json::to_string(&msg).expect("serialize failed");
        let deserialized: Message = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(msg, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_message_branch() {
        let branch = MessageBranch::new(
            "conv-1".to_string(),
            Some("msg-1".to_string()),
            "Branch A".to_string(),
        );
        let json = serde_json::to_string(&branch).expect("serialize failed");
        let deserialized: MessageBranch = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(branch, deserialized);
    }
}
