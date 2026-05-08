use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;
use uuid::Uuid;

/// Position where a lorebook entry is inserted relative to the character definition.
/// Per D-11, entries inject at depth_from_bottom in chat history,
/// but position relative to character definition is also tracked.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum InsertionPosition {
    BeforeChar,
    AfterChar,
}

impl fmt::Display for InsertionPosition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InsertionPosition::BeforeChar => write!(f, "before_char"),
            InsertionPosition::AfterChar => write!(f, "after_char"),
        }
    }
}

impl FromStr for InsertionPosition {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "before_char" => Ok(InsertionPosition::BeforeChar),
            "after_char" => Ok(InsertionPosition::AfterChar),
            _ => Err(format!("Unknown insertion position: {}", s)),
        }
    }
}

/// Lorebook entry domain model — per D-08, a keyword-activated text injection entry.
///
/// Each entry belongs to a character and is activated when its keywords match
/// recent messages. Scan depth is per-character (lorebook_scan_depth on CharacterCardV2),
/// NOT per-entry. Per D-10, activation_depth is removed from per-entry fields.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct LorebookEntry {
    pub id: String,
    /// FK to the character this entry belongs to.
    pub character_id: String,
    /// Display name for the entry.
    pub name: String,
    /// OR-matched keywords per D-09. Any keyword match activates the entry.
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Additional terms for broader matching (secondary conditions).
    #[serde(default)]
    pub secondary_terms: Vec<String>,
    /// The text to inject when activated.
    #[serde(default)]
    pub content: String,
    /// Position relative to character definition (before/after).
    #[serde(default = "default_insertion_position")]
    pub insertion_position: InsertionPosition,
    /// Depth-from-bottom insertion position per D-11.
    /// 0 = after last message, 1 = one message from end, etc.
    /// Default is 4 per UI-SPEC default.
    #[serde(default = "default_depth_from_bottom")]
    pub depth_from_bottom: i32,
    /// Activation toggle — disabled entries are skipped during activation.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    /// Per-entry soft token budget per D-13. None = unlimited.
    pub token_budget: Option<i64>,
    /// Display/ordering sort position.
    #[serde(default)]
    pub sort_order: i32,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

fn default_insertion_position() -> InsertionPosition {
    InsertionPosition::AfterChar
}

fn default_depth_from_bottom() -> i32 {
    4
}

fn default_enabled() -> bool {
    true
}

impl LorebookEntry {
    /// Creates a new lorebook entry with a generated UUID v4 id, current timestamps,
    /// and defaults (enabled: true, depth_from_bottom: 4, sort_order: 0).
    pub fn new(
        character_id: String,
        name: String,
        keywords: Vec<String>,
        content: String,
    ) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            character_id,
            name,
            keywords,
            secondary_terms: Vec::new(),
            content,
            insertion_position: InsertionPosition::AfterChar,
            depth_from_bottom: 4,
            enabled: true,
            token_budget: None,
            sort_order: 0,
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
        let entry = LorebookEntry::new(
            "char-1".to_string(),
            "Test Entry".to_string(),
            vec!["keyword1".to_string()],
            "Some content".to_string(),
        );

        // UUID v4 id should be valid and non-empty
        assert!(!entry.id.is_empty());
        let parsed = Uuid::parse_str(&entry.id);
        assert!(parsed.is_ok(), "id should be valid UUID v4, got: {}", entry.id);

        // Character ID should match
        assert_eq!(entry.character_id, "char-1");
        assert_eq!(entry.name, "Test Entry");
        assert_eq!(entry.keywords, vec!["keyword1"]);
        assert_eq!(entry.content, "Some content");

        // Defaults
        assert!(entry.enabled);
        assert_eq!(entry.depth_from_bottom, 4);
        assert_eq!(entry.sort_order, 0);
        assert!(entry.secondary_terms.is_empty());
        assert!(entry.token_budget.is_none());
        assert_eq!(entry.insertion_position, InsertionPosition::AfterChar);

        // Timestamps should be set
        let now = chrono::Local::now().naive_local();
        assert!(entry.created_at <= now);
        assert!(entry.updated_at <= now);
    }

    #[test]
    fn test_lorebook_entry_fields_match_spec() {
        let mut entry = LorebookEntry::new(
            "char-1".to_string(),
            "Spec Entry".to_string(),
            vec!["kw1".to_string(), "kw2".to_string()],
            "Content text".to_string(),
        );
        entry.secondary_terms = vec!["sec1".to_string()];
        entry.insertion_position = InsertionPosition::BeforeChar;
        entry.depth_from_bottom = 2;
        entry.enabled = false;
        entry.token_budget = Some(500);
        entry.sort_order = 3;

        assert_eq!(entry.name, "Spec Entry");
        assert_eq!(entry.keywords, vec!["kw1", "kw2"]);
        assert_eq!(entry.secondary_terms, vec!["sec1"]);
        assert_eq!(entry.content, "Content text");
        assert_eq!(entry.insertion_position, InsertionPosition::BeforeChar);
        assert_eq!(entry.depth_from_bottom, 2);
        assert!(!entry.enabled);
        assert_eq!(entry.token_budget, Some(500));
        assert_eq!(entry.sort_order, 3);
    }

    #[test]
    fn test_insertion_position_display() {
        assert_eq!(InsertionPosition::BeforeChar.to_string(), "before_char");
        assert_eq!(InsertionPosition::AfterChar.to_string(), "after_char");
    }

    #[test]
    fn test_insertion_position_from_str() {
        assert_eq!(
            InsertionPosition::from_str("before_char").unwrap(),
            InsertionPosition::BeforeChar
        );
        assert_eq!(
            InsertionPosition::from_str("after_char").unwrap(),
            InsertionPosition::AfterChar
        );
        assert_eq!(
            InsertionPosition::from_str("BEFORE_CHAR").unwrap(),
            InsertionPosition::BeforeChar
        );
        assert!(InsertionPosition::from_str("invalid").is_err());
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut entry = LorebookEntry::new(
            "char-1".to_string(),
            "Roundtrip Test".to_string(),
            vec!["keyword".to_string()],
            "Content".to_string(),
        );
        entry.secondary_terms = vec!["sec".to_string()];
        entry.insertion_position = InsertionPosition::BeforeChar;
        entry.token_budget = Some(100);

        let json = serde_json::to_string(&entry).expect("serialize failed");
        let deserialized: LorebookEntry =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(entry, deserialized, "roundtrip should preserve all fields");
    }
}
