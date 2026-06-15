use anyhow::{anyhow, Result};
use serde::Deserialize;

use crate::domain::lorebook::{Lorebook, LorebookEntry, LorebookPosition};

/// Maximum allowed JSON payload size (10 MB).
const MAX_JSON_SIZE: usize = 10 * 1024 * 1024;

/// Resolve a CharacterBook V2 entry `position` field.
///
/// The field may be a descriptive string (`before_char`, `after_char`,
/// `at_depth`) — as used by SillyTavern's string form — or a numeric code:
///   0 = before_char, 1 = after_char, 4 = at_depth.
/// Positions 2/3 (relative to the author's note) and anything unrecognized
/// fall back to the app default (after_char).
fn parse_position(value: &serde_json::Value) -> LorebookPosition {
    match value {
        serde_json::Value::String(s) => s.parse().unwrap_or_default(),
        serde_json::Value::Number(n) => match n.as_i64() {
            Some(0) => LorebookPosition::BeforeChar,
            Some(1) => LorebookPosition::AfterChar,
            Some(4) => LorebookPosition::AtDepth,
            _ => LorebookPosition::default(),
        },
        _ => LorebookPosition::default(),
    }
}

/// A single entry in the CharacterBook V2 spec, as authored by SillyTavern
/// and compatible tools. Unknown/extra fields are tolerated and ignored.
#[derive(Deserialize)]
struct CharacterBookEntrySpec {
    #[serde(default)]
    keys: Vec<String>,
    #[serde(default)]
    secondary_keywords: Vec<String>,
    #[serde(default)]
    content: String,
    #[serde(default)]
    enabled: Option<bool>,
    #[serde(default)]
    insertion_order: Option<i32>,
    #[serde(default)]
    position: serde_json::Value,
    #[serde(default)]
    depth: Option<i32>,
    #[serde(default)]
    token_budget: Option<i64>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    comment: Option<String>,
}

/// The top-level CharacterBook V2 object.
#[derive(Deserialize)]
struct CharacterBookSpec {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    entries: Vec<CharacterBookEntrySpec>,
}

/// Import a lorebook from a CharacterBook V2 JSON string.
///
/// Accepts either a standalone CharacterBook object (top-level `entries`)
/// or a Character Card V2 wrapper whose `data.character_book` (or
/// `character_book`) field holds the book. Returns a fresh `Lorebook` (new
/// UUID + timestamps) and a list of `LorebookEntry` values keyed to that
/// lorebook, ready to be persisted. No I/O is performed.
pub fn import_from_json(json_str: &str) -> Result<(Lorebook, Vec<LorebookEntry>)> {
    if json_str.len() > MAX_JSON_SIZE {
        return Err(anyhow!(
            "JSON payload exceeds maximum size of {} bytes",
            MAX_JSON_SIZE
        ));
    }

    let value: serde_json::Value = serde_json::from_str(json_str)?;

    // The book may be standalone (top-level) or embedded inside a character
    // card under `data.character_book` or `character_book`.
    let book_value = value
        .get("data")
        .and_then(|d| d.get("character_book"))
        .or_else(|| value.get("character_book"))
        .unwrap_or(&value);

    let book: CharacterBookSpec = serde_json::from_value(book_value.clone())?;

    let name = if book.name.trim().is_empty() {
        "Imported Lorebook".to_string()
    } else {
        book.name
    };
    let mut lorebook = Lorebook::new(name);
    lorebook.description = book.description;

    let mut entries = Vec::with_capacity(book.entries.len());
    for spec in book.entries {
        let mut entry = LorebookEntry::new(lorebook.id.clone());
        entry.keys = spec.keys;
        entry.secondary_keys = spec.secondary_keywords;
        entry.content = spec.content;
        entry.position = parse_position(&spec.position);
        entry.depth = spec.depth.unwrap_or(4);
        entry.token_budget = spec.token_budget;
        entry.sort_order = spec.insertion_order.unwrap_or(0);
        entry.enabled = spec.enabled.unwrap_or(true);

        // Preserve human-facing metadata that has no dedicated field so it
        // survives a round-trip if the lorebook is later edited/exported.
        let mut ext = serde_json::Map::new();
        if let Some(n) = spec.name.filter(|n| !n.is_empty()) {
            ext.insert("name".to_string(), serde_json::Value::String(n));
        }
        if let Some(c) = spec.comment.filter(|c| !c.is_empty()) {
            ext.insert("comment".to_string(), serde_json::Value::String(c));
        }
        entry.extensions = serde_json::Value::Object(ext);

        entries.push(entry);
    }

    Ok((lorebook, entries))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::lorebook::LorebookPosition;

    #[test]
    fn parses_standalone_character_book() {
        let json = r#"{
            "name": "Test Book",
            "description": "A test",
            "entries": [
                {
                    "keys": ["alpha", "first"],
                    "content": "First entry content.",
                    "enabled": true,
                    "insertion_order": 10,
                    "position": "after_char",
                    "name": "Alpha",
                    "comment": "the first"
                },
                {
                    "keys": ["beta"],
                    "secondary_keywords": ["second"],
                    "content": "Second entry.",
                    "enabled": false,
                    "insertion_order": 5,
                    "position": "before_char",
                    "depth": 2
                }
            ]
        }"#;

        let (lorebook, entries) = import_from_json(json).expect("parse");
        assert_eq!(lorebook.name, "Test Book");
        assert_eq!(lorebook.description, "A test");
        assert_eq!(entries.len(), 2);

        assert_eq!(entries[0].keys, vec!["alpha", "first"]);
        assert_eq!(entries[0].content, "First entry content.");
        assert!(entries[0].enabled);
        assert_eq!(entries[0].sort_order, 10);
        assert_eq!(entries[0].position, LorebookPosition::AfterChar);
        assert_eq!(entries[0].lorebook_id, lorebook.id);
        assert_eq!(
            entries[0].extensions,
            serde_json::json!({"name": "Alpha", "comment": "the first"})
        );

        assert_eq!(entries[1].keys, vec!["beta"]);
        assert_eq!(entries[1].secondary_keys, vec!["second"]);
        assert!(!entries[1].enabled);
        assert_eq!(entries[1].sort_order, 5);
        assert_eq!(entries[1].position, LorebookPosition::BeforeChar);
        assert_eq!(entries[1].depth, 2);
        assert_eq!(entries[1].extensions, serde_json::json!({}));
    }

    #[test]
    fn parses_numeric_positions() {
        let json = r#"{
            "name": "Num",
            "entries": [
                { "keys": ["a"], "content": "x", "position": 0 },
                { "keys": ["b"], "content": "x", "position": 1 },
                { "keys": ["c"], "content": "x", "position": 4 },
                { "keys": ["d"], "content": "x", "position": 2 }
            ]
        }"#;
        let (_lb, entries) = import_from_json(json).expect("parse");
        assert_eq!(entries[0].position, LorebookPosition::BeforeChar);
        assert_eq!(entries[1].position, LorebookPosition::AfterChar);
        assert_eq!(entries[2].position, LorebookPosition::AtDepth);
        assert_eq!(entries[3].position, LorebookPosition::AfterChar);
    }

    #[test]
    fn parses_embedded_in_character_card() {
        let json = r#"{
            "spec": "chara_card_v2",
            "data": {
                "name": "Some Character",
                "character_book": {
                    "name": "Embedded Book",
                    "entries": [
                        { "keys": ["k"], "content": "c", "position": "at_depth", "depth": 3 }
                    ]
                }
            }
        }"#;
        let (lorebook, entries) = import_from_json(json).expect("parse");
        assert_eq!(lorebook.name, "Embedded Book");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].position, LorebookPosition::AtDepth);
        assert_eq!(entries[0].depth, 3);
    }

    #[test]
    fn defaults_for_missing_optional_fields() {
        let json = r#"{ "name": "Min", "entries": [ { "keys": ["k"], "content": "c" } ] }"#;
        let (_lb, entries) = import_from_json(json).expect("parse");
        assert!(entries[0].enabled, "enabled defaults to true");
        assert_eq!(entries[0].sort_order, 0);
        assert_eq!(entries[0].depth, 4);
        assert_eq!(entries[0].position, LorebookPosition::default());
    }

    #[test]
    fn uses_default_name_when_blank() {
        let json = r#"{ "entries": [] }"#;
        let (lorebook, entries) = import_from_json(json).expect("parse");
        assert_eq!(lorebook.name, "Imported Lorebook");
        assert!(entries.is_empty());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(import_from_json("not json").is_err());
    }

    #[test]
    fn rejects_oversized_payload() {
        let big = "x".repeat(MAX_JSON_SIZE + 1);
        assert!(import_from_json(&big).is_err());
    }
}
