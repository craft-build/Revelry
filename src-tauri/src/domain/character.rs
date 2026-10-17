use jiff::Timestamp;
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
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

fn default_creator() -> String {
    "Unknown".to_string()
}

fn default_character_version() -> String {
    "2.0".to_string()
}

fn default_extensions() -> serde_json::Value {
    serde_json::json!({})
}

fn default_character_book() -> serde_json::Value {
    serde_json::json!({})
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
        let now = Timestamp::now();
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
            extensions: serde_json::json!({}),
            creator_notes: String::new(),
            character_book: serde_json::json!({}),
            default_persona_id: None,
            default_preset_id: None,
            lorebook_scan_depth: 2,
            lorebook_recursive_depth: 2,
            created_at: now,
            updated_at: now,
        }
    }

    /// Parses the embedded `character_book` (SillyTavern CharacterCardV2 spec)
    /// into `LorebookEntry` values for prompt activation. Returns an empty vec
    /// when the book is absent, empty, or malformed (parsing is best-effort and
    /// never panics on unexpected shapes). Each parsed entry is assigned a
    /// synthetic `lorebook_id` (the book's `name`, or the character id) so it can
    /// be merged with junction-table entries and reported in activation details.
    pub fn character_book_entries(&self) -> Vec<crate::domain::lorebook::LorebookEntry> {
        use crate::domain::lorebook::{LorebookActivation, LorebookEntry, LorebookPosition};

        let entries_val = match self.character_book.get("entries") {
            Some(v) => v,
            None => return Vec::new(),
        };

        // The spec allows `entries` as either an array or an object keyed by
        // entry id; normalize both to a vec of owned entry values.
        let entry_values: Vec<serde_json::Value> = match entries_val {
            serde_json::Value::Array(arr) => arr.clone(),
            serde_json::Value::Object(map) => map.values().cloned().collect(),
            _ => Vec::new(),
        };

        let book_name = self
            .character_book
            .get("name")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| format!("{}-embedded", self.id));

        let now = Timestamp::now();
        let mut result = Vec::new();
        for (idx, raw) in entry_values.iter().enumerate() {
            // Deserialize with permissive defaults so partial/odd entries survive.
            let id = raw
                .get("id")
                .and_then(|v| v.as_u64())
                .map(|n| n.to_string())
                .or_else(|| {
                    raw.get("id")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                })
                .unwrap_or_else(|| format!("cb-entry-{idx}"));

            let keys = raw
                .get("keys")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let secondary_keys = raw
                .get("secondary_keys")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|k| k.as_str().map(|s| s.to_string()))
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let content = raw
                .get("content")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let comment = raw
                .get("comment")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();

            let position = raw
                .get("position")
                .and_then(|v| v.as_str())
                .map(|s| match s {
                    "before_char" => LorebookPosition::BeforeChar,
                    "after_char" => LorebookPosition::AfterChar,
                    _ => LorebookPosition::AtDepth,
                })
                .unwrap_or(LorebookPosition::AtDepth);

            let depth = raw
                .get("depth")
                .and_then(|v| v.as_i64())
                .map(|n| n as i32)
                .unwrap_or(4);

            let activation_state = raw
                .get("constant")
                .and_then(|v| v.as_bool())
                .map(|c| {
                    if c {
                        LorebookActivation::Always
                    } else {
                        LorebookActivation::CharacterOnly
                    }
                })
                .unwrap_or(LorebookActivation::CharacterOnly);

            let enabled = raw
                .get("disable")
                .and_then(|v| v.as_bool())
                .map(|d| !d)
                .unwrap_or(true);

            let sort_order = raw
                .get("order")
                .and_then(|v| v.as_i64())
                .map(|n| n as i32)
                .unwrap_or(idx as i32);

            let token_budget = raw
                .get("token_budget")
                .and_then(|v| v.as_i64())
                .filter(|n| *n > 0);

            result.push(LorebookEntry {
                id: format!("{}-{}", book_name, id),
                lorebook_id: book_name.clone(),
                keys,
                secondary_keys,
                content,
                position,
                depth,
                activation_state,
                // Embedded character books use keyword activation (their
                // entries have no vectors in the embedding index).
                activation_mode: Default::default(),
                semantic_threshold: 0.6,
                token_budget,
                sort_order,
                enabled,
                extensions: raw
                    .get("extensions")
                    .cloned()
                    .unwrap_or(serde_json::json!({})),
                created_at: now,
                updated_at: now,
            });
            // `comment` is intentionally not stored on LorebookEntry (no field);
            // entry_name in activation details falls back to the first key.
            let _ = comment;
        }
        result
    }
}

/// Wrapper for the Character Card V2 spec format used in PNG tEXt chunks.
/// The spec format nests character data under a "data" key:
/// `{ "spec": "chara_card_v2", "spec_version": "2.0", "data": { ... } }`
#[derive(Clone, Debug, Deserialize)]
pub struct CharaCardV2Spec {
    #[serde(default)]
    pub spec: String,
    #[serde(default)]
    pub spec_version: String,
    pub data: CharaCardV2SpecData,
}

/// The character data payload inside a CharaCardV2Spec.
/// Field names match the official V2 spec.
#[derive(Clone, Debug, Deserialize)]
pub struct CharaCardV2SpecData {
    #[serde(default)]
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
    #[serde(default)]
    pub creator: String,
    #[serde(default, rename = "character_version")]
    pub character_version: String,
    #[serde(default)]
    pub creator_notes: String,
    #[serde(default)]
    pub character_book: serde_json::Value,
    #[serde(default)]
    pub extensions: serde_json::Value,
}

impl CharaCardV2Spec {
    pub fn to_domain(&self) -> CharacterCardV2 {
        let now = Timestamp::now();
        CharacterCardV2 {
            id: Uuid::new_v4().to_string(),
            name: self.data.name.clone(),
            description: self.data.description.clone(),
            personality: self.data.personality.clone(),
            scenario: self.data.scenario.clone(),
            first_mes: self.data.first_mes.clone(),
            mes_example: self.data.mes_example.clone(),
            system_prompt: self.data.system_prompt.clone(),
            post_history_instructions: self.data.post_history_instructions.clone(),
            alternate_greetings: self.data.alternate_greetings.clone(),
            tags: self.data.tags.clone(),
            creator: if self.data.creator.is_empty() {
                "Unknown".to_string()
            } else {
                self.data.creator.clone()
            },
            character_version: if self.data.character_version.is_empty() {
                "2.0".to_string()
            } else {
                self.data.character_version.clone()
            },
            avatar_path: None,
            extensions: self.data.extensions.clone(),
            creator_notes: self.data.creator_notes.clone(),
            character_book: self.data.character_book.clone(),
            default_persona_id: None,
            default_preset_id: None,
            lorebook_scan_depth: 2,
            lorebook_recursive_depth: 2,
            created_at: now,
            updated_at: now,
        }
    }
}

/// CharacterV1 format for importing older character cards.
/// Maps old field names (char_name, char_persona, etc.) to the V2 equivalents.
#[derive(Clone, Debug, Deserialize)]
pub struct CharacterV1 {
    pub char_name: String,
    #[serde(default)]
    pub char_persona: String,
    #[serde(default)]
    pub char_greeting: String,
    #[serde(default)]
    pub world_scenario: String,
    #[serde(default)]
    pub example_dialogue: String,
    #[serde(default)]
    pub creator_notes: String,
    #[serde(default)]
    pub tags: String,
    #[serde(default)]
    pub creator: String,
    #[serde(default)]
    pub character_version: String,
}

impl CharacterV1 {
    pub fn to_v2(&self) -> CharacterCardV2 {
        let now = Timestamp::now();
        CharacterCardV2 {
            id: Uuid::new_v4().to_string(),
            name: self.char_name.clone(),
            description: String::new(),
            personality: self.char_persona.clone(),
            scenario: self.world_scenario.clone(),
            first_mes: self.char_greeting.clone(),
            mes_example: self.example_dialogue.clone(),
            system_prompt: String::new(),
            post_history_instructions: String::new(),
            alternate_greetings: Vec::new(),
            tags: if self.tags.is_empty() {
                Vec::new()
            } else {
                self.tags
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .filter(|s| !s.is_empty())
                    .collect()
            },
            creator: if self.creator.is_empty() {
                "Unknown".to_string()
            } else {
                self.creator.clone()
            },
            character_version: "2.0".to_string(),
            avatar_path: None,
            extensions: serde_json::json!({}),
            creator_notes: self.creator_notes.clone(),
            character_book: serde_json::json!({}),
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
        assert!(
            parsed.is_ok(),
            "id should be valid UUID v4, got: {}",
            character.id
        );

        // Name should match
        assert_eq!(character.name, "Test Character");

        // character_version defaults to "2.0"
        assert_eq!(character.character_version, "2.0");

        // Timestamps should be set
        let now = Timestamp::now();
        assert!(character.created_at <= now);
        assert!(character.updated_at <= now);

        // Optional fields should default correctly
        assert_eq!(character.creator, "Unknown");
        assert!(character.avatar_path.is_none());
        assert_eq!(character.extensions, serde_json::json!({}));
        assert!(character.creator_notes.is_empty());
        assert_eq!(character.character_book, serde_json::json!({}));
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

        assert_eq!(
            character, deserialized,
            "roundtrip should preserve all fields"
        );
    }

    #[test]
    fn test_defaults_on_partial_json() {
        // Partial JSON with only the required name field should deserialize with defaults
        let json = r#"{"id": "test-id", "name": "Partial", "created_at": "2026-01-01T00:00:00Z", "updated_at": "2026-01-01T00:00:00Z"}"#;
        let character: CharacterCardV2 =
            serde_json::from_str(json).expect("partial JSON should deserialize");

        assert_eq!(character.id, "test-id");
        assert_eq!(character.name, "Partial");
        assert!(character.description.is_empty());
        assert!(character.alternate_greetings.is_empty());
        assert!(character.tags.is_empty());
        assert_eq!(character.creator, "Unknown");
        assert_eq!(character.character_version, "2.0");
        assert!(character.avatar_path.is_none());
        assert_eq!(character.extensions, serde_json::json!({}));
        assert!(character.creator_notes.is_empty());
        assert_eq!(character.character_book, serde_json::json!({}));
    }

    #[test]
    fn test_lorebook_defaults() {
        let character = CharacterCardV2::new("Lorebook Test".to_string());
        assert_eq!(
            character.lorebook_scan_depth, 2,
            "lorebook_scan_depth should default to 2"
        );
        assert_eq!(
            character.lorebook_recursive_depth, 2,
            "lorebook_recursive_depth should default to 2"
        );
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

    #[test]
    fn test_character_book_entries_parses_array_entries() {
        let mut character = CharacterCardV2::new("Embedded Book".to_string());
        character.character_book = serde_json::json!({
            "name": "World Lore",
            "entries": [
                {
                    "id": 1,
                    "keys": ["dragon"],
                    "secondary_keys": ["fire"],
                    "content": "Dragons breathe fire.",
                    "comment": "Dragon entry",
                    "position": "after_char",
                    "depth": 4,
                    "constant": true,
                    "disable": false,
                    "order": 10,
                    "token_budget": 200
                },
                {
                    "id": 2,
                    "keys": ["ocean", "wave"],
                    "content": "The sea is vast.",
                    "position": "at_depth",
                    "depth": 2,
                    "constant": false
                }
            ]
        });

        let entries = character.character_book_entries();
        assert_eq!(entries.len(), 2, "both array entries should parse");

        let dragon = entries
            .iter()
            .find(|e| e.content.contains("Dragons"))
            .expect("dragon entry present");
        assert_eq!(dragon.keys, vec!["dragon".to_string()]);
        assert_eq!(dragon.secondary_keys, vec!["fire".to_string()]);
        assert_eq!(dragon.lorebook_id, "World Lore");
        assert!(dragon.enabled);
        assert_eq!(
            dragon.activation_state,
            crate::domain::lorebook::LorebookActivation::Always,
            "constant=true maps to Always"
        );
        assert_eq!(dragon.token_budget, Some(200));

        let ocean = entries
            .iter()
            .find(|e| e.content.contains("The sea"))
            .expect("ocean entry present");
        // Comma-style keys within a single string already parse as one group
        // at the matching layer; here they're stored as separate key strings.
        assert_eq!(ocean.keys.len(), 2);
        assert_eq!(
            ocean.activation_state,
            crate::domain::lorebook::LorebookActivation::CharacterOnly,
            "constant=false maps to CharacterOnly"
        );
    }

    #[test]
    fn test_character_book_entries_parses_object_entries() {
        let mut character = CharacterCardV2::new("Obj Book".to_string());
        character.character_book = serde_json::json!({
            "entries": {
                "0": {"keys": ["castle"], "content": "A tall castle."},
                "1": {"keys": ["forest"], "content": "A dark forest."}
            }
        });

        let entries = character.character_book_entries();
        assert_eq!(entries.len(), 2, "object-keyed entries should parse");
        assert!(entries.iter().any(|e| e.content.contains("castle")));
        assert!(entries.iter().any(|e| e.content.contains("forest")));
    }

    #[test]
    fn test_character_book_entries_empty_when_no_book() {
        let character = CharacterCardV2::new("No Book".to_string());
        assert!(character.character_book_entries().is_empty());
    }

    #[test]
    fn test_character_book_entries_skips_disabled() {
        let mut character = CharacterCardV2::new("Disabled".to_string());
        character.character_book = serde_json::json!({
            "entries": [
                {"keys": ["a"], "content": "active", "disable": false},
                {"keys": ["b"], "content": "inactive", "disable": true}
            ]
        });
        let entries = character.character_book_entries();
        assert_eq!(entries.len(), 2, "all entries parse");
        let active = entries.iter().find(|e| e.content == "active").unwrap();
        assert!(active.enabled);
        let inactive = entries.iter().find(|e| e.content == "inactive").unwrap();
        assert!(!inactive.enabled, "disable=true → enabled=false");
    }
}
