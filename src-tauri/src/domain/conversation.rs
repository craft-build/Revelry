use jiff::Timestamp;
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
    /// Per-conversation max output token override. Takes precedence over preset and provider defaults.
    /// None = use preset/provider default.
    pub override_max_tokens: Option<i64>,
    /// Display title for the conversation (may be auto-generated or user-set).
    pub title: String,
    /// Truncated preview of the last message for list display.
    pub last_message_preview: String,
    /// Rolling story summary maintained in the background and injected into
    /// the prompt once history starts truncating (memory layer, D-04.5).
    #[serde(default)]
    pub summary: String,
    /// Highest message sort_order covered by `summary` (0 = nothing covered).
    #[serde(default)]
    pub summary_upto: i64,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Conversation {
    /// Creates a new conversation with a generated UUID v4 id and current timestamps.
    /// provider_id is None (no providers until Phase 3).
    pub fn new(character_id: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4().to_string(),
            character_id,
            provider_id: None,
            model_name: None,
            creator_persona_id: None,
            generation_preset_id: None,
            override_max_tokens: None,
            title: String::new(),
            last_message_preview: String::new(),
            summary: String::new(),
            summary_upto: 0,
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
/// Phase 8 — includes branch_id for conversation forking.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub role: MessageRole,
    pub content: String,
    pub sort_order: i32,
    pub timestamp: Timestamp,
    /// Branch this message belongs to. None = Main/root path.
    pub branch_id: Option<String>,
}

impl Message {
    /// Creates a new message with a generated UUID v4 id and current timestamp.
    pub fn new(conversation_id: String, role: MessageRole, content: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            role,
            content,
            sort_order: 0,
            timestamp: now,
            branch_id: None,
        }
    }

    /// Creates a new message with a specific sort_order.
    pub fn with_order(
        conversation_id: String,
        role: MessageRole,
        content: String,
        sort_order: i32,
    ) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            role,
            content,
            sort_order,
            timestamp: now,
            branch_id: None,
        }
    }

    /// Builder method to set branch_id.
    pub fn with_branch_id(mut self, branch_id: Option<String>) -> Self {
        self.branch_id = branch_id;
        self
    }
}

/// A branch in a conversation, created when a user forks from a message.
/// Tracks which message the branch diverges from and a human-readable label.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct MessageBranch {
    pub id: String,
    pub conversation_id: String,
    pub parent_message_id: String,
    pub label: String,
    pub created_at: Timestamp,
}

impl MessageBranch {
    /// Creates a new branch with a generated UUID v4 id and current timestamp.
    pub fn new(conversation_id: String, parent_message_id: String, label: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4().to_string(),
            conversation_id,
            parent_message_id,
            label,
            created_at: now,
        }
    }
}

/// A search result from FTS5 message search or conversation title search.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct SearchResult {
    pub conversation_id: String,
    pub conversation_title: String,
    pub character_id: String,
    /// Either "title" or "message"
    pub match_type: String,
    pub message_snippet: Option<String>,
    pub message_id: Option<String>,
    pub branch_id: Option<String>,
    pub rank: f64,
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
        let now = Timestamp::now();
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
        let now = Timestamp::now();
        assert!(msg.timestamp <= now);

        // branch_id should be None by default
        assert!(msg.branch_id.is_none());
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
        assert!(msg.branch_id.is_none());
    }

    #[test]
    fn test_message_with_branch_id() {
        let msg = Message::new(
            "conv-1".to_string(),
            MessageRole::User,
            "Hello!".to_string(),
        )
        .with_branch_id(Some("branch-1".to_string()));

        assert_eq!(msg.branch_id, Some("branch-1".to_string()));
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

    #[test]
    fn test_message_branch_new() {
        let branch = MessageBranch::new(
            "conv-1".to_string(),
            "msg-1".to_string(),
            "Branch 1".to_string(),
        );

        assert!(!branch.id.is_empty());
        assert!(Uuid::parse_str(&branch.id).is_ok());
        assert_eq!(branch.conversation_id, "conv-1");
        assert_eq!(branch.parent_message_id, "msg-1");
        assert_eq!(branch.label, "Branch 1");
        let now = Timestamp::now();
        assert!(branch.created_at <= now);
    }

    #[test]
    fn test_serde_roundtrip_message_branch() {
        let branch = MessageBranch::new(
            "conv-1".to_string(),
            "msg-1".to_string(),
            "Branch 1".to_string(),
        );
        let json = serde_json::to_string(&branch).expect("serialize failed");
        let deserialized: MessageBranch = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(branch, deserialized);
    }

    #[test]
    fn test_serde_roundtrip_search_result() {
        let result = SearchResult {
            conversation_id: "conv-1".to_string(),
            conversation_title: "Chat with Aria".to_string(),
            character_id: "char-1".to_string(),
            match_type: "message".to_string(),
            message_snippet: Some("Hello <mark>world</mark>".to_string()),
            message_id: Some("msg-1".to_string()),
            branch_id: None,
            rank: -1.5,
        };
        let json = serde_json::to_string(&result).expect("serialize failed");
        let deserialized: SearchResult = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(result, deserialized);
    }
}
