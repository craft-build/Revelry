use crate::domain::character::CharacterCardV2;
use crate::domain::conversation::{Message, MessageRole};
use crate::domain::lorebook::{LorebookActivationDetail, LorebookEntry};
use crate::domain::persona::Persona;
use crate::service::lorebook::truncate_to_token_budget;
use crate::service::lorebook::SemanticContext;
use crate::service::template::{self, PersonaCompressionTier};
use crate::service::token_count::TokenCountService;
use crate::service::vector::{content_hash, cosine_similarity};

use std::collections::HashMap;

/// Fraction of the context window reserved for the rolling story summary
/// (memory layer). Only spent once history actually starts truncating.
const SUMMARY_BUDGET_FRACTION: f64 = 0.10;
/// Fraction of the context window reserved for semantic recall of
/// out-of-window messages.
const RECALL_BUDGET_FRACTION: f64 = 0.12;
/// Maximum number of out-of-window messages recalled per prompt.
const RECALL_MAX_MESSAGES: usize = 8;
/// Per-recalled-message snippet token cap.
const RECALL_SNIPPET_TOKENS: usize = 96;

/// A recalled snippet from a *different* conversation with the same
/// character (cross-conversation memory). Pre-ranked and hash-verified by
/// the builder; the prompt layer only budget-caps and formats them.
#[derive(Clone, Debug)]
pub struct ExternalMemory {
    pub similarity: f32,
    pub is_user: bool,
    pub content: String,
}

/// Inputs for the memory layer: the conversation's rolling summary plus the
/// vectors needed to recall out-of-window messages semantically. Everything
/// is optional-by-emptiness — a missing query vector or empty vector map
/// degrades to summary-only memory.
pub struct MemoryInput {
    /// Rolling conversation summary (empty = none maintained yet).
    pub summary: String,
    /// Highest message sort_order covered by the summary.
    pub summary_upto: i64,
    /// Query vector (embedding of the latest user message) for recall.
    pub query_vector: Option<Vec<f32>>,
    /// message_id → (content_hash, vector), current embedding model only.
    pub message_vectors: HashMap<String, (String, Vec<f32>)>,
    /// Semantically similar moments from other conversations with the same
    /// character, newest-context builder-verified (empty when the
    /// cross-conversation toggle is off or embeddings are unavailable).
    pub external_memories: Vec<ExternalMemory>,
}

impl MemoryInput {
    fn has_recall_inputs(&self) -> bool {
        self.query_vector.is_some() && !self.message_vectors.is_empty()
    }
}

/// What the memory layer actually injected, for status display.
#[derive(Clone, Debug, Default)]
pub struct MemoryResult {
    pub summary_injected: bool,
    pub recalled: Vec<RecalledMemory>,
    pub external_count: usize,
}

#[derive(Clone, Debug)]
pub struct RecalledMemory {
    pub message_id: String,
    pub similarity: f32,
    pub snippet: String,
}

/// An assembled prompt ready to send to the AI provider.
/// Contains the system prompt (preamble) and chat history as rig messages.
#[derive(Clone, Debug)]
pub struct AssembledPrompt {
    pub preamble: String,
    pub chat_history: Vec<rig::completion::Message>,
    /// Total tokens used in the assembled prompt (preamble + history + overhead).
    pub tokens_used: usize,
    /// Number of messages truncated from history due to budget constraints.
    pub truncated_messages: usize,
    /// Details of activated lorebook entries for the Complete chunk payload.
    pub lorebook_details: Vec<LorebookActivationDetail>,
    /// What the memory layer injected (Phase 11).
    pub memory: MemoryResult,
}

/// Engine for assembling prompts from character and conversation data.
pub struct PromptEngine;

impl PromptEngine {
    /// Budget-aware prompt assembly per D-04, D-05, D-06, D-07, with the
    /// Phase 11 memory layer (D-04.5).
    ///
    /// Priority layer order (D-04):
    /// 1. Sticky layer: system prompt + character card fields (never truncated)
    /// 2. Lorebook layer: activated entries injected at their depth positions
    /// 3. Memory layer: rolling summary + semantically recalled out-of-window
    ///    messages (only engaged once history truncates; see below)
    /// 4. Persona layer: compressed when budget is tight (D-06)
    /// 5. History layer: oldest messages truncated first (D-05)
    /// 6. Post-history instructions: placed after history, dropped if no room
    ///
    /// The memory layer runs as a second pass: the prompt is first assembled
    /// without memory; if (and only if) history actually truncated and memory
    /// inputs exist, it is re-assembled with memory budgets reserved up front.
    /// This keeps memory free of window waste on conversations that still fit.
    ///
    /// `lorebook_name_map` resolves lorebook ids → display names for the
    /// activation details (junction lorebooks + the embedded character book).
    /// `semantic` carries precomputed query/entry vectors (None = embeddings
    /// unavailable; every entry fails open to keyword matching).
    ///
    /// Returns an AssembledPrompt with context utilization tracking.
    #[allow(clippy::too_many_arguments)]
    pub fn assemble_with_budget(
        character: &CharacterCardV2,
        persona: Option<&Persona>,
        messages: &[Message],
        lorebook_entries: &[LorebookEntry],
        lorebook_name_map: &std::collections::HashMap<String, String>,
        token_service: &TokenCountService,
        context_window: i64,
        semantic: Option<&SemanticContext>,
        memory: Option<&MemoryInput>,
    ) -> AssembledPrompt {
        let mut assembled = Self::assemble_inner(
            character,
            persona,
            messages,
            lorebook_entries,
            lorebook_name_map,
            token_service,
            context_window,
            semantic,
            None,
        );

        // Memory summary/recall only matters once history outgrows the
        // window — but cross-conversation echoes apply from the very first
        // turn of a new conversation, before any local history exists to
        // truncate. The final clause skips the re-assembly pass when the
        // memory input has nothing to contribute at all.
        if let Some(mem) = memory {
            if (assembled.truncated_messages > 0 || !mem.external_memories.is_empty())
                && (!mem.summary.is_empty()
                    || mem.has_recall_inputs()
                    || !mem.external_memories.is_empty())
            {
                assembled = Self::assemble_inner(
                    character,
                    persona,
                    messages,
                    lorebook_entries,
                    lorebook_name_map,
                    token_service,
                    context_window,
                    semantic,
                    Some(mem),
                );
            }
        }

        assembled
    }

    #[allow(clippy::too_many_arguments)]
    fn assemble_inner(
        character: &CharacterCardV2,
        persona: Option<&Persona>,
        messages: &[Message],
        lorebook_entries: &[LorebookEntry],
        lorebook_name_map: &std::collections::HashMap<String, String>,
        token_service: &TokenCountService,
        context_window: i64,
        semantic: Option<&SemanticContext>,
        memory: Option<&MemoryInput>,
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
        let lorebook_budget_fraction = 0.25; // 25% of context window for lorebook
        let lorebook_token_budget = (budget as f64 * lorebook_budget_fraction) as i64;

        let activated_entries = crate::service::lorebook::activate_lorebook_entries(
            lorebook_entries,
            messages,
            scan_depth,
            max_recursive_depth,
            lorebook_token_budget,
            token_service,
            semantic,
        );

        // Count lorebook tokens
        let lorebook_tokens: usize = activated_entries
            .iter()
            .map(|e| token_service.count_tokens(&e.injected_content))
            .sum();
        tokens_used += lorebook_tokens;

        // ── Layer 3: Memory layer — summary half (recall reserves below) ──
        // Budgets are reserved before history truncation so the memory layer
        // never steals tokens from the window retroactively; unused reserved
        // recall budget is simply left unspent.
        let summary_budget = (budget as f64 * SUMMARY_BUDGET_FRACTION) as usize;
        let mut summary_block: Option<String> = None;
        if let Some(mem) = memory {
            if !mem.summary.is_empty() {
                let capped = truncate_to_token_budget(&mem.summary, summary_budget, token_service);
                if !capped.is_empty() {
                    tokens_used += token_service.count_tokens(&capped);
                    summary_block = Some(capped);
                }
            }
        }

        // ── Layer 4: Persona layer (compressed when budget is tight, per D-06) ──
        let persona_block = if let Some(p) = persona {
            let remaining = budget.saturating_sub(tokens_used);
            let persona_block = select_persona_tier(p, remaining, token_service);
            let persona_tokens = token_service.count_tokens(&persona_block);
            tokens_used += persona_tokens;
            persona_block
        } else {
            String::new()
        };

        // ── Layer 5: History layer (oldest truncated first, per D-05) ──
        // Truncate BEFORE injecting depth-based lorebook entries.
        // This prevents depth-injected system messages at old positions from
        // being silently removed by budget truncation. Depth offsets are
        // relative to the truncated history length (matching SillyTavern/gtk_ref).
        let history_messages: Vec<Message> = messages.to_vec();

        // Reserve the recall budget up front when recall inputs exist;
        // cross-conversation echoes share it, so they reserve it too even
        // in a fresh conversation with no local vectors. The actual
        // snippets are selected from the dropped messages after
        // truncation below.
        let recall_budget = if memory
            .map(|m| m.has_recall_inputs() || !m.external_memories.is_empty())
            .unwrap_or(false)
        {
            (budget as f64 * RECALL_BUDGET_FRACTION) as usize
        } else {
            0
        };
        let remaining_for_history = budget.saturating_sub(tokens_used + recall_budget);

        // Count history tokens and truncate oldest messages if needed
        let truncated_history =
            truncate_history_oldest_first(&history_messages, remaining_for_history, token_service);
        let truncated_messages = truncated_history.truncated_count;

        // ── Layer 3b: Memory layer — semantic recall half ──
        // Out-of-window messages whose vectors are close to the query get
        // pulled back in as attributed snippets. Stale vectors (content
        // edited since embedding) are skipped.
        let mut recalled_memories: Vec<RecalledMemory> = Vec::new();
        let mut recall_tokens_used = 0usize;
        let mut external_selected: Vec<&ExternalMemory> = Vec::new();
        if let Some(mem) = memory {
            if let Some(query) = &mem.query_vector {
                if !truncated_history.dropped.is_empty() {
                    let mut candidates: Vec<(f32, &Message)> = truncated_history
                        .dropped
                        .iter()
                        .filter(|m| !m.content.is_empty())
                        .filter_map(|m| {
                            let (hash, vector) = mem.message_vectors.get(&m.id)?;
                            if hash != &content_hash(&m.content) {
                                return None; // Stale vector — skip.
                            }
                            let similarity = cosine_similarity(query, vector);
                            Some((similarity, m))
                        })
                        .collect();
                    candidates
                        .sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

                    for (similarity, message) in candidates.into_iter().take(RECALL_MAX_MESSAGES) {
                        if recalled_memories.len() >= RECALL_MAX_MESSAGES {
                            break;
                        }
                        let snippet = truncate_to_token_budget(
                            &message.content,
                            RECALL_SNIPPET_TOKENS,
                            token_service,
                        );
                        let snippet_tokens = token_service.count_tokens(&snippet);
                        if recall_tokens_used + snippet_tokens > recall_budget {
                            break; // Budget exhausted.
                        }
                        recall_tokens_used += snippet_tokens;
                        recalled_memories.push(RecalledMemory {
                            message_id: message.id.clone(),
                            similarity,
                            snippet: message.content.chars().take(60).collect(),
                        });
                    }
                }
            }

            // Cross-conversation echoes: builder-ranked moments from other
            // conversations with this character, sharing the leftover budget.
            if !mem.external_memories.is_empty() {
                for external in &mem.external_memories {
                    if recall_tokens_used >= recall_budget {
                        break;
                    }
                    let snippet = truncate_to_token_budget(
                        &external.content,
                        RECALL_SNIPPET_TOKENS,
                        token_service,
                    );
                    let snippet_tokens = token_service.count_tokens(&snippet);
                    if recall_tokens_used + snippet_tokens > recall_budget {
                        break;
                    }
                    recall_tokens_used += snippet_tokens;
                    external_selected.push(external);
                }
            }
        }
        tokens_used += recall_tokens_used;
        let summary_injected = summary_block.is_some();
        let memory_result = MemoryResult {
            summary_injected,
            recalled: recalled_memories.clone(),
            external_count: external_selected.len(),
        };

        // Build chat history from truncated messages, then inject lorebook entries
        let mut chat_history =
            build_chat_history(&truncated_history.messages, persona, Some(character));
        tokens_used += truncated_history.tokens_used;

        // Apply lorebook injection via service function
        let (preamble_before, preamble_after) = crate::service::lorebook::inject_lorebook_entries(
            &sticky_prompt, // preamble_before_char (BeforeChar content appended before this)
            "", // preamble_after_char (AfterChar content appended after this; persona added separately)
            &activated_entries,
            &mut chat_history,
        );

        // ── Layer 6: Post-history instructions (placed after history, dropped if no room) ──
        let post_history = if !character.post_history_instructions.is_empty() {
            let post_history_text =
                format!("Instructions: {}", character.post_history_instructions);
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
        // Lorebook BeforeChar entries go before sticky prompt, AfterChar goes
        // after; memory sits between lorebook and persona.
        let mut preamble_parts = Vec::new();
        if !preamble_before.is_empty() {
            preamble_parts.push(preamble_before);
        }
        if !preamble_after.is_empty() {
            preamble_parts.push(preamble_after);
        }
        if let Some(summary) = summary_block {
            preamble_parts.push(format!("[Summary of the story so far]\n{summary}"));
        }
        if !recalled_memories.is_empty() {
            let lines: Vec<String> = recalled_memories
                .iter()
                .filter_map(|r| {
                    // Recover the dropped message for its role and full snippet.
                    truncated_history
                        .dropped
                        .iter()
                        .find(|m| m.id == r.message_id)
                        .and_then(|m| {
                            let speaker = match m.role {
                                MessageRole::User => "{{user}}",
                                MessageRole::Assistant => "{{char}}",
                                MessageRole::System => return None,
                            };
                            let snippet = truncate_to_token_budget(
                                &m.content,
                                RECALL_SNIPPET_TOKENS,
                                token_service,
                            );
                            Some(format!("- {speaker}: {snippet}"))
                        })
                })
                .collect();
            if !lines.is_empty() {
                preamble_parts.push(format!(
                    "[Moments from earlier that may be relevant]\n{}",
                    lines.join("\n")
                ));
            }
        }
        if !external_selected.is_empty() {
            let lines: Vec<String> = external_selected
                .iter()
                .map(|external| {
                    let speaker = if external.is_user {
                        "{{user}}"
                    } else {
                        "{{char}}"
                    };
                    let snippet = truncate_to_token_budget(
                        &external.content,
                        RECALL_SNIPPET_TOKENS,
                        token_service,
                    );
                    format!("- {speaker}: {snippet}")
                })
                .collect();
            preamble_parts.push(format!(
                "[Echoes from previous conversations with {{{{char}}}}]\n{}",
                lines.join("\n")
            ));
        }
        if !persona_block.is_empty() {
            preamble_parts.push(persona_block);
        }
        if let Some(post) = post_history {
            preamble_parts.push(post);
        }

        let raw_preamble = preamble_parts.join("\n\n");
        let preamble = template::resolve_variables(&raw_preamble, persona, character);

        // Build lorebook activation details for the Complete chunk.
        // The lorebook_name map (passed from ChatContext) resolves lorebook ids
        // to display names for both junction-table lorebooks and the embedded
        // character book.
        let lorebook_details: Vec<LorebookActivationDetail> = activated_entries
            .iter()
            .map(|e| LorebookActivationDetail {
                lorebook_name: lorebook_name_map
                    .get(&e.entry.lorebook_id)
                    .cloned()
                    .unwrap_or_else(|| e.entry.lorebook_id.clone()),
                entry_name: e.entry.keys.first().cloned().unwrap_or_default(),
                content_snippet: e.injected_content.chars().take(30).collect(),
                token_cost: token_service.count_tokens(&e.injected_content),
                matched_keys: Some(
                    e.entry
                        .keys
                        .iter()
                        .chain(e.entry.secondary_keys.iter())
                        .cloned()
                        .collect(),
                ),
                activation_reason: e.activation_reason.clone(),
                similarity: e.similarity,
            })
            .collect();

        AssembledPrompt {
            preamble,
            chat_history,
            tokens_used,
            truncated_messages,
            lorebook_details,
            memory: memory_result,
        }
    }
}

/// Selects the best persona compression tier based on remaining budget.
/// Tries Full → Trimmed → Minimal → Essential progressively per D-06.
fn select_persona_tier(
    persona: &Persona,
    remaining_budget: usize,
    token_service: &TokenCountService,
) -> String {
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

/// Formatting rules appended to every sticky prompt. Actions and thoughts in
/// `*asterisks*` render as muted italic narration in the chat view
/// (`.prose-markdown em`), so dialogue stays visually distinct.
const FORMATTING_INSTRUCTIONS: &str = "Formatting:\n\
- Wrap actions, descriptions, and inner thoughts in single asterisks, e.g. *She glances toward the door.*\n\
- Write spoken dialogue as plain text without asterisks.\n\
- Keep narration and dialogue as separate segments; never put spoken lines inside asterisks.";

/// Builds the sticky system prompt (never truncated): system_prompt + character fields
/// but NOT post_history_instructions (those go after history in D-04 priority order).
/// Includes `mes_example` as an example-dialogue block placed after the
/// scenario/description context, mirroring how character cards define voice/tone.
/// Always ends with the formatting section.
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
    if !character.mes_example.is_empty() {
        parts.push(format!("Example dialogue:\n{}", character.mes_example));
    }

    if parts.is_empty() {
        parts.push(format!("You are {}. Respond in character.", character.name));
    }
    parts.push(FORMATTING_INSTRUCTIONS.to_string());

    parts.join("\n\n")
}

/// Result of history truncation.
struct TruncatedHistory {
    messages: Vec<Message>,
    /// Messages excluded by the budget, oldest-first (memory recall input).
    dropped: Vec<Message>,
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
            dropped: Vec::new(),
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

    // Dropped = messages not included, in original (oldest-first) order.
    let included_ids: std::collections::HashSet<&str> =
        included.iter().map(|m| m.id.as_str()).collect();
    let dropped: Vec<Message> = messages
        .iter()
        .filter(|m| !included_ids.contains(m.id.as_str()))
        .cloned()
        .collect();

    TruncatedHistory {
        messages: included,
        dropped,
        tokens_used,
        truncated_count,
    }
}

/// Converts domain messages to rig messages.
/// System messages (including steering) have template variables resolved so that
/// `{{user}}` and `{{char}}` become actual names that the LLM can understand.
fn build_chat_history(
    messages: &[Message],
    persona: Option<&Persona>,
    character: Option<&CharacterCardV2>,
) -> Vec<rig::completion::Message> {
    messages
        .iter()
        .filter(|m| !m.content.is_empty()) // Skip empty messages
        .map(|msg| {
            let content = if matches!(msg.role, MessageRole::System) {
                if let Some(ch) = character {
                    template::resolve_variables(&msg.content, persona, ch)
                } else {
                    msg.content.clone()
                }
            } else {
                msg.content.clone()
            };
            match msg.role {
                MessageRole::User => rig::completion::Message::user(&content),
                MessageRole::Assistant => rig::completion::Message::assistant(&content),
                MessageRole::System => rig::completion::Message::system(&content),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::Message as DomainMessage;
    use crate::domain::persona::Persona;
    use crate::service::token_count::TokenCountService;
    use jiff::Timestamp;

    fn make_test_character() -> CharacterCardV2 {
        let mut character = CharacterCardV2::new("Test Character".to_string());
        character.system_prompt = "You are a helpful assistant.".to_string();
        character.personality = "Friendly and kind.".to_string();
        character.scenario = "A peaceful village.".to_string();
        character.description = "A helpful guide.".to_string();
        character.post_history_instructions = "Stay in character.".to_string();
        character
    }

    fn make_test_message(role: MessageRole, content: &str) -> DomainMessage {
        DomainMessage {
            id: "test-id".to_string(),
            conversation_id: "conv-1".to_string(),
            role,
            content: content.to_string(),
            sort_order: 0,
            timestamp: Timestamp::now(),
            branch_id: None,
        }
    }

    fn token_service() -> TokenCountService {
        TokenCountService::new()
    }

    #[test]
    fn test_build_chat_history_user_message() {
        let messages = vec![make_test_message(MessageRole::User, "Hello")];
        let history = build_chat_history(&messages, None, None);

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_build_chat_history_assistant_message() {
        let messages = vec![make_test_message(MessageRole::Assistant, "Hi there")];
        let history = build_chat_history(&messages, None, None);

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_build_chat_history_system_message() {
        let messages = vec![make_test_message(MessageRole::System, "System info")];
        let history = build_chat_history(&messages, None, None);

        assert_eq!(history.len(), 1);
    }

    #[test]
    fn test_build_chat_history_system_message_resolves_templates() {
        let character = make_test_character();
        let messages = vec![make_test_message(
            MessageRole::System,
            "Do not post for {{user}}",
        )];
        let history = build_chat_history(&messages, None, Some(&character));

        assert_eq!(history.len(), 1);
        // {{user}} should be resolved to "User" (no persona → default "User")
        let content = match &history[0] {
            rig::completion::Message::System { content } => content.clone(),
            _ => panic!("Expected system message"),
        };
        assert!(
            content.contains("User"),
            "Expected {{user}} to be resolved, got: {}",
            content
        );
        assert!(
            !content.contains("{{user}}"),
            "{{user}} should have been resolved, got: {}",
            content
        );
    }

    #[test]
    fn test_build_chat_history_filters_empty() {
        let messages = vec![
            make_test_message(MessageRole::User, "Hello"),
            make_test_message(MessageRole::Assistant, ""), // Empty
            make_test_message(MessageRole::User, "World"),
        ];
        let history = build_chat_history(&messages, None, None);

        assert_eq!(history.len(), 2); // Empty message filtered out
    }

    #[test]
    fn test_build_chat_history_multiple() {
        let messages = vec![
            make_test_message(MessageRole::User, "Question"),
            make_test_message(MessageRole::Assistant, "Answer"),
            make_test_message(MessageRole::User, "Follow up"),
        ];
        let history = build_chat_history(&messages, None, None);

        assert_eq!(history.len(), 3);
    }

    // ── Budget-aware assembly tests ──

    #[test]
    fn test_assemble_with_budget_tracks_tokens_used() {
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();
        let context_window = 8192;

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &std::collections::HashMap::new(),
            &token_svc,
            context_window,
            None,
            None,
        );

        assert!(
            prompt.tokens_used > 0,
            "tokens_used should be > 0 when there is content"
        );
    }

    #[test]
    fn test_sticky_layers_always_included() {
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &std::collections::HashMap::new(),
            &token_svc,
            8192,
            None,
            None,
        );

        assert!(
            prompt.preamble.contains("helpful assistant"),
            "System prompt should always be included"
        );
        assert!(
            prompt.preamble.contains("Personality:"),
            "Personality should always be included"
        );
        assert!(
            prompt.preamble.contains("Scenario:"),
            "Scenario should always be included"
        );
    }

    #[test]
    fn test_formatting_instructions_included() {
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        let mut empty_character = make_test_character();
        empty_character.system_prompt.clear();
        empty_character.personality.clear();
        empty_character.scenario.clear();
        empty_character.description.clear();
        empty_character.mes_example.clear();

        for character in [make_test_character(), empty_character] {
            let prompt = PromptEngine::assemble_with_budget(
                &character,
                None,
                &messages,
                &[],
                &std::collections::HashMap::new(),
                &token_svc,
                8192,
                None,
                None,
            );
            assert!(
                prompt.preamble.contains("Formatting:"),
                "Formatting section should always be included"
            );
        }
    }

    #[test]
    fn test_persona_full_tier_when_budget_plenty() {
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
            &std::collections::HashMap::new(),
            &token_svc,
            8192,
            None,
            None,
        );

        assert!(prompt.preamble.contains("Name: Alice"));
        assert!(prompt.preamble.contains("Personality: Bold"));
        assert!(
            prompt.preamble.contains("Appearance: Red hair"),
            "Full tier should include Appearance"
        );
        assert!(
            prompt.preamble.contains("Background: A wandering mage"),
            "Full tier should include Background"
        );
    }

    #[test]
    fn test_history_truncated_oldest_first() {
        let character = make_test_character();
        let messages: Vec<DomainMessage> = (0..50)
            .map(|i| {
                make_test_message(
                    if i % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    &format!("Message number {} with some content to consume tokens", i),
                )
            })
            .collect();
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &std::collections::HashMap::new(),
            &token_svc,
            500, // Small budget — should truncate
            None,
            None,
        );

        assert!(
            prompt.truncated_messages > 0,
            "Should truncate messages when budget is tight"
        );
        assert!(
            prompt.chat_history.len() < messages.len(),
            "Chat history should be shorter than original"
        );
        // Most recent messages should be preserved
        assert!(
            !prompt.chat_history.is_empty(),
            "Should keep at least some messages"
        );
    }

    #[test]
    fn test_assemble_with_budget_returns_truncated_count() {
        let character = make_test_character();
        let messages: Vec<DomainMessage> = (0..20)
            .map(|i| {
                make_test_message(
                    if i % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    &format!("This is message {} with enough text to use tokens", i),
                )
            })
            .collect();
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &std::collections::HashMap::new(),
            &token_svc,
            300,
            None,
            None,
        );

        // truncated_messages should be tracked
        let total = prompt.chat_history.len() + prompt.truncated_messages;
        assert!(
            total <= messages.len(),
            "Truncated + kept should not exceed total"
        );
    }

    #[test]
    fn test_post_history_instructions_after_history() {
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &std::collections::HashMap::new(),
            &token_svc,
            8192,
            None,
            None,
        );

        // Post-history instructions should appear in the preamble
        assert!(
            prompt.preamble.contains("Stay in character"),
            "Post-history instructions should be included"
        );
    }

    #[test]
    fn test_mes_example_injected_into_prompt() {
        let mut character = make_test_character();
        character.mes_example = "<START>\n{{user}}: Hi\n{{char}}: Hello there".to_string();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &std::collections::HashMap::new(),
            &token_svc,
            8192,
            None,
            None,
        );

        assert!(
            prompt.preamble.contains("Example dialogue:"),
            "mes_example should be injected as an example-dialogue block"
        );
        assert!(
            prompt.preamble.contains(": Hi"),
            "mes_example content should appear in the preamble ({{user}} resolved to the persona name)"
        );
    }

    // Phase 9: add lorebook activation tests

    #[test]
    fn test_assemble_with_budget_lorebook_entries_activated() {
        let mut character = make_test_character();
        character.lorebook_scan_depth = 2;
        character.lorebook_recursive_depth = 2;

        let mut entry = crate::domain::lorebook::LorebookEntry::new("lorebook-1".to_string());
        entry.keys = vec!["dragon".to_string()];
        entry.content = "Dragons are ancient creatures of immense power.".to_string();
        entry.position = crate::domain::lorebook::LorebookPosition::AfterChar;
        entry.enabled = true;

        let messages = vec![make_test_message(MessageRole::User, "I saw a dragon!")];
        let token_svc = token_service();

        let mut lorebook_names = std::collections::HashMap::new();
        lorebook_names.insert("lorebook-1".to_string(), "World Lore".to_string());
        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[entry.clone()],
            &lorebook_names,
            &token_svc,
            8192,
            None,
            None,
        );

        assert_eq!(
            prompt.lorebook_details.len(),
            1,
            "One lorebook entry should be activated when keyword matches"
        );
        assert!(
            prompt.preamble.contains("Dragons are ancient"),
            "Activated lorebook content should appear in the preamble"
        );
        assert_eq!(
            prompt.lorebook_details.len(),
            1,
            "lorebook_details should contain one entry"
        );
        assert_eq!(
            prompt.lorebook_details[0].entry_name, "dragon",
            "entry_name should be the first key"
        );
        assert_eq!(
            prompt.lorebook_details[0].lorebook_name, "World Lore",
            "lorebook_name should resolve from the name map"
        );
    }

    // ── Memory layer tests (Phase 11) ─────────────────────────────────────

    fn memory_with_summary(summary: &str) -> MemoryInput {
        MemoryInput {
            summary: summary.to_string(),
            summary_upto: 0,
            query_vector: None,
            message_vectors: HashMap::new(),
            external_memories: Vec::new(),
        }
    }

    #[test]
    fn test_summary_not_injected_without_truncation() {
        // Short conversation fits the window — memory must stay out even when
        // a summary exists (no waste, no duplication of visible history).
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let token_svc = token_service();
        let memory = memory_with_summary("Once upon a time, a dragon attacked.");

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &HashMap::new(),
            &token_svc,
            8192,
            None,
            Some(&memory),
        );

        assert_eq!(prompt.truncated_messages, 0);
        assert!(!prompt.memory.summary_injected);
        assert!(
            !prompt.preamble.contains("Story so far"),
            "summary must not inject when nothing truncated"
        );
    }

    #[test]
    fn test_summary_injected_once_history_truncates() {
        let character = make_test_character();
        let messages: Vec<DomainMessage> = (0..60)
            .map(|i| {
                make_test_message(
                    if i % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    &format!("Message number {i} with plenty of words to consume tokens quickly"),
                )
            })
            .collect();
        let token_svc = token_service();
        let memory = memory_with_summary("The hero met the dragon and they became allies.");

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &HashMap::new(),
            &token_svc,
            600, // Tiny window — forces truncation
            None,
            Some(&memory),
        );

        assert!(prompt.truncated_messages > 0, "history should truncate");
        assert!(
            prompt.memory.summary_injected,
            "summary should inject once history truncates"
        );
        assert!(prompt.preamble.contains("Summary of the story so far"));
        assert!(
            prompt.preamble.contains("dragon and they became allies"),
            "summary content should appear in the preamble"
        );
    }

    #[test]
    fn test_semantic_recall_pulls_relevant_dropped_messages() {
        let character = make_test_character();
        // Message 0 is relevant to the query vector; message 1 is not.
        let mut relevant = make_test_message(MessageRole::User, "We discovered the dragon crown.");
        relevant.id = "msg-relevant".to_string();
        let mut irrelevant = make_test_message(MessageRole::Assistant, "The weather is mild.");
        irrelevant.id = "msg-irrelevant".to_string();
        let filler: Vec<DomainMessage> = (0..60)
            .map(|i| {
                make_test_message(
                    if i % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    &format!("Padding message {i} consuming tokens to force truncation"),
                )
            })
            .collect();
        let mut messages = vec![relevant, irrelevant];
        messages.extend(filler);

        let query = vec![1.0, 0.0];
        let mut message_vectors = HashMap::new();
        message_vectors.insert(
            "msg-relevant".to_string(),
            (
                content_hash("We discovered the dragon crown."),
                vec![0.9, 0.1],
            ),
        );
        message_vectors.insert(
            "msg-irrelevant".to_string(),
            (content_hash("The weather is mild."), vec![0.0, 1.0]),
        );
        let memory = MemoryInput {
            summary: String::new(),
            summary_upto: 0,
            query_vector: Some(query),
            message_vectors,
            external_memories: Vec::new(),
        };

        let token_svc = token_service();
        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &HashMap::new(),
            &token_svc,
            700,
            None,
            Some(&memory),
        );

        assert!(prompt.truncated_messages > 0);
        assert!(!prompt.memory.recalled.is_empty(), "recall should fire");
        assert_eq!(
            prompt.memory.recalled[0].message_id, "msg-relevant",
            "the most similar dropped message is recalled first"
        );
        assert!(
            prompt.preamble.contains("dragon crown"),
            "recalled snippet content should appear in the preamble"
        );
        assert!(
            prompt.preamble.contains("Moments from earlier"),
            "recall block header should be present"
        );
    }

    #[test]
    fn test_recall_skips_stale_vectors() {
        let character = make_test_character();
        let mut edited = make_test_message(MessageRole::User, "Edited content.");
        edited.id = "msg-edited".to_string();
        let filler: Vec<DomainMessage> = (0..60)
            .map(|i| {
                make_test_message(
                    if i % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    &format!("Padding message {i} consuming tokens to force truncation"),
                )
            })
            .collect();
        let mut messages = vec![edited];
        messages.extend(filler);

        // Vector exists but its content_hash belongs to the OLD content.
        let mut message_vectors = HashMap::new();
        message_vectors.insert(
            "msg-edited".to_string(),
            (content_hash("Old content before the edit."), vec![1.0, 0.0]),
        );
        let memory = MemoryInput {
            summary: String::new(),
            summary_upto: 0,
            query_vector: Some(vec![1.0, 0.0]),
            message_vectors,
            external_memories: Vec::new(),
        };

        let token_svc = token_service();
        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &HashMap::new(),
            &token_svc,
            700,
            None,
            Some(&memory),
        );

        assert!(prompt.truncated_messages > 0);
        assert!(
            prompt.memory.recalled.is_empty(),
            "stale vectors must be skipped"
        );
    }

    #[test]
    fn test_external_memories_injected_as_echoes() {
        let character = make_test_character();
        let filler: Vec<DomainMessage> = (0..60)
            .map(|i| {
                make_test_message(
                    if i % 2 == 0 {
                        MessageRole::User
                    } else {
                        MessageRole::Assistant
                    },
                    &format!("Padding message {i} consuming tokens to force truncation"),
                )
            })
            .collect();
        // One in-conversation candidate plus one echo from another conversation.
        let mut dropped_hit =
            make_test_message(MessageRole::Assistant, "The dragon crown gleamed.");
        dropped_hit.id = "msg-in-conv".to_string();
        let mut messages = vec![dropped_hit];
        messages.extend(filler);

        let mut message_vectors = HashMap::new();
        message_vectors.insert(
            "msg-in-conv".to_string(),
            (content_hash("The dragon crown gleamed."), vec![1.0, 0.0]),
        );
        let memory = MemoryInput {
            summary: String::new(),
            summary_upto: 0,
            query_vector: Some(vec![1.0, 0.0]),
            message_vectors,
            external_memories: vec![ExternalMemory {
                similarity: 0.82,
                is_user: true,
                content: "In an older story, we forged an alliance with the dragon.".to_string(),
            }],
        };

        let token_svc = token_service();
        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &HashMap::new(),
            &token_svc,
            700,
            None,
            Some(&memory),
        );

        assert!(prompt.truncated_messages > 0);
        assert_eq!(prompt.memory.external_count, 1, "echo injected");
        assert!(
            prompt
                .preamble
                .contains("Echoes from previous conversations"),
            "echo block header present"
        );
        assert!(
            prompt
                .preamble
                .contains("forged an alliance with the dragon"),
            "echo content present"
        );
    }

    #[test]
    fn test_external_memories_injected_without_truncation() {
        // A fresh conversation: no local history to truncate, no query
        // vector, no summary — echoes from other conversations must still
        // inject (the feature's primary scenario).
        let character = make_test_character();
        let messages = vec![make_test_message(MessageRole::User, "Hello!")];
        let memory = MemoryInput {
            summary: String::new(),
            summary_upto: 0,
            query_vector: None,
            message_vectors: HashMap::new(),
            external_memories: vec![ExternalMemory {
                similarity: 0.9,
                is_user: false,
                content: "Old echo.".to_string(),
            }],
        };

        let prompt = PromptEngine::assemble_with_budget(
            &character,
            None,
            &messages,
            &[],
            &HashMap::new(),
            &token_service(),
            8192,
            None,
            Some(&memory),
        );

        assert_eq!(prompt.truncated_messages, 0);
        assert_eq!(
            prompt.memory.external_count, 1,
            "echo injects even without truncation"
        );
        assert!(
            prompt.preamble.contains("Old echo."),
            "echo content present in preamble"
        );
    }
}
