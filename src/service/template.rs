use crate::domain::character::CharacterCardV2;
use crate::domain::persona::Persona;

/// Resolves template variables in a string using the provided persona and character.
///
/// Supported variables:
/// - {{user}} → persona.name (or "User" if no persona)
/// - {{char}} → character.name
/// - {{persona.name}} → persona.name
/// - {{persona.appearance}} → persona.appearance
/// - {{persona.personality}} → persona.personality
/// - {{persona.background}} → persona.background
/// - {{persona.role_description}} → persona.role_description
/// - {{persona.relationships}} → persona.relationships
/// - {{persona.speech_style}} → persona.speech_style
///
/// Unknown variables are left untouched (no replacement occurs).
/// Variables referencing empty fields resolve to empty strings.
pub fn resolve_variables(
    text: &str,
    persona: Option<&Persona>,
    character: &CharacterCardV2,
) -> String {
    let mut result = text.to_string();

    // Resolve {{user}} → persona name or "User"
    let user_name = persona.map(|p| p.name.as_str()).unwrap_or("User");
    result = result.replace("{{user}}", user_name);

    // Resolve {{char}} → character name
    result = result.replace("{{char}}", &character.name);

    // Resolve {{persona.*}} variables
    if let Some(p) = persona {
        result = result.replace("{{persona.name}}", &p.name);
        result = result.replace("{{persona.appearance}}", &p.appearance);
        result = result.replace("{{persona.personality}}", &p.personality);
        result = result.replace("{{persona.background}}", &p.background);
        result = result.replace("{{persona.role_description}}", &p.role_description);
        result = result.replace("{{persona.relationships}}", &p.relationships);
        result = result.replace("{{persona.speech_style}}", &p.speech_style);
    }

    result
}

/// Builds the persona injection block for the AI prompt.
///
/// Format:
///   You are interacting with {{user}}, who is:
///   Name: Alice
///   Personality: Bold and charismatic
///   Appearance: Red hair
///   Background: A wandering mage
///   Role: Mage
///   Relationships: Friend of Bob
///   Speech Style: Formal
///
/// Only non-empty fields are included. Empty fields are omitted entirely.
/// {{user}} in the header is left as a raw template variable — it will be
/// resolved later by PromptEngine::assemble().
pub fn build_persona_block(persona: &Persona) -> String {
    let mut lines = Vec::new();

    // Header line (raw {{user}} — resolved later by PromptEngine)
    lines.push("You are interacting with {{user}}, who is:".to_string());

    // Field order: name, personality, appearance, background, role_description, relationships, speech_style
    let fields: Vec<(&str, &str)> = vec![
        ("Name", &persona.name),
        ("Personality", &persona.personality),
        ("Appearance", &persona.appearance),
        ("Background", &persona.background),
        ("Role", &persona.role_description),
        ("Relationships", &persona.relationships),
        ("Speech Style", &persona.speech_style),
    ];

    for (label, value) in fields {
        if !value.is_empty() {
            lines.push(format!("{}: {}", label, value));
        }
    }

    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::character::CharacterCardV2;
    use crate::domain::persona::Persona;

    fn make_test_persona(name: &str, personality: &str) -> Persona {
        Persona::new(name.to_string(), personality.to_string())
    }

    fn make_test_character(name: &str) -> CharacterCardV2 {
        CharacterCardV2::new(name.to_string())
    }

    #[test]
    fn test_resolve_user_with_persona() {
        let persona = make_test_persona("Alice", "Friendly");
        let character = make_test_character("Eve");
        let result = resolve_variables("Hello {{user}}!", Some(&persona), &character);
        assert_eq!(result, "Hello Alice!");
    }

    #[test]
    fn test_resolve_user_without_persona() {
        let character = make_test_character("Eve");
        let result = resolve_variables("Hello {{user}}!", None, &character);
        assert_eq!(result, "Hello User!");
    }

    #[test]
    fn test_resolve_char() {
        let persona = make_test_persona("Alice", "Friendly");
        let character = make_test_character("Eve");
        let result = resolve_variables("{{char}} greets you.", Some(&persona), &character);
        assert_eq!(result, "Eve greets you.");
    }

    #[test]
    fn test_resolve_persona_name() {
        let persona = make_test_persona("Alice", "Friendly");
        let character = make_test_character("Eve");
        let result = resolve_variables("{{persona.name}} is here", Some(&persona), &character);
        assert_eq!(result, "Alice is here");
    }

    #[test]
    fn test_resolve_persona_appearance() {
        let mut persona = make_test_persona("Alice", "Friendly");
        persona.appearance = "Red hair".to_string();
        let character = make_test_character("Eve");
        let result = resolve_variables("{{persona.appearance}}", Some(&persona), &character);
        assert_eq!(result, "Red hair");
    }

    #[test]
    fn test_resolve_persona_empty_field() {
        let persona = make_test_persona("Alice", "Friendly");
        let character = make_test_character("Eve");
        // Background is empty by default
        let result = resolve_variables("{{persona.background}}", Some(&persona), &character);
        assert_eq!(result, "", "Empty field should resolve to empty string");
    }

    #[test]
    fn test_resolve_persona_role_description() {
        let mut persona = make_test_persona("Alice", "Friendly");
        persona.role_description = "Mage".to_string();
        let character = make_test_character("Eve");
        let result = resolve_variables("{{persona.role_description}}", Some(&persona), &character);
        assert_eq!(result, "Mage");
    }

    #[test]
    fn test_resolve_persona_relationships() {
        let mut persona = make_test_persona("Alice", "Friendly");
        persona.relationships = "Friend of Bob".to_string();
        let character = make_test_character("Eve");
        let result = resolve_variables("{{persona.relationships}}", Some(&persona), &character);
        assert_eq!(result, "Friend of Bob");
    }

    #[test]
    fn test_resolve_persona_speech_style() {
        let mut persona = make_test_persona("Alice", "Friendly");
        persona.speech_style = "Formal".to_string();
        let character = make_test_character("Eve");
        let result = resolve_variables("{{persona.speech_style}}", Some(&persona), &character);
        assert_eq!(result, "Formal");
    }

    #[test]
    fn test_resolve_mixed_variables() {
        let mut persona = make_test_persona("Alice", "Friendly");
        persona.appearance = "Red hair".to_string();
        let character = make_test_character("Eve");
        let result = resolve_variables(
            "{{user}} and {{char}}: {{persona.name}} has {{persona.appearance}}",
            Some(&persona),
            &character,
        );
        assert_eq!(result, "Alice and Eve: Alice has Red hair");
    }

    #[test]
    fn test_resolve_unknown_variable_unchanged() {
        let persona = make_test_persona("Alice", "Friendly");
        let character = make_test_character("Eve");
        let result = resolve_variables("{{unknown}} stays", Some(&persona), &character);
        assert_eq!(result, "{{unknown}} stays");
    }

    #[test]
    fn test_build_persona_block_full() {
        let mut persona = make_test_persona("Alice", "Bold and charismatic");
        persona.appearance = "Red hair, blue eyes".to_string();
        persona.background = "A wandering mage".to_string();
        persona.role_description = "Mage".to_string();
        persona.relationships = "Friend of Bob".to_string();
        persona.speech_style = "Formal".to_string();

        let block = build_persona_block(&persona);
        assert!(block.contains("You are interacting with {{user}}, who is:"));
        assert!(block.contains("Name: Alice"));
        assert!(block.contains("Personality: Bold and charismatic"));
        assert!(block.contains("Appearance: Red hair, blue eyes"));
        assert!(block.contains("Background: A wandering mage"));
        assert!(block.contains("Role: Mage"));
        assert!(block.contains("Relationships: Friend of Bob"));
        assert!(block.contains("Speech Style: Formal"));
    }

    #[test]
    fn test_build_persona_block_omits_empty_fields() {
        let persona = make_test_persona("Alice", "Bold");
        // Only name and personality are set

        let block = build_persona_block(&persona);
        assert!(block.contains("Name: Alice"));
        assert!(block.contains("Personality: Bold"));
        assert!(!block.contains("Appearance:"));
        assert!(!block.contains("Background:"));
        assert!(!block.contains("Role:"));
        assert!(!block.contains("Relationships:"));
        assert!(!block.contains("Speech Style:"));
    }

    #[test]
    fn test_build_persona_block_header_raw_user() {
        let persona = make_test_persona("Alice", "Friendly");
        let block = build_persona_block(&persona);
        assert!(block.contains("You are interacting with {{user}}, who is:"));
        // {{user}} should not be resolved in build_persona_block — that's done later
    }
}
