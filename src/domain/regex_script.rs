use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Regex script domain model — per D-04, a per-character find-and-replace script.
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
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
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
    pub fn new(
        character_id: String,
        name: String,
        find_pattern: String,
        replace_pattern: String,
    ) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            character_id,
            name,
            find_pattern,
            replace_pattern,
            flags: "gi".to_string(),
            sort_order: 0,
            enabled: true,
            created_at: now,
            updated_at: now,
        }
    }

    /// Creates RegexScript instances from SillyTavern extensions.regex_scripts JSON array.
    /// Each element in the array should have: scriptName, findRegex, replaceString, 
    /// placement (ignored), disabled (mapped to enabled), sortOrder.
    pub fn from_extensions_json(character_id: &str, json: &serde_json::Value) -> Vec<RegexScript> {
        let arr = match json.as_array() {
            Some(a) => a,
            None => return Vec::new(),
        };

        arr.iter().filter_map(|item| {
            let name = item.get("scriptName")?.as_str()?.to_string();
            let find_pattern = item.get("findRegex")?.as_str()?.to_string();
            let replace_pattern = item.get("replaceString").and_then(|v| v.as_str()).unwrap_or("").to_string();
            let disabled = item.get("disabled")?.as_bool().unwrap_or(false);
            let sort_order = item.get("sortOrder")?.as_i64().unwrap_or(0) as i32;

            let now = chrono::Local::now().naive_local();
            Some(RegexScript {
                id: Uuid::new_v4().to_string(),
                character_id: character_id.to_string(),
                name,
                find_pattern,
                replace_pattern,
                flags: "gi".to_string(),
                sort_order,
                enabled: !disabled,
                created_at: now,
                updated_at: now,
            })
        }).collect()
    }

    /// Converts this script to a serde_json::Value for SillyTavern extensions.regex_scripts.
    pub fn to_extensions_json(&self) -> serde_json::Value {
        serde_json::json!({
            "scriptName": self.name,
            "findRegex": self.find_pattern,
            "replaceString": self.replace_pattern,
            "placement": 0,
            "disabled": !self.enabled,
            "sortOrder": self.sort_order,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_generates_uuid_and_defaults() {
        let script = RegexScript::new(
            "char-1".to_string(),
            "Test Script".to_string(),
            "hello".to_string(),
            "world".to_string(),
        );

        assert!(!script.id.is_empty());
        let parsed = Uuid::parse_str(&script.id);
        assert!(parsed.is_ok(), "id should be valid UUID v4, got: {}", script.id);

        assert_eq!(script.character_id, "char-1");
        assert_eq!(script.name, "Test Script");
        assert_eq!(script.find_pattern, "hello");
        assert_eq!(script.replace_pattern, "world");
        assert_eq!(script.flags, "gi");
        assert_eq!(script.sort_order, 0);
        assert!(script.enabled);

        let now = chrono::Local::now().naive_local();
        assert!(script.created_at <= now);
        assert!(script.updated_at <= now);
    }

    #[test]
    fn test_from_extensions_json() {
        let json = serde_json::json!([
            {
                "scriptName": "Remove OOC",
                "findRegex": "\\(.*?\\)",
                "replaceString": "",
                "placement": 0,
                "disabled": false,
                "sortOrder": 1
            },
            {
                "scriptName": "Replace Name",
                "findRegex": "Alice",
                "replaceString": "Bob",
                "placement": 0,
                "disabled": true,
                "sortOrder": 2
            }
        ]);

        let scripts = RegexScript::from_extensions_json("char-1", &json);
        assert_eq!(scripts.len(), 2);

        assert_eq!(scripts[0].name, "Remove OOC");
        assert_eq!(scripts[0].find_pattern, "\\(.*?\\)");
        assert_eq!(scripts[0].replace_pattern, "");
        assert!(scripts[0].enabled);
        assert_eq!(scripts[0].sort_order, 1);

        assert_eq!(scripts[1].name, "Replace Name");
        assert!(!scripts[1].enabled); // disabled: true → enabled: false
        assert_eq!(scripts[1].sort_order, 2);
    }

    #[test]
    fn test_to_extensions_json() {
        let script = RegexScript::new(
            "char-1".to_string(),
            "Test".to_string(),
            "pattern".to_string(),
            "replacement".to_string(),
        );

        let json = script.to_extensions_json();
        assert_eq!(json["scriptName"], "Test");
        assert_eq!(json["findRegex"], "pattern");
        assert_eq!(json["replaceString"], "replacement");
        assert_eq!(json["disabled"], false);
        assert_eq!(json["sortOrder"], 0);
    }

    #[test]
    fn test_from_extensions_json_invalid_input() {
        let json = serde_json::json!("not an array");
        let scripts = RegexScript::from_extensions_json("char-1", &json);
        assert!(scripts.is_empty());

        let json2 = serde_json::json!([{"missingFields": true}]);
        let scripts2 = RegexScript::from_extensions_json("char-1", &json2);
        assert!(scripts2.is_empty()); // Missing required fields → filtered out
    }

    #[test]
    fn test_serde_roundtrip() {
        let script = RegexScript::new(
            "char-1".to_string(),
            "Roundtrip Test".to_string(),
            "(\\w+)".to_string(),
            "[$1]".to_string(),
        );

        let json = serde_json::to_string(&script).expect("serialize failed");
        let deserialized: RegexScript =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(script, deserialized, "roundtrip should preserve all fields");
    }
}
