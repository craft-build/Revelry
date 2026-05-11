use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// CharacterCardV2 domain model — the canonical character format.
/// Compatible with the SillyTavern CharacterCardV2 specification.
/// All fields use serde(default) so partial JSON still deserializes.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct CharacterCardV2 {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub scenario: String,
    #[serde(default)]
    pub first_mes: String,
    #[serde(default)]
    pub mes_example: String,
    #[serde(default)]
    pub system_prompt: String,
    #[serde(default)]
    pub post_history_instructions: String,
    #[serde(default)]
    pub alternate_greetings: Vec<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default = "default_creator")]
    pub creator: String,
    #[serde(default = "default_character_version")]
    pub character_version: String,
    #[serde(default)]
    pub avatar_path: Option<String>,
    #[serde(default = "default_extensions")]
    pub extensions: serde_json::Value,
    /// Notes from the creator about the card (not used in prompt generation).
    /// Maps to V2 spec's `creator_notes` field.
    #[serde(default)]
    pub creator_notes: String,
    /// Character lorebook / world info (JSON, not used in prompt by default).
    /// Maps to V2 spec's `character_book` field.
    #[serde(default = "default_character_book")]
    pub character_book: serde_json::Value,
    /// Per-character default persona ID for persona assignment hierarchy.
    #[serde(default)]
    pub default_persona_id: Option<String>,
    /// Per-character default preset ID for generation parameter hierarchy per D-03.
    #[serde(default)]
    pub default_preset_id: Option<String>,
    /// Number of recent messages to scan for keyword matching when activating lorebook entries.
    /// Per D-10: per-character scan depth (default 2).
    #[serde(default = "default_lorebook_scan_depth")]
    pub lorebook_scan_depth: i32,
    /// Maximum depth of recursive lorebook activation (how many times activated entries can trigger other entries).
    /// Per D-12: recursive activation depth limit (default 2).
    #[serde(default = "default_lorebook_recursive_depth")]
    pub lorebook_recursive_depth: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn default_creator() -> String {
    "Unknown".to_string()
}

fn default_character_version() -> String {
    "2.0".to_string()
}

fn default_extensions() -> serde_json::Value {
    serde_json::Value::Null
}

fn default_character_book() -> serde_json::Value {
    serde_json::Value::Null
}

fn default_lorebook_scan_depth() -> i32 {
    2
}

fn default_lorebook_recursive_depth() -> i32 {
    2
}

impl CharacterCardV2 {
    /// Creates a new character with a generated UUID v4 id, current timestamps,
    /// character_version "2.0", and all other fields at their defaults.
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            description: String::new(),
            personality: String::new(),
            scenario: String::new(),
            first_mes: String::new(),
            mes_example: String::new(),
            system_prompt: String::new(),
            post_history_instructions: String::new(),
            alternate_greetings: Vec::new(),
            tags: Vec::new(),
            creator: "Unknown".to_string(),
            character_version: "2.0".to_string(),
            avatar_path: None,
            extensions: serde_json::Value::Null,
            creator_notes: String::new(),
            character_book: serde_json::Value::Null,
            default_persona_id: None,
            default_preset_id: None,
            lorebook_scan_depth: 2,
            lorebook_recursive_depth: 2,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generates_uuid_and_defaults() {
        let character = CharacterCardV2::new("Test Character".to_string());

        // UUID v4 id should be valid and non-empty
        assert!(!character.id.is_empty());
        let parsed = Uuid::parse_str(&character.id);
        assert!(parsed.is_ok(), "id should be valid UUID v4, got: {}", character.id);

        // Name should match
        assert_eq!(character.name, "Test Character");

        // character_version defaults to "2.0"
        assert_eq!(character.character_version, "2.0");

        // Timestamps should be set
        let now = Utc::now();
        assert!(character.created_at <= now);
        assert!(character.updated_at <= now);

        // Optional fields should default correctly
        assert_eq!(character.creator, "Unknown");
        assert!(character.avatar_path.is_none());
        assert_eq!(character.extensions, serde_json::Value::Null);
        assert!(character.creator_notes.is_empty());
        assert_eq!(character.character_book, serde_json::Value::Null);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut character = CharacterCardV2::new("Roundtrip Test".to_string());
        character.description = "A brave hero".to_string();
        character.personality = "Bold and fearless".to_string();
        character.alternate_greetings = vec!["Hello there!".to_string(), "Greetings!".to_string()];
        character.tags = vec!["fantasy".to_string(), "hero".to_string()];
        character.creator = "TestCreator".to_string();
        character.avatar_path = Some("/path/to/avatar.png".to_string());
        character.extensions = serde_json::json!({"custom_field": "value"});
        character.creator_notes = "A test note".to_string();
        character.character_book = serde_json::json!({"entries": [], "name": "test"});

        let json = serde_json::to_string(&character).expect("serialize failed");
        let deserialized: CharacterCardV2 =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(character, deserialized, "roundtrip should preserve all fields");
    }

    #[test]
    fn test_defaults_on_partial_json() {
        // Partial JSON with only the required name field should deserialize with defaults
        let json = r#"{"id": "test-id", "name": "Partial", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"}"#;
        let character: CharacterCardV2 = serde_json::from_str(json).expect("partial JSON should deserialize");

        assert_eq!(character.id, "test-id");
        assert_eq!(character.name, "Partial");
        assert!(character.description.is_empty());
        assert!(character.alternate_greetings.is_empty());
        assert!(character.tags.is_empty());
        assert_eq!(character.creator, "Unknown");
        assert_eq!(character.character_version, "2.0");
        assert!(character.avatar_path.is_none());
        assert_eq!(character.extensions, serde_json::Value::Null);
        assert!(character.creator_notes.is_empty());
        assert_eq!(character.character_book, serde_json::Value::Null);
    }

    #[test]
    fn test_lorebook_defaults() {
        let character = CharacterCardV2::new("Lorebook Test".to_string());
        assert_eq!(character.lorebook_scan_depth, 2, "lorebook_scan_depth should default to 2");
        assert_eq!(character.lorebook_recursive_depth, 2, "lorebook_recursive_depth should default to 2");
    }

    #[test]
    fn test_lorebook_fields_serde_roundtrip() {
        let mut character = CharacterCardV2::new("Lorebook Roundtrip".to_string());
        character.lorebook_scan_depth = 5;
        character.lorebook_recursive_depth = 3;
        let json = serde_json::to_string(&character).expect("serialize failed");
        let deserialized: CharacterCardV2 =
            serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(deserialized.lorebook_scan_depth, 5);
        assert_eq!(deserialized.lorebook_recursive_depth, 3);
    }
}
