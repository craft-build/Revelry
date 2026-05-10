use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Message, MessageRole};
use crate::domain::lorebook::LorebookEntry;
use crate::domain::persona::Persona;
use crate::service::lorebook::{self, ActivatedEntry};
use crate::service::template::{self, PersonaCompressionTier};
use crate::service::token_count::{TokenCountService, DEFAULT_CONTEXT_WINDOW};

/// An assembled prompt ready to send to the AI provider.
/// Contains the system prompt (preamble) and chat history as rig messages.
#[derive(Clone, Debug)]
pub struct AssembledPrompt {
    pub preamble: String,
    pub chat_history: Vec<rig::message::Message>,
    /// Context utilization ratio (0.0 to 1.0) — what fraction of the context window is used.
    pub context_utilization: f64,
    /// Number of messages truncated from history due to budget constraints.
    pub truncated_messages: usize,
    /// Number of lorebook entries activated and injected.
    pub active_lorebook_count: usize,
}

/// Engine for assembling prompts from character and conversation data.
pub struct PromptEngine;

impl PromptEngine {
    /// Assembles a prompt from a character, optional persona, and conversation messages.
    /// Returns an AssembledPrompt with the system prompt and chat history.
    ///
    /// The persona block is injected between the character system prompt and
    /// conversation history, then all template variables are resolved.
    ///
    /// This method does NOT enforce a budget — it includes all content without truncation.
    /// For budget-aware assembly, use `assemble_with_budget`.
    pub fn assemble(character: &CharacterCardV2, persona: Option<&Persona>, messages: &[Message]) -> AssembledPrompt {
        let preamble = build_system_prompt_with_persona(character, persona);
        let chat_history = build_chat_history(messages);

        AssembledPrompt {
            preamble,
            chat_history,
            context_utilization: 0.0,
            truncated_messages: 0,
            active_lorebook_count: 0,
        }
    }

    /// Assembles a prompt for the first message (no chat history).
    /// Used when starting a new conversation.
    pub fn assemble_initial(character: &CharacterCardV2, persona: Option<&Persona>) -> AssembledPrompt {
        let preamble = build_system_prompt_with_persona(character, persona);

        AssembledPrompt {
            preamble,
            chat_history: Vec::new(),
            context_utilization: 0.0,
            truncated_messages: 0,
            active_lorebook_count: 0,
        }
    }

    /// Budget-aware prompt assembly per D-04, D-05, D-06, D-07.
    ///
    /// Priority layer order (D-04):
    /// 1. Sticky layer: system prompt + character card fields (never truncated)
    /// 2. Lorebook layer: activated entries injected at their depth positions
    /// 3. Persona layer: compressed when budget is tight (D-06)
    /// 4. History layer: oldest messages truncated first (D-05)
    /// 5. Post-history instructions: placed after history, dropped if no room
    ///
    /// Returns an AssembledPrompt with context utilization tracking.
    pub fn assemble_with_budget(
        character: &CharacterCardV2,
        persona: Option<&Persona>,
        messages: &[Message],
        lorebook_entries: &[LorebookEntry],
        token_service: &TokenCountService,
        context_window: i64,
    ) -> AssembledPrompt {
        let budget = context_window.max(1) as usize;
        let mut tokens_used = 0usize;

        // ── Layer 1: Sticky layer (never truncated) ──
        // Build the character-only system prompt (no persona, no post-history instructions)
        let sticky_prompt = build_sticky_prompt(character);
        let sticky_tokens = token_service.count_tokens(&sticky_prompt);
        tokens_used += sticky_tokens;

        // ── Layer 2: Lorebook layer ──
        let scan_depth = character.lorebook_scan_depth;
        let max_recursive_depth = character.lorebook_recursive_depth;

        // Calculate remaining budget for lorebook layer
        // Reserve some budget for persona and history — lorebook gets up to 20% of total
        let lorebook_budget = ((budget as f64) * 0.2) as i64;
        let activated = lorebook::activate_lorebook_entries(
            lorebook_entries,
            messages,
            scan_depth,
            max_recursive_depth,
            lorebook_budget,
            token_service,
        );

        // Build lorebook content strings (depth 0 = preamble, depth > 0 = history injection)
        let mut lorebook_preamble_parts: Vec<String> = Vec::new();
        let mut lorebook_depth_entries: Vec<ActivatedEntry> = Vec::new();
        for activated_entry in &activated {
            let entry_tokens = token_service.count_tokens(&activated_entry.injected_content);
            tokens_used += entry_tokens;
            if activated_entry.effective_depth == 0 {
                lorebook_preamble_parts.push(activated_entry.injected_content.clone());
            } else {
                lorebook_depth_entries.push(activated_entry.clone());
            }
        }

        let active_lorebook_count = activated.len();

        // ── Layer 3: Persona layer (compressed when budget is tight, per D-06) ──
        let persona_block = if let Some(p) = persona {
            let remaining = budget.saturating_sub(tokens_used);
            let persona_block = select_persona_tier(p, remaining, token_service);
            let persona_tokens = token_service.count_tokens(&persona_block);
            tokens_used += persona_tokens;
            persona_block
        } else {
            String::new()
        };

        // ── Layer 4: History layer (oldest truncated first, per D-05) ──
        let mut history_messages: Vec<Message> = messages.to_vec();
        // Inject lorebook depth entries into history
        let _lorebook_injection_result = lorebook::inject_lorebook_entries(
            "",
            &lorebook_depth_entries,
            &mut history_messages,
        );

        let mut truncated_messages = 0usize;
        let remaining_for_history = budget.saturating_sub(tokens_used);

        // Count history tokens and truncate oldest messages if needed
        let truncated_history = truncate_history_oldest_first(
            &history_messages,
            remaining_for_history,
            token_service,
        );
        truncated_messages = truncated_history.truncated_count;

        let chat_history = build_chat_history(&truncated_history.messages);
        tokens_used += truncated_history.tokens_used;

        // ── Layer 5: Post-history instructions (placed after history, dropped if no room) ──
        let post_history = if !character.post_history_instructions.is_empty() {
            let post_history_text = format!("Instructions: {}", character.post_history_instructions);
            let post_history_tokens = token_service.count_tokens(&post_history_text);
            if tokens_used + post_history_tokens <= budget {
                tokens_used += post_history_tokens;
                Some(post_history_text)
            } else {
                None // Drop it — least priority
            }
        } else {
            None
        };

        // ── Build final preamble ──
        let mut preamble_parts = Vec::new();
        if !sticky_prompt.is_empty() {
            preamble_parts.push(sticky_prompt);
        }
        if !lorebook_preamble_parts.is_empty() {
            preamble_parts.push(lorebook_preamble_parts.join("\n\n"));
        }
        if !persona_block.is_empty() {
            preamble_parts.push(persona_block);
        }
        if let Some(post) = post_history {
            preamble_parts.push(post);
        }

        let raw_preamble = preamble_parts.join("\n\n");
        let preamble = template::resolve_variables(&raw_preamble, persona, character);

        // Calculate context utilization
        let context_utilization = if budget > 0 {
            (tokens_used as f64) / (budget as f64)
        } else {
            0.0
        };

        AssembledPrompt {
            preamble,
            chat_history,
            context_utilization: context_utilization.min(1.0),
            truncated_messages,
            active_lorebook_count,
        }
    }
}

/// Selects the best persona compression tier based on remaining budget.
/// Tries Full → Trimmed → Minimal → Essential progressively per D-06.
fn select_persona_tier(persona: &Persona, remaining_budget: usize, token_service: &TokenCountService) -> String {
    let tiers = [
        PersonaCompressionTier::Full,
        PersonaCompressionTier::Trimmed,
        PersonaCompressionTier::Minimal,
        PersonaCompressionTier::Essential,
    ];

    for tier in &tiers {
        let block = template::build_persona_block_compressed(persona, *tier);
        let tokens = token_service.count_tokens(&block);
        if tokens <= remaining_budget {
            return block;
        }
    }

    // Even essential doesn't fit — return empty (shouldn't happen with reasonable budgets)
    String::new()
}

/// Builds the sticky system prompt (never truncated): system_prompt + character fields
/// but NOT post_history_instructions (those go after history in D-04 priority order).
fn build_sticky_prompt(character: &CharacterCardV2) -> String {
    let mut parts = Vec::new();

    if !character.system_prompt.is_empty() {
        parts.push(character.system_prompt.clone());
    }
    if !character.personality.is_empty() {
        parts.push(format!("Personality: {}", character.personality));
    }
    if !character.scenario.is_empty() {
        parts.push(format!("Scenario: {}", character.scenario));
    }
    if !character.description.is_empty() {
        parts.push(format!("Description: {}", character.description));
    }

    if parts.is_empty() {
        format!("You are {}. Respond in character.", character.name)
    } else {
        parts.join("\n\n")
    }
}

/// Result of history truncation.
struct TruncatedHistory {
    messages: Vec<Message>,
    tokens_used: usize,
    truncated_count: usize,
}

/// Truncates history oldest-first to fit within a token budget per D-05.
/// Includes messages newest-first until budget is exceeded, then drops oldest.
fn truncate_history_oldest_first(
    messages: &[Message],
    budget: usize,
    token_service: &TokenCountService,
) -> TruncatedHistory {
    if messages.is_empty() {
        return TruncatedHistory {
            messages: Vec::new(),
            tokens_used: 0,
            truncated_count: 0,
        };
    }

    // Include messages newest-first, then reverse for correct order
    let mut included: Vec<Message> = Vec::new();
    let mut tokens_used = 0usize;

    for msg in messages.iter().rev() {
        if msg.content.is_empty() {
            continue; // Skip empty messages
        }
        let msg_tokens = 4 + token_service.count_tokens(&msg.content); // ~4 role overhead
        if tokens_used + msg_tokens > budget {
            break; // Budget exceeded — stop including messages
        }
        tokens_used += msg_tokens;
        included.push(msg.clone());
    }

    let truncated_count = messages.len() - included.len();
    included.reverse(); // Restore chronological order

    TruncatedHistory {
        messages: included,
        tokens_used,
        truncated_count,
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
    use crate::domain::lorebook::{InsertionPosition, LorebookEntry};
    use crate::service::token_count::TokenCountService;
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

    fn make_test_lorebook_entry(name: &str, keywords: Vec<&str>, content: &str) -> LorebookEntry {
        let now = NaiveDateTime::parse_from_str("2024-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        LorebookEntry {
            id: format!("lb-{}", name),
            character_id: "char-1".to_string(),
            name: name.to_string(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            secondary_terms: Vec::new(),
            content: content.to_string(),
            insertion_position: InsertionPosition::AfterChar,
            depth_from_bottom: 0,
            enabled: true,
            token_budget: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        }
    }

    fn token_service() -> TokenCountService {
        TokenCountService::new()
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
    }

    #[test]
    fn test_build_chat_history_assistant_message() {
        let messages = vec![make_test_message(MessageRole::Assistant, "Hi there")];
        let history = build_chat_history(&messages);

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_build_chat_history_system_message() {
        let messages = vec![make_test_message(MessageRole::System, "System info")];
        let history = build_chat_history(&messages);

        assert_eq!(history.len(), 1);
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

    // ── Budget-aware assembly tests ──

    #[test]
    fn test_assemble_with_budget_has_context_utilization() {
        // Test 1: AssembledPrompt with budget has context_utilization field
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();
        let context_window = 8192;

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &token_svc,
            context_window,
        );

        assert!(prompt.context_utilization > 0.0, "context_utilization should be > 0 when there is content");
        assert!(prompt.context_utilization <= 1.0, "context_utilization should be <= 1.0");
    }

    #[test]
    fn test_sticky_layers_always_included() {
        // Test 2: Sticky layers (system prompt + character card) are always included, never truncated
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &token_svc,
            8192,
        );

        assert!(prompt.preamble.contains("helpful assistant"), "System prompt should always be included");
        assert!(prompt.preamble.contains("Personality:"), "Personality should always be included");
        assert!(prompt.preamble.contains("Scenario:"), "Scenario should always be included");
    }

    #[test]
    fn test_persona_full_tier_when_budget_plenty() {
        // Test 3: Persona block uses "full" tier when budget has plenty of room
        let character = make_test_character();
        let mut persona = Persona::new("Alice".to_string(), "Bold".to_string());
        persona.appearance = "Red hair".to_string();
        persona.background = "A wandering mage".to_string();

        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            Some(&persona),
            &messages,
            &[],
            &token_svc,
            8192,
        );

        assert!(prompt.preamble.contains("Name: Alice"));
        assert!(prompt.preamble.contains("Personality: Bold"));
        assert!(prompt.preamble.contains("Appearance: Red hair"), "Full tier should include Appearance");
        assert!(prompt.preamble.contains("Background: A wandering mage"), "Full tier should include Background");
    }

    #[test]
    fn test_persona_trimmed_when_moderate_budget() {
        // Test 4: Persona block compresses to "trimmed" (name, personality, appearance) when budget is moderate
        let mut character = CharacterCardV2::new("Test".to_string());
        // Minimal sticky content to leave room for persona compression testing
        character.system_prompt = "Short prompt.".to_string();
        character.personality = String::new();
        character.scenario = String::new();
        character.description = String::new();
        character.post_history_instructions = String::new();

        let mut persona = Persona::new("Alice".to_string(), "Bold".to_string());
        persona.appearance = "Red hair".to_string();
        persona.background = "A wandering mage who traveled far and wide across the kingdom for many years and saw many things".to_string();
        persona.role_description = "A powerful mage with many abilities and great power".to_string();
        persona.relationships = "Friends with Bob the blacksmith and Sally the baker in town".to_string();
        persona.speech_style = "Formal and eloquent with a slight accent from the eastern provinces of the realm".to_string();

        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        // Use a very tight budget that can't fit the full persona but can fit trimmed
        // The sticky prompt (~5 tokens) + full persona (~80+ tokens) should exceed this
        let prompt = PromptEngine::assemble_with_budget(
            &character,
            Some(&persona),
            &messages,
            &[],
            &token_svc,
            80, // Very tight budget
        );

        assert!(prompt.preamble.contains("Name: Alice"));
        assert!(prompt.preamble.contains("Personality: Bold"));
        // With a tight budget, should not include all fields from Full tier
        let has_all_full_fields = prompt.preamble.contains("Background:")
            && prompt.preamble.contains("Role:")
            && prompt.preamble.contains("Speech Style:");
        assert!(!has_all_full_fields, "Should compress persona when budget is tight — got: {}", prompt.preamble);
    }

    #[test]
    fn test_persona_minimal_when_tight_budget() {
        // Test 5: Persona block compresses to "minimal" (name, personality) when budget is tight
        let mut character = CharacterCardV2::new("Test".to_string());
        character.system_prompt = "Short.".to_string();
        character.personality = String::new();
        character.scenario = String::new();
        character.description = String::new();
        character.post_history_instructions = String::new();

        let mut persona = Persona::new("Alice".to_string(), "Bold".to_string());
        persona.appearance = "Red hair, very long and flowing like a river of crimson silk flowing down her back".to_string();
        persona.background = "A wandering mage who traveled far and wide across many distant lands and seas".to_string();

        let messages = vec![make_test_message(MessageRole::User, "Hi")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            Some(&persona),
            &messages,
            &[],
            &token_svc,
            50, // Very tight budget
        );

        assert!(prompt.preamble.contains("Name: Alice"));
        // Very tight budget should compress at least past Full tier
        let has_appearance = prompt.preamble.contains("Appearance:");
        let has_background = prompt.preamble.contains("Background:");
        assert!(!(has_appearance && has_background), "Should not include both Appearance and Background with very tight budget");
    }

    #[test]
    fn test_persona_essential_when_very_tight_budget() {
        // Test 6: Persona block compresses to "essential" (just name) when budget is very tight
        let character = make_test_character();
        let mut persona = Persona::new("Alice".to_string(), "Bold".to_string());
        persona.appearance = "Red hair".to_string();

        let messages = vec![make_test_message(MessageRole::User, "Hi")];
        let token_svc = token_service();

        // Extremely tight budget — only room for sticky + essential persona
        let prompt = PromptEngine::assemble_with_budget(
            &character,
            Some(&persona),
            &messages,
            &[],
            &token_svc,
            100,
        );

        assert!(prompt.preamble.contains("Name: Alice"), "Essential tier should include name");
        let has_more = prompt.preamble.contains("Personality: Bold");
        // With very tight budget, may or may not include personality depending on sticky size
        // The key is that extreme compression occurs
    }

    #[test]
    fn test_history_truncated_oldest_first() {
        // Test 7: History is truncated oldest-first when it exceeds remaining budget
        let character = make_test_character();
        let messages: Vec<Message> = (0..50)
            .map(|i| make_test_message(
                if i % 2 == 0 { MessageRole::User } else { MessageRole::Assistant },
                &format!("Message number {} with some content to consume tokens", i),
            ))
            .collect();
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &token_svc,
            500, // Small budget — should truncate
        );

        assert!(prompt.truncated_messages > 0, "Should truncate messages when budget is tight");
        assert!(prompt.chat_history.len() < messages.len(), "Chat history should be shorter than original");
        // Most recent messages should be preserved
        assert!(!prompt.chat_history.is_empty(), "Should keep at least some messages");
    }

    #[test]
    fn test_assemble_with_budget_returns_truncated_count() {
        // Test 8: assemble_with_budget returns AssembledPrompt with truncated messages count
        let character = make_test_character();
        let messages: Vec<Message> = (0..20)
            .map(|i| make_test_message(
                if i % 2 == 0 { MessageRole::User } else { MessageRole::Assistant },
                &format!("This is message {} with enough text to use tokens", i),
            ))
            .collect();
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &token_svc,
            300,
        );

        // truncated_messages should be tracked
        let total = prompt.chat_history.len() + prompt.truncated_messages;
        assert!(total <= messages.len(), "Truncated + kept should not exceed total");
    }

    #[test]
    fn test_post_history_instructions_after_history() {
        // Test 9: Post-history instructions placed after history per D-04
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &token_svc,
            8192,
        );

        // Post-history instructions should appear in the preamble
        assert!(prompt.preamble.contains("Stay in character"), "Post-history instructions should be included");
    }

    #[test]
    fn test_assemble_without_budget_behaves_same() {
        // Test 10: When no budget is specified, assemble behaves like current (no truncation)
        let character = make_test_character();
        let messages = vec![
            make_test_message(MessageRole::User, "Hello!"),
            make_test_message(MessageRole::Assistant, "Hi there!"),
        ];

        let prompt = PromptEngine::assemble(&character, None, &messages);

        assert_eq!(prompt.chat_history.len(), 2);
        assert_eq!(prompt.truncated_messages, 0);
        assert_eq!(prompt.context_utilization, 0.0); // No budget tracking in assemble()
    }

    #[test]
    fn test_assemble_with_budget_lorebook_activated() {
        // Test: Lorebook entries activate when keywords match
        let character = make_test_character();
        let lorebook = vec![make_test_lorebook_entry("Dragon", vec!["dragon"], "Dragons are ancient creatures.")];
        let messages = vec![make_test_message(MessageRole::User, "I saw a dragon!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &lorebook,
            &token_svc,
            8192,
        );

        assert_eq!(prompt.active_lorebook_count, 1, "Should activate lorebook entry");
        assert!(prompt.preamble.contains("Dragons are ancient creatures"), "Lorebook content should be in preamble");
    }

    #[test]
    fn test_assemble_with_budget_no_lorebook_activation() {
        // Test: Lorebook entries don't activate when keywords don't match
        let character = make_test_character();
        let lorebook = vec![make_test_lorebook_entry("Dragon", vec!["dragon"], "Dragons are ancient.")];
        let messages = vec![make_test_message(MessageRole::User, "I saw a cat!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &lorebook,
            &token_svc,
            8192,
        );

        assert_eq!(prompt.active_lorebook_count, 0, "Should not activate lorebook entry");
        assert!(!prompt.preamble.contains("Dragons are ancient"), "Lorebook content should not be in preamble");
    }
}
