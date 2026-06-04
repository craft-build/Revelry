use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Regex script domain model — per-character find-and-replace script.
/// Scripts run in sort_order sequence (pipe chain) per D-05.
/// ReDoS validation is enforced at save time per D-06.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct RegexScript {
    pub id: String,
    /// FK to the character this script belongs to.
    pub character_id: String,
    /// Display name for the script.
    pub name: String,
    /// The regex pattern to search for.
    pub find_pattern: String,
    /// The replacement string (supports $1 capture group syntax).
    pub replace_pattern: String,
    /// Regex flags: i=case_insensitive, g=global, m=multi_line, s=dot_matches_new_line.
    #[serde(default = "default_flags")]
    pub flags: String,
    /// Execution order (lower runs first).
    #[serde(default)]
    pub sort_order: i32,
    /// Activation toggle — disabled scripts are skipped at runtime.
    #[serde(default = "default_enabled")]
    pub enabled: bool,
    pub created_at: Timestamp,
    pub updated_at: Timestamp,
}

fn default_flags() -> String {
    "gi".to_string()
}

fn default_enabled() -> bool {
    true
}

impl RegexScript {
    /// Creates a new regex script with a generated UUID v4 id, current timestamps,
    /// and defaults (flags: "gi", sort_order: 0, enabled: true).
    pub fn new(character_id: String, name: String) -> Self {
        let now = Timestamp::now();
        Self {
            id: Uuid::new_v4().to_string(),
            character_id,
            name,
            find_pattern: String::new(),
            replace_pattern: String::new(),
            flags: "gi".to_string(),
            sort_order: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Deserialize from SillyTavern extensions JSON (card import compat).
    /// Mirrors gtk_ref `from_extensions_json` — extracts regex entries from
    /// the `data.extensions.regex_scripts` array in a character card.
    pub fn from_extensions_json(value: &serde_json::Value, character_id: &str) -> Vec<Self> {
        value
            .get("regex_scripts")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|entry| {
                        Some(Self {
                            id: entry
                                .get("id")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&Uuid::new_v4().to_string())
                                .to_string(),
                            character_id: character_id.to_string(),
                            name: entry
                                .get("scriptName")
                                .and_then(|v| v.as_str())
                                .unwrap_or("Unnamed")
                                .to_string(),
                            find_pattern: entry
                                .get("findRegex")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            replace_pattern: entry
                                .get("replaceString")
                                .and_then(|v| v.as_str())
                                .unwrap_or("")
                                .to_string(),
                            flags: entry
                                .get("flags")
                                .and_then(|v| v.as_str())
                                .unwrap_or("gi")
                                .to_string(),
                            sort_order: entry
                                .get("sort_order")
                                .and_then(|v| v.as_i64())
                                .unwrap_or(0) as i32,
                            enabled: entry
                                .get("enabled")
                                .and_then(|v| v.as_bool())
                                .unwrap_or(true),
                            created_at: Timestamp::now(),
                            updated_at: Timestamp::now(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Serialize to SillyTavern extensions JSON format (card export compat).
    /// Mirrors gtk_ref `to_extensions_json` — produces the `regex_scripts` array
    /// that goes inside `data.extensions`.
    pub fn to_extensions_json(scripts: &[Self]) -> serde_json::Value {
        serde_json::Value::Array(
            scripts
                .iter()
                .map(|s| {
                    serde_json::json!({
                        "id": s.id,
                        "scriptName": s.name,
                        "findRegex": s.find_pattern,
                        "replaceString": s.replace_pattern,
                        "flags": s.flags,
                        "sort_order": s.sort_order,
                        "enabled": s.enabled,
                    })
                })
                .collect(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_regex_script_new() {
        let script = RegexScript::new("char-1".to_string(), "Test Script".to_string());

        assert!(!script.id.is_empty());
        let parsed = Uuid::parse_str(&script.id);
        assert!(
            parsed.is_ok(),
            "id should be valid UUID v4, got: {}",
            script.id
        );

        assert_eq!(script.character_id, "char-1");
        assert_eq!(script.name, "Test Script");
        assert!(script.find_pattern.is_empty());
        assert!(script.replace_pattern.is_empty());
        assert_eq!(script.flags, "gi");
        assert_eq!(script.sort_order, 0);
        assert!(script.enabled);

        let now = Timestamp::now();
        assert!(script.created_at <= now);
        assert!(script.updated_at <= now);
    }

    #[test]
    fn test_regex_script_serde_roundtrip() {
        let script = RegexScript::new("char-1".to_string(), "Roundtrip Test".to_string());

        let json = serde_json::to_string(&script).expect("serialize failed");
        let deserialized: RegexScript = serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(script, deserialized, "roundtrip should preserve all fields");
    }

    #[test]
    fn test_from_extensions_json() {
        let json = serde_json::json!({
            "regex_scripts": [
                {
                    "id": "ext-1",
                    "scriptName": "Remove OOC",
                    "findRegex": "\\(.*?\\)",
                    "replaceString": "",
                    "flags": "gi",
                    "sort_order": 1,
                    "enabled": true,
                },
                {
                    "id": "ext-2",
                    "scriptName": "Replace Name",
                    "findRegex": "Alice",
                    "replaceString": "Bob",
                    "flags": "g",
                    "sort_order": 2,
                    "enabled": false,
                },
            ]
        });

        let scripts = RegexScript::from_extensions_json(&json, "char-1");
        assert_eq!(scripts.len(), 2);

        assert_eq!(scripts[0].id, "ext-1");
        assert_eq!(scripts[0].name, "Remove OOC");
        assert_eq!(scripts[0].find_pattern, "\\(.*?\\)");
        assert_eq!(scripts[0].replace_pattern, "");
        assert_eq!(scripts[0].flags, "gi");
        assert!(scripts[0].enabled);
        assert_eq!(scripts[0].sort_order, 1);

        assert_eq!(scripts[1].id, "ext-2");
        assert_eq!(scripts[1].name, "Replace Name");
        assert_eq!(scripts[1].find_pattern, "Alice");
        assert_eq!(scripts[1].replace_pattern, "Bob");
        assert!(!scripts[1].enabled);
        assert_eq!(scripts[1].sort_order, 2);
    }

    #[test]
    fn test_from_extensions_json_empty() {
        let json = serde_json::json!({"other_key": "value"});
        let scripts = RegexScript::from_extensions_json(&json, "char-1");
        assert!(
            scripts.is_empty(),
            "JSON without regex_scripts key should return empty Vec"
        );
    }

    #[test]
    fn test_to_extensions_json() {
        let scripts = vec![RegexScript::new("char-1".to_string(), "Test".to_string())];

        let json = RegexScript::to_extensions_json(&scripts);
        let arr = json.as_array().expect("should be array");
        assert_eq!(arr.len(), 1);
        let entry = &arr[0];
        assert_eq!(entry["scriptName"], "Test");
        assert!(
            entry.get("findRegex").is_some(),
            "should have findRegex key"
        );
        assert!(
            entry.get("replaceString").is_some(),
            "should have replaceString key"
        );
        // Should NOT have Rust field names
        assert!(
            entry.get("find_pattern").is_none(),
            "should not have Rust field name find_pattern"
        );
        assert!(
            entry.get("replace_pattern").is_none(),
            "should not have Rust field name replace_pattern"
        );
    }

    #[test]
    fn test_extensions_json_roundtrip() {
        let json = serde_json::json!({
            "regex_scripts": [
                {
                    "id": "rt-1",
                    "scriptName": "Script A",
                    "findRegex": "hello",
                    "replaceString": "world",
                    "flags": "gi",
                    "sort_order": 5,
                    "enabled": true,
                },
            ]
        });

        let scripts = RegexScript::from_extensions_json(&json, "char-1");
        let exported = RegexScript::to_extensions_json(&scripts);
        let reimported = RegexScript::from_extensions_json(
            &serde_json::json!({"regex_scripts": exported}),
            "char-1",
        );

        assert_eq!(reimported.len(), 1);
        assert_eq!(reimported[0].name, "Script A");
        assert_eq!(reimported[0].find_pattern, "hello");
        assert_eq!(reimported[0].replace_pattern, "world");
        assert_eq!(reimported[0].flags, "gi");
        assert_eq!(reimported[0].sort_order, 5);
        assert!(reimported[0].enabled);
        // ID is preserved in the roundtrip
        assert_eq!(reimported[0].id, "rt-1");
    }
}
