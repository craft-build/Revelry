use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Message, MessageRole};
use crate::domain::persona::Persona;
use crate::service::template;

/// An assembled prompt ready to send to the AI provider.
/// Contains the system prompt (preamble) and chat history as rig messages.
#[derive(Clone, Debug)]
pub struct AssembledPrompt {
    pub preamble: String,
    pub chat_history: Vec<rig::message::Message>,
}

/// Engine for assembling prompts from character and conversation data.
pub struct PromptEngine;

impl PromptEngine {
    /// Assembles a prompt from a character, optional persona, and conversation messages.
    /// Returns an AssembledPrompt with the system prompt and chat history.
    ///
    /// The persona block is injected between the character system prompt and
    /// conversation history, then all template variables are resolved.
    pub fn assemble(character: &CharacterCardV2, persona: Option<&Persona>, messages: &[Message]) -> AssembledPrompt {
        let preamble = build_system_prompt_with_persona(character, persona);
        let chat_history = build_chat_history(messages);

        AssembledPrompt {
            preamble,
            chat_history,
        }
    }

    /// Assembles a prompt for the first message (no chat history).
    /// Used when starting a new conversation.
    pub fn assemble_initial(character: &CharacterCardV2, persona: Option<&Persona>) -> AssembledPrompt {
        let preamble = build_system_prompt_with_persona(character, persona);

        AssembledPrompt {
            preamble,
            chat_history: Vec::new(),
        }
    }
}

/// Builds the system prompt from character data, optionally including a persona block.
/// Template variables ({{user}}, {{char}}, {{persona.*}}) are resolved in the final output.
fn build_system_prompt_with_persona(character: &CharacterCardV2, persona: Option<&Persona>) -> String {
    let mut parts = Vec::new();

    // 1. Character system prompt
    let character_prompt = build_system_prompt(character);
    if !character_prompt.is_empty() {
        parts.push(character_prompt);
    }

    // 2. Persona block (injected between character and history)
    if let Some(p) = persona {
        let persona_block = template::build_persona_block(p);
        if !persona_block.is_empty() {
            parts.push(persona_block);
        }
    }

    // Join parts
    let raw_preamble = parts.join("\n\n");

    // Resolve all template variables
    template::resolve_variables(&raw_preamble, persona, character)
}

/// Builds the character-only system prompt (no persona, no template resolution).
fn build_system_prompt(character: &CharacterCardV2) -> String {
    let mut parts = Vec::new();

    // Use the character's system_prompt if available
    if !character.system_prompt.is_empty() {
        parts.push(character.system_prompt.clone());
    }

    // Add personality context
    if !character.personality.is_empty() {
        parts.push(format!("Personality: {}", character.personality));
    }

    // Add scenario context
    if !character.scenario.is_empty() {
        parts.push(format!("Scenario: {}", character.scenario));
    }

    // Add description context
    if !character.description.is_empty() {
        parts.push(format!("Description: {}", character.description));
    }

    // Add post-history instructions if available
    if !character.post_history_instructions.is_empty() {
        parts.push(format!(
            "Instructions: {}",
            character.post_history_instructions
        ));
    }

    if parts.is_empty() {
        // Fallback: use the character name as minimal context
        format!(
            "You are {}. Respond in character.",
            character.name
        )
    } else {
        parts.join("\n\n")
    }
}

/// Converts domain messages to rig messages.
fn build_chat_history(messages: &[Message]) -> Vec<rig::message::Message> {
    messages
        .iter()
        .filter(|m| !m.content.is_empty()) // Skip empty messages
        .map(|msg| match msg.role {
            MessageRole::User => {
                rig::message::Message::user(&msg.content)
            }
            MessageRole::Assistant => {
                rig::message::Message::assistant(&msg.content)
            }
            MessageRole::System => {
                rig::message::Message::system(&msg.content)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::Message;
    use crate::domain::persona::Persona;
    use chrono::NaiveDateTime;

    fn make_test_character() -> CharacterCardV2 {
        let mut character = CharacterCardV2::new("Test Character".to_string());
        character.system_prompt = "You are a helpful assistant.".to_string();
        character.personality = "Friendly and kind.".to_string();
        character.scenario = "A peaceful village.".to_string();
        character.description = "A helpful guide.".to_string();
        character.post_history_instructions = "Stay in character.".to_string();
        character
    }

    fn make_test_message(role: MessageRole, content: &str) -> Message {
        Message {
            id: "test-id".to_string(),
            conversation_id: "conv-1".to_string(),
            branch_id: None,
            role,
            content: content.to_string(),
            timestamp: NaiveDateTime::parse_from_str(
                "2024-01-01T00:00:00",
                "%Y-%m-%dT%H:%M:%S",
            )
            .unwrap(),
            sort_order: 0,
        }
    }

    #[test]
    fn test_assemble_initial() {
        let character = make_test_character();
        let prompt = PromptEngine::assemble_initial(&character, None);

        assert!(!prompt.preamble.is_empty());
        assert!(prompt.preamble.contains("helpful assistant"));
        assert!(prompt.chat_history.is_empty());
    }

    #[test]
    fn test_assemble_with_messages() {
        let character = make_test_character();
        let messages = vec![
            make_test_message(MessageRole::User, "Hello!"),
            make_test_message(MessageRole::Assistant, "Hi there!"),
        ];

        let prompt = PromptEngine::assemble(&character, None, &messages);

        assert!(!prompt.preamble.is_empty());
        assert_eq!(prompt.chat_history.len(), 2);
    }

    #[test]
    fn test_assemble_initial_with_persona() {
        let character = make_test_character();
        let persona = Persona::new("Alice".to_string(), "Bold and charismatic".to_string());
        let prompt = PromptEngine::assemble_initial(&character, Some(&persona));

        assert!(prompt.preamble.contains("You are interacting with Alice, who is:"));
        assert!(prompt.preamble.contains("Name: Alice"));
        assert!(prompt.preamble.contains("Personality: Bold and charismatic"));
    }

    #[test]
    fn test_assemble_with_persona_resolves_user_in_character_fields() {
        let mut character = make_test_character();
        character.system_prompt = "You are {{user}}'s assistant.".to_string();
        let persona = Persona::new("Alice".to_string(), "Friendly".to_string());
        let messages = vec![
            make_test_message(MessageRole::User, "Hello!"),
        ];

        let prompt = PromptEngine::assemble(&character, Some(&persona), &messages);

        // {{user}} in the character's system_prompt should be resolved
        assert!(prompt.preamble.contains("You are Alice's assistant."), "Expected Alice in system prompt, got: {}", prompt.preamble);
    }

    #[test]
    fn test_build_system_prompt_full() {
        let character = make_test_character();
        let preamble = build_system_prompt(&character);

        assert!(preamble.contains("helpful assistant"));
        assert!(preamble.contains("Personality:"));
        assert!(preamble.contains("Scenario:"));
        assert!(preamble.contains("Description:"));
        assert!(preamble.contains("Instructions:"));
    }

    #[test]
    fn test_build_system_prompt_minimal() {
        let mut character = CharacterCardV2::new("Minimal".to_string());
        character.system_prompt = String::new();
        character.personality = String::new();
        character.scenario = String::new();
        character.description = String::new();
        character.post_history_instructions = String::new();

        let preamble = build_system_prompt(&character);

        assert!(preamble.contains("Minimal"));
        assert!(preamble.contains("Respond in character"));
    }

    #[test]
    fn test_build_chat_history_user_message() {
        let messages = vec![make_test_message(MessageRole::User, "Hello")];
        let history = build_chat_history(&messages);

        assert_eq!(history.len(), 1);
        // Verify message was created (rig internal structure not tested)
    }

    #[test]
    fn test_build_chat_history_assistant_message() {
        let messages = vec![make_test_message(MessageRole::Assistant, "Hi there")];
        let history = build_chat_history(&messages);

        assert_eq!(history.len(), 1);
        // Verify message was created (rig internal structure not tested)
    }

    #[test]
    fn test_build_chat_history_system_message() {
        let messages = vec![make_test_message(MessageRole::System, "System info")];
        let history = build_chat_history(&messages);

        assert_eq!(history.len(), 1);
        // Verify message was created (rig internal structure not tested)
    }

    #[test]
    fn test_build_chat_history_filters_empty() {
        let messages = vec![
            make_test_message(MessageRole::User, "Hello"),
            make_test_message(MessageRole::Assistant, ""), // Empty
            make_test_message(MessageRole::User, "World"),
        ];
        let history = build_chat_history(&messages);

        assert_eq!(history.len(), 2); // Empty message filtered out
    }

    #[test]
    fn test_build_chat_history_multiple() {
        let messages = vec![
            make_test_message(MessageRole::User, "Question"),
            make_test_message(MessageRole::Assistant, "Answer"),
            make_test_message(MessageRole::User, "Follow up"),
        ];
        let history = build_chat_history(&messages);

        assert_eq!(history.len(), 3);
    }
}
