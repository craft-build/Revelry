use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Persona domain model — represents the user's identity in roleplay.
/// Contains 8 structured fields that describe the user's persona.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Persona {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub appearance: String,
    #[serde(default)]
    pub personality: String,
    #[serde(default)]
    pub background: String,
    #[serde(default)]
    pub role_description: String,
    #[serde(default)]
    pub relationships: String,
    #[serde(default)]
    pub speech_style: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub avatar_path: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Persona {
    /// Creates a new persona with a generated UUID v4 id, current timestamps,
    /// and all optional fields empty.
    pub fn new(name: String, personality: String) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            appearance: String::new(),
            personality,
            background: String::new(),
            role_description: String::new(),
            relationships: String::new(),
            speech_style: String::new(),
            tags: Vec::new(),
            avatar_path: None,
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
        let persona = Persona::new("Alice".to_string(), "Bold and charismatic".to_string());

        // UUID v4 id should be valid and non-empty
        assert!(!persona.id.is_empty());
        let parsed = Uuid::parse_str(&persona.id);
        assert!(
            parsed.is_ok(),
            "id should be valid UUID v4, got: {}",
            persona.id
        );

        // Name and personality should match
        assert_eq!(persona.name, "Alice");
        assert_eq!(persona.personality, "Bold and charismatic");

        // Timestamps should be set
        let now = Utc::now();
        assert!(persona.created_at <= now);
        assert!(persona.updated_at <= now);

        // Optional fields should be empty by default
        assert!(persona.appearance.is_empty());
        assert!(persona.background.is_empty());
        assert!(persona.role_description.is_empty());
        assert!(persona.relationships.is_empty());
        assert!(persona.speech_style.is_empty());
        assert!(persona.tags.is_empty());
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut persona =
            Persona::new("Roundtrip Test".to_string(), "Test personality".to_string());
        persona.appearance = "Red hair, blue eyes".to_string();
        persona.background = "A wandering mage".to_string();
        persona.role_description = "Mage".to_string();
        persona.relationships = "Friend of Bob".to_string();
        persona.speech_style = "Formal".to_string();
        persona.tags = vec!["fantasy".to_string(), "mage".to_string()];

        let json = serde_json::to_string(&persona).expect("serialize failed");
        let deserialized: Persona = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(
            persona, deserialized,
            "roundtrip should preserve all fields"
        );
    }

    #[test]
    fn test_defaults_on_partial_json() {
        // Partial JSON with only required fields should deserialize with defaults
        let json = r#"{"id": "test-id", "name": "Partial", "personality": "", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"}"#;
        let persona: Persona = serde_json::from_str(json).expect("partial JSON should deserialize");

        assert_eq!(persona.id, "test-id");
        assert_eq!(persona.name, "Partial");
        assert!(persona.appearance.is_empty());
        assert!(persona.tags.is_empty());
    }
}
