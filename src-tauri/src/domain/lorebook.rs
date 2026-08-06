use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Position where a lorebook entry is inserted relative to the character card.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum LorebookPosition {
    BeforeChar,
    #[default]
    AfterChar,
    AtDepth,
}

impl fmt::Display for LorebookPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LorebookPosition::BeforeChar => write!(f, "before_char"),
            LorebookPosition::AfterChar => write!(f, "after_char"),
            LorebookPosition::AtDepth => write!(f, "at_depth"),
        }
    }
}

impl FromStr for LorebookPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "before_char" => Ok(LorebookPosition::BeforeChar),
            "after_char" => Ok(LorebookPosition::AfterChar),
            "at_depth" => Ok(LorebookPosition::AtDepth),
            _ => Err(format!("Invalid LorebookPosition: {}", s)),
        }
    }
}

/// Activation condition for a lorebook entry.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum LorebookActivation {
    #[default]
    Always,
    CharacterOnly,
    UserOnly,
}

impl fmt::Display for LorebookActivation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LorebookActivation::Always => write!(f, "always"),
            LorebookActivation::CharacterOnly => write!(f, "character_only"),
            LorebookActivation::UserOnly => write!(f, "user_only"),
        }
    }
}

impl FromStr for LorebookActivation {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "always" => Ok(LorebookActivation::Always),
            "character_only" => Ok(LorebookActivation::CharacterOnly),
            "user_only" => Ok(LorebookActivation::UserOnly),
            _ => Err(format!("Invalid LorebookActivation: {}", s)),
        }
    }
}

/// A standalone lorebook (world info) that can be shared across characters.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Lorebook {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

impl Lorebook {
    /// Creates a new lorebook with a generated UUID v4 id, current timestamps,
    /// and empty description.
    pub fn new(name: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            created_at: now,
            updated_at: now,
        }
    }
}

/// A single entry within a lorebook — a piece of world info activated by keywords.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LorebookEntry {
    pub id: String,
    pub lorebook_id: String,
    #[serde(default)]
    pub keys: Vec<String>,
    #[serde(default)]
    pub secondary_keys: Vec<String>,
    #[serde(default)]
    pub content: String,
    #[serde(default)]
    pub position: LorebookPosition,
    #[serde(default)]
    pub depth: i32,
    #[serde(default)]
    pub activation_state: LorebookActivation,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub token_budget: Option<i64>,
    #[serde(default = "default_sort_order")]
    pub sort_order: i32,
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    #[serde(default = "default_extensions")]
    pub extensions: serde_json::Value,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

fn default_extensions() -> serde_json::Value {
    serde_json::json!({})
}

fn default_sort_order() -> i32 {
    0
}

fn default_enabled() -> bool {
    true
}

impl LorebookEntry {
    /// Creates a new lorebook entry with a generated UUID v4 id, current timestamps,
    /// and all other fields at their defaults.
    pub fn new(lorebook_id: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4().to_string(),
            lorebook_id,
            keys: Vec::new(),
            secondary_keys: Vec::new(),
            content: String::new(),
            position: LorebookPosition::AfterChar,
            depth: 4,
            activation_state: LorebookActivation::Always,
            token_budget: None,
            sort_order: 0,
            enabled: true,
            extensions: serde_json::json!({}),
            created_at: now,
            updated_at: now,
        }
    }
}

/// Detail about a single lorebook entry that was activated during prompt assembly.
/// Sent in the StreamChunk::Complete payload for frontend display.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LorebookActivationDetail {
    pub lorebook_name: String,
    pub entry_name: String,
    pub content_snippet: String,
    pub token_cost: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub matched_keys: Option<Vec<String>>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lorebook_new() {
        let lorebook = Lorebook::new("Test Lorebook".to_string());
        assert!(!lorebook.id.is_empty());
        let parsed = Uuid::parse_str(&lorebook.id);
        assert!(parsed.is_ok(), "id should be valid UUID v4");
        assert_eq!(lorebook.name, "Test Lorebook");
        assert!(lorebook.description.is_empty());
    }

    #[test]
    fn test_lorebook_entry_new() {
        let entry = LorebookEntry::new("lorebook-123".to_string());
        assert!(!entry.id.is_empty());
        let parsed = Uuid::parse_str(&entry.id);
        assert!(parsed.is_ok(), "id should be valid UUID v4");
        assert_eq!(entry.lorebook_id, "lorebook-123");
        assert!(entry.keys.is_empty());
        assert!(entry.secondary_keys.is_empty());
        assert!(entry.content.is_empty());
        assert_eq!(entry.position, LorebookPosition::AfterChar);
        assert_eq!(entry.depth, 4);
        assert_eq!(entry.activation_state, LorebookActivation::Always);
        assert!(entry.token_budget.is_none());
        assert_eq!(entry.sort_order, 0);
        assert!(entry.enabled);
        assert_eq!(entry.extensions, serde_json::json!({}));
    }

    #[test]
    fn test_lorebook_position_display_from_str() {
        assert_eq!(LorebookPosition::BeforeChar.to_string(), "before_char");
        assert_eq!(LorebookPosition::AfterChar.to_string(), "after_char");
        assert_eq!(LorebookPosition::AtDepth.to_string(), "at_depth");

        assert_eq!(
            "before_char".parse::<LorebookPosition>(),
            Ok(LorebookPosition::BeforeChar)
        );
        assert_eq!(
            "after_char".parse::<LorebookPosition>(),
            Ok(LorebookPosition::AfterChar)
        );
        assert_eq!(
            "at_depth".parse::<LorebookPosition>(),
            Ok(LorebookPosition::AtDepth)
        );
        assert!("invalid".parse::<LorebookPosition>().is_err());
    }

    #[test]
    fn test_lorebook_activation_display_from_str() {
        assert_eq!(LorebookActivation::Always.to_string(), "always");
        assert_eq!(
            LorebookActivation::CharacterOnly.to_string(),
            "character_only"
        );
        assert_eq!(LorebookActivation::UserOnly.to_string(), "user_only");

        assert_eq!(
            "always".parse::<LorebookActivation>(),
            Ok(LorebookActivation::Always)
        );
        assert_eq!(
            "character_only".parse::<LorebookActivation>(),
            Ok(LorebookActivation::CharacterOnly)
        );
        assert_eq!(
            "user_only".parse::<LorebookActivation>(),
            Ok(LorebookActivation::UserOnly)
        );
        assert!("invalid".parse::<LorebookActivation>().is_err());
    }

    #[test]
    fn test_lorebook_serde_roundtrip() {
        let mut lorebook = Lorebook::new("Roundtrip".to_string());
        lorebook.description = "A test lorebook".to_string();

        let json = serde_json::to_string(&lorebook).expect("serialize failed");
        let deserialized: Lorebook = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(lorebook, deserialized);
    }

    #[test]
    fn test_lorebook_entry_serde_roundtrip() {
        let mut entry = LorebookEntry::new("lb-1".to_string());
        entry.keys = vec!["magic".to_string(), "spell".to_string()];
        entry.secondary_keys = vec!["wizard".to_string()];
        entry.content = "Magic flows through everything.".to_string();
        entry.position = LorebookPosition::AtDepth;
        entry.depth = 2;
        entry.activation_state = LorebookActivation::CharacterOnly;
        entry.token_budget = Some(100);
        entry.sort_order = 2;
        entry.enabled = false;
        entry.extensions = serde_json::json!({"custom": true});

        let json = serde_json::to_string(&entry).expect("serialize failed");
        let deserialized: LorebookEntry = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(entry, deserialized);
    }

    #[test]
    fn test_lorebook_entry_defaults() {
        let entry = LorebookEntry::new("lb-defaults".to_string());
        assert!(entry.token_budget.is_none());
        assert_eq!(entry.sort_order, 0);
        assert!(entry.enabled);
    }

    #[test]
    fn test_lorebook_entry_serde_with_token_budget() {
        let mut entry = LorebookEntry::new("lb-budget".to_string());
        entry.token_budget = Some(50);

        let json = serde_json::to_string(&entry).expect("serialize failed");
        assert!(
            json.contains("\"token_budget\":50"),
            "Serialized JSON should include token_budget when Some"
        );
        let deserialized: LorebookEntry = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.token_budget, Some(50));
    }

    #[test]
    fn test_lorebook_entry_serde_without_token_budget() {
        let entry = LorebookEntry::new("lb-no-budget".to_string());

        let json = serde_json::to_string(&entry).expect("serialize failed");
        assert!(
            !json.contains("token_budget"),
            "Serialized JSON should NOT include token_budget when None"
        );
        let deserialized: LorebookEntry = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.token_budget, None);
    }
}
