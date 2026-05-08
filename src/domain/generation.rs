use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generation preset domain model — per D-01, contains generation parameters only.
/// Built-in presets (Creative, Balanced, Precise) are read-only per D-14.
/// Fixed UUIDs ensure stability across import/export per research.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct GenerationPreset {
    pub id: String,
    pub name: String,
    /// True for built-in presets that cannot be edited or deleted (D-14).
    pub is_builtin: bool,
    pub temperature: f64,
    pub top_p: f64,
    /// 0 = no limit
    pub max_tokens: i64,
    pub frequency_penalty: f64,
    pub presence_penalty: f64,
    #[serde(default)]
    pub stop_sequences: Vec<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

impl GenerationPreset {
    /// Creates a new generation preset with a generated UUID v4 id, current timestamps,
    /// and default parameter values (temperature 0.8, top_p 0.9, no limits/penalties).
    pub fn new(name: String) -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: Uuid::new_v4().to_string(),
            name,
            is_builtin: false,
            temperature: 0.8,
            top_p: 0.9,
            max_tokens: 0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Built-in Creative preset per D-12: temp 1.2, top_p 0.9, penalties 0, no stop sequences.
    pub fn creative() -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: "builtin-creative".to_string(),
            name: "Creative".to_string(),
            is_builtin: true,
            temperature: 1.2,
            top_p: 0.9,
            max_tokens: 0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Built-in Balanced preset per D-12: temp 0.8, top_p 0.9, penalties 0, no stop sequences.
    pub fn balanced() -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: "builtin-balanced".to_string(),
            name: "Balanced".to_string(),
            is_builtin: true,
            temperature: 0.8,
            top_p: 0.9,
            max_tokens: 0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    /// Built-in Precise preset per D-12: temp 0.3, top_p 0.75, penalties 0, no stop sequences.
    pub fn precise() -> Self {
        let now = chrono::Local::now().naive_local();
        Self {
            id: "builtin-precise".to_string(),
            name: "Precise".to_string(),
            is_builtin: true,
            temperature: 0.3,
            top_p: 0.75,
            max_tokens: 0,
            frequency_penalty: 0.0,
            presence_penalty: 0.0,
            stop_sequences: Vec::new(),
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
        let preset = GenerationPreset::new("Custom".to_string());

        assert!(!preset.id.is_empty());
        let parsed = Uuid::parse_str(&preset.id);
        assert!(parsed.is_ok(), "id should be valid UUID v4, got: {}", preset.id);

        assert_eq!(preset.name, "Custom");
        assert!(!preset.is_builtin);
        assert_eq!(preset.temperature, 0.8);
        assert_eq!(preset.top_p, 0.9);
        assert_eq!(preset.max_tokens, 0);
        assert_eq!(preset.frequency_penalty, 0.0);
        assert_eq!(preset.presence_penalty, 0.0);
        assert!(preset.stop_sequences.is_empty());

        let now = chrono::Local::now().naive_local();
        assert!(preset.created_at <= now);
        assert!(preset.updated_at <= now);
    }

    #[test]
    fn test_creative_preset_values() {
        let preset = GenerationPreset::creative();
        assert_eq!(preset.id, "builtin-creative");
        assert_eq!(preset.name, "Creative");
        assert!(preset.is_builtin);
        assert_eq!(preset.temperature, 1.2);
        assert_eq!(preset.top_p, 0.9);
        assert!(preset.stop_sequences.is_empty());
    }

    #[test]
    fn test_balanced_preset_values() {
        let preset = GenerationPreset::balanced();
        assert_eq!(preset.id, "builtin-balanced");
        assert_eq!(preset.name, "Balanced");
        assert!(preset.is_builtin);
        assert_eq!(preset.temperature, 0.8);
        assert_eq!(preset.top_p, 0.9);
    }

    #[test]
    fn test_precise_preset_values() {
        let preset = GenerationPreset::precise();
        assert_eq!(preset.id, "builtin-precise");
        assert_eq!(preset.name, "Precise");
        assert!(preset.is_builtin);
        assert_eq!(preset.temperature, 0.3);
        assert_eq!(preset.top_p, 0.75);
    }

    #[test]
    fn test_serde_roundtrip() {
        let mut preset = GenerationPreset::new("Roundtrip".to_string());
        preset.stop_sequences = vec!["\\n".to_string(), "STOP".to_string()];
        preset.temperature = 1.5;
        preset.frequency_penalty = 0.3;

        let json = serde_json::to_string(&preset).expect("serialize failed");
        let deserialized: GenerationPreset =
            serde_json::from_str(&json).expect("deserialize failed");

        assert_eq!(preset, deserialized, "roundtrip should preserve all fields");
    }
}
