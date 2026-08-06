use crate::domain::conversation::{Message, MessageRole};
use crate::domain::lorebook::{LorebookActivation, LorebookEntry, LorebookPosition};
use crate::service::token_count::TokenCountService;

/// A lorebook entry that has been activated and is ready for injection into the prompt.
#[derive(Clone, Debug)]
pub struct ActivatedEntry {
    /// The original lorebook entry.
    pub entry: LorebookEntry,
    /// The content to inject, possibly truncated by token budget enforcement.
    pub injected_content: String,
    /// The depth-from-bottom where this entry should be injected.
    /// 0 = after last message, 1 = one message from end, etc.
    pub effective_depth: i32,
}

/// Activates lorebook entries based on keyword matching against recent messages.
///
/// Per the design spec:
/// - OR keyword matching: entry activates if ANY key or secondary_key appears in recent messages
/// - Scan depth: only the last `scan_depth` messages are scanned
/// - Recursive activation: activated entries' content may trigger other entries, up to `max_recursive_depth` iterations
/// - Token budget enforcement: per-entry soft token budgets truncate content from the end
/// - Entries without explicit budgets share the remaining lorebook allocation
/// - LorebookActivation role filtering: CharacterOnly scans only assistant messages, UserOnly scans only user messages
///
/// Returns activated entries sorted by sort_order, each with their effective depth.
pub fn activate_lorebook_entries(
    all_entries: &[LorebookEntry],
    recent_messages: &[Message],
    scan_depth: i32,
    max_recursive_depth: i32,
    lorebook_token_budget: i64,
    token_service: &TokenCountService,
) -> Vec<ActivatedEntry> {
    // Step 1: Filter to enabled entries only
    let enabled_entries: Vec<&LorebookEntry> = all_entries.iter().filter(|e| e.enabled).collect();

    // Step 2: Take the last `scan_depth` messages as scan scope
    let scan_count = scan_depth.max(0) as usize;
    let scan_messages: Vec<&Message> = recent_messages.iter().rev().take(scan_count).collect();

    // Build separate scan texts per activation type
    let scan_text_all: String = scan_messages
        .iter()
        .map(|m| m.content.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    let scan_text_character_only: String = scan_messages
        .iter()
        .filter(|m| m.role == MessageRole::Assistant)
        .map(|m| m.content.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    let scan_text_user_only: String = scan_messages
        .iter()
        .filter(|m| m.role == MessageRole::User)
        .map(|m| m.content.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    // Step 3: Initial keyword scan — entries activate when their primary keys
    // match the appropriate scan text (per activation_state role filtering).
    // `Always` entries are injected unconditionally (SillyTavern "constant"),
    // regardless of keyword hits. Secondary keys gate activation when present.
    let mut activated_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut activated_entries: Vec<LorebookEntry> = Vec::new();

    for entry in &enabled_entries {
        let matches = if entry.activation_state == LorebookActivation::Always {
            true
        } else {
            let scan_text = match entry.activation_state {
                LorebookActivation::Always => &scan_text_all,
                LorebookActivation::CharacterOnly => &scan_text_character_only,
                LorebookActivation::UserOnly => &scan_text_user_only,
            };
            keyword_matches(&entry.keys, &entry.secondary_keys, scan_text)
        };

        if matches {
            activated_ids.insert(entry.id.clone());
            activated_entries.push((*entry).clone());
        }
    }

    // Step 4: Recursive activation
    // Iteratively check if content of currently activated entries contains keywords of NOT-yet-activated entries
    for _iteration in 0..max_recursive_depth {
        let mut new_activations = Vec::new();

        // Concatenate content of all currently activated entries
        let activated_content: String = activated_entries
            .iter()
            .map(|e| e.content.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        // Check each enabled entry that hasn't been activated yet.
        // `Always` entries are already activated in the initial pass, so here
        // only keyword-gated entries (CharacterOnly/UserOnly) are considered.
        for entry in &enabled_entries {
            if !activated_ids.contains(&entry.id) {
                if keyword_matches(&entry.keys, &entry.secondary_keys, &activated_content) {
                    activated_ids.insert(entry.id.clone());
                    new_activations.push((*entry).clone());
                }
            }
        }

        if new_activations.is_empty() {
            break; // No new activations — stop early
        }

        activated_entries.extend(new_activations);
    }

    // Step 5: Sort by sort_order
    activated_entries.sort_by_key(|e| e.sort_order);

    // Step 6: Token budget enforcement
    let mut remaining_budget = lorebook_token_budget;
    let mut result = Vec::new();

    for entry in &activated_entries {
        let content_tokens = token_service.count_tokens(&entry.content) as i64;

        if let Some(budget) = entry.token_budget {
            // Per-entry explicit token budget: truncate from end if exceeds budget
            if content_tokens > budget {
                let truncated =
                    truncate_to_token_budget(&entry.content, budget as usize, token_service);
                remaining_budget -= budget.min(content_tokens);
                result.push(ActivatedEntry {
                    entry: entry.clone(),
                    injected_content: truncated,
                    effective_depth: entry.depth,
                });
            } else {
                // Under budget — remaining tokens freed for others
                remaining_budget -= content_tokens;
                result.push(ActivatedEntry {
                    entry: entry.clone(),
                    injected_content: entry.content.clone(),
                    effective_depth: entry.depth,
                });
            }
        } else {
            // No explicit budget — share remaining lorebook allocation
            if remaining_budget <= 0 {
                // No budget left — skip this entry
                continue;
            }
            if content_tokens <= remaining_budget {
                remaining_budget -= content_tokens;
                result.push(ActivatedEntry {
                    entry: entry.clone(),
                    injected_content: entry.content.clone(),
                    effective_depth: entry.depth,
                });
            } else {
                // Truncate to fit remaining budget
                let truncated = truncate_to_token_budget(
                    &entry.content,
                    remaining_budget as usize,
                    token_service,
                );
                remaining_budget = 0;
                result.push(ActivatedEntry {
                    entry: entry.clone(),
                    injected_content: truncated,
                    effective_depth: entry.depth,
                });
            }
        }
    }

    result
}

/// Injects activated lorebook entries into the prompt.
///
/// Separates entries by position:
/// - BeforeChar: append to preamble_before_char
/// - AfterChar: append to preamble_after_char
/// - AtDepth with depth == 0: append to preamble_after_char
/// - AtDepth with depth > 0: inject as system message into messages at N messages from end
///
/// Returns (preamble_before_char_with_injections, preamble_after_char_with_injections).
/// Modifies messages in place for AtDepth injections.
pub fn inject_lorebook_entries(
    preamble_before_char: &str,
    preamble_after_char: &str,
    entries: &[ActivatedEntry],
    messages: &mut Vec<rig::completion::Message>,
) -> (String, String) {
    let mut before_additions = Vec::new();
    let mut after_additions = Vec::new();
    let mut depth_injections: Vec<(i32, String)> = Vec::new();

    for activated in entries {
        match activated.entry.position {
            LorebookPosition::BeforeChar => {
                before_additions.push(activated.injected_content.clone());
            }
            LorebookPosition::AfterChar => {
                after_additions.push(activated.injected_content.clone());
            }
            LorebookPosition::AtDepth => {
                if activated.effective_depth == 0 {
                    // Depth 0: append to preamble_after_char
                    after_additions.push(activated.injected_content.clone());
                } else {
                    // Depth > 0: inject into chat history at N messages from end
                    depth_injections.push((
                        activated.effective_depth,
                        activated.injected_content.clone(),
                    ));
                }
            }
        }
    }

    // Build the modified preamble_before_char
    let mut result_before = preamble_before_char.to_string();
    if !before_additions.is_empty() {
        if !result_before.is_empty() {
            result_before.push_str("\n\n");
        }
        result_before.push_str(&before_additions.join("\n\n"));
    }

    // Build the modified preamble_after_char
    let mut result_after = preamble_after_char.to_string();
    if !after_additions.is_empty() {
        if !result_after.is_empty() {
            result_after.push_str("\n\n");
        }
        result_after.push_str(&after_additions.join("\n\n"));
    }

    // Inject depth-based entries into messages (from highest depth to lowest to preserve indices)
    depth_injections.sort_by(|a, b| b.0.cmp(&a.0));

    for (depth, content) in depth_injections {
        let insert_pos = if depth as usize >= messages.len() {
            0 // If depth exceeds message count, insert at the beginning
        } else {
            messages.len() - depth as usize
        };
        // Insert as a system message
        let msg = rig::completion::Message::system(&content);
        messages.insert(insert_pos, msg);
    }

    (result_before, result_after)
}

/// Canonical keyword-matching semantics (SillyTavern-style), shared by the
/// runtime lorebook activator and the test panel so preview == runtime.
///
/// - Primary `keys`: each key string may itself be a comma-separated group;
///   every sub-key in a group must appear in the text (AND within a group), and
///   any group matching is enough (OR across groups). At least one primary group
///   must match for the entry to activate.
/// - Secondary `secondary_keys`: when present, they gate activation (any
///   secondary key must also match) but never activate an entry on their own.
pub fn keyword_matches(keys: &[String], secondary_keys: &[String], text: &str) -> bool {
    let text_lower = text.to_lowercase();

    // Primary keys: AND within a comma-separated group, OR across groups.
    let primary_match = keys.iter().any(|key| {
        let sub_keys: Vec<&str> = key
            .split(',')
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
            .collect();
        if sub_keys.is_empty() {
            // Treat an empty/whitespace key string as a single empty token that
            // never matches, so an entry with only empty keys doesn't auto-fire.
            return false;
        }
        sub_keys
            .iter()
            .all(|sk| text_lower.contains(&sk.to_lowercase()))
    });

    if !primary_match {
        return false;
    }

    // Secondary keys gate activation but never activate alone.
    if secondary_keys.is_empty() {
        return true;
    }
    secondary_keys
        .iter()
        .any(|sk| !sk.trim().is_empty() && text_lower.contains(&sk.to_lowercase()))
}

/// Truncates text to fit within a token budget by removing characters from the end.
/// Uses binary search to find the longest prefix that fits within budget.
fn truncate_to_token_budget(
    content: &str,
    budget: usize,
    token_service: &TokenCountService,
) -> String {
    if token_service.count_tokens(content) <= budget {
        return content.to_string();
    }

    // Binary search for the right truncation point
    let chars: Vec<char> = content.chars().collect();
    let mut low = 0usize;
    let mut high = chars.len();

    while low < high {
        let mid = low + (high - low) / 2;
        let truncated: String = chars[..mid].iter().collect();
        if token_service.count_tokens(&truncated) <= budget {
            low = mid + 1;
        } else {
            high = mid;
        }
    }

    // low is now the first position that exceeds budget, so use low-1
    if low == 0 {
        return String::new();
    }
    chars[..low - 1].iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::conversation::Message as DomainMessage;
    use crate::domain::lorebook::{LorebookActivation, LorebookPosition};
    use jiff::Timestamp;

    fn make_test_entry(name: &str, keys: Vec<&str>, content: &str) -> LorebookEntry {
        let mut entry = LorebookEntry::new("lorebook-1".to_string());
        entry.id = format!("entry-{}", name);
        entry.keys = keys.iter().map(|s| s.to_string()).collect();
        entry.content = content.to_string();
        entry
    }

    fn make_test_entry_with_details(
        name: &str,
        keys: Vec<&str>,
        content: &str,
        depth: i32,
        enabled: bool,
        token_budget: Option<i64>,
        sort_order: i32,
        secondary_keys: Vec<&str>,
        activation_state: LorebookActivation,
        position: LorebookPosition,
    ) -> LorebookEntry {
        let mut entry = LorebookEntry::new("lorebook-1".to_string());
        entry.id = format!("entry-{}", name);
        entry.keys = keys.iter().map(|s| s.to_string()).collect();
        entry.secondary_keys = secondary_keys.iter().map(|s| s.to_string()).collect();
        entry.content = content.to_string();
        entry.position = position;
        entry.depth = depth;
        entry.enabled = enabled;
        entry.token_budget = token_budget;
        entry.sort_order = sort_order;
        entry.activation_state = activation_state;
        entry
    }

    fn make_test_message(role: MessageRole, content: &str) -> DomainMessage {
        DomainMessage::new("conv-1".to_string(), role, content.to_string())
    }

    fn token_service() -> TokenCountService {
        TokenCountService::new()
    }

    #[test]
    fn test_keyword_in_recent_messages_activates_entry() {
        let entry = make_test_entry(
            "Dragon Lore",
            vec!["dragon"],
            "Dragons are ancient creatures.",
        );
        let messages = vec![make_test_message(MessageRole::User, "I saw a dragon!")];

        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());

        assert_eq!(
            activated.len(),
            1,
            "Entry should activate when keyword appears in recent messages"
        );
    }

    #[test]
    fn test_keyword_not_present_no_activation() {
        // Use a keyword-gated activation state (CharacterOnly) so absence of the
        // keyword means no activation. (`Always` entries activate unconditionally.)
        let mut entry = make_test_entry(
            "Dragon Lore",
            vec!["dragon"],
            "Dragons are ancient creatures.",
        );
        entry.activation_state = LorebookActivation::CharacterOnly;
        let messages = vec![make_test_message(MessageRole::Assistant, "I saw a cat!")];

        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());

        assert!(
            activated.is_empty(),
            "Entry should not activate when keyword is not in recent messages"
        );
    }

    #[test]
    fn test_or_logic_any_keyword_activates() {
        let mut entry = make_test_entry(
            "Fire Lore",
            vec!["dragon", "fire"],
            "Fire breathes from dragons.",
        );
        entry.keys = vec!["dragon".to_string(), "fire".to_string()];
        let messages = vec![make_test_message(MessageRole::User, "The fire is warm.")];

        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());

        assert_eq!(
            activated.len(),
            1,
            "Entry should activate on any keyword match (OR logic)"
        );
    }

    #[test]
    fn test_disabled_entry_not_activated() {
        let entry = make_test_entry_with_details(
            "Disabled",
            vec!["keyword"],
            "Content",
            4,
            false,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let messages = vec![make_test_message(MessageRole::User, "A keyword appeared!")];

        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());

        assert!(
            activated.is_empty(),
            "Disabled entry should not be activated"
        );
    }

    #[test]
    fn test_scan_depth_limits_scope() {
        let mut entry = make_test_entry("Recent Only", vec!["dragon"], "Dragons exist.");
        // Keyword-gated so scan scope actually matters (Always would activate regardless).
        entry.activation_state = LorebookActivation::CharacterOnly;
        let messages = vec![
            make_test_message(MessageRole::User, "Hello there"),
            make_test_message(MessageRole::Assistant, "Hi!"),
            make_test_message(MessageRole::User, "I love dragons"),
            make_test_message(MessageRole::Assistant, "Cool!"),
        ];

        // scan_depth=1: only the most recent message is scanned, no "dragon"
        let activated =
            activate_lorebook_entries(&[entry.clone()], &messages, 1, 2, 10000, &token_service());
        assert!(
            activated.is_empty(),
            "With scan_depth=1, keyword in 3rd-from-last message should not activate"
        );

        // scan_depth=3: "dragon" is in scope, but CharacterOnly scans only
        // assistant messages (none contain "dragon"), so it stays inactive.
        let activated =
            activate_lorebook_entries(&[entry], &messages, 3, 2, 10000, &token_service());
        assert!(
            activated.is_empty(),
            "CharacterOnly scans assistant messages; none contain the keyword"
        );
    }

    #[test]
    fn test_recursive_activation() {
        // A is the seed (Always). B is keyword-gated (CharacterOnly) on
        // "dungeon", which only appears in A's injected content — so B can
        // only activate via recursive activation from A.
        let entry_a = make_test_entry_with_details(
            "A",
            vec!["castle"],
            "The castle has a dungeon.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let entry_b = make_test_entry_with_details(
            "B",
            vec!["dungeon"],
            "The dungeon is dark and cold.",
            4,
            true,
            None,
            1,
            vec![],
            LorebookActivation::CharacterOnly,
            LorebookPosition::AfterChar,
        );

        let messages = vec![make_test_message(
            MessageRole::Assistant,
            "I approach the castle.",
        )];

        let activated = activate_lorebook_entries(
            &[entry_a, entry_b],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert_eq!(
            activated.len(),
            2,
            "Entry B should activate recursively because entry A's content contains 'dungeon'"
        );
    }

    #[test]
    fn test_recursive_max_depth_prevents_infinite_loop() {
        // A is the seed (Always, activates unconditionally). B is keyword-gated
        // (CharacterOnly) so it can ONLY activate via recursion from A's content.
        let entry_a = make_test_entry_with_details(
            "A",
            vec!["castle"],
            "The castle has a dungeon.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let entry_b = make_test_entry_with_details(
            "B",
            vec!["dungeon"],
            "The dungeon has a treasure.",
            4,
            true,
            None,
            1,
            vec![],
            LorebookActivation::CharacterOnly,
            LorebookPosition::AfterChar,
        );

        let messages = vec![make_test_message(
            MessageRole::Assistant,
            "I approach the castle.",
        )];

        // max_recursive_depth=0: only initial scan, no recursive activation.
        // A activates (Always); B does not (keyword "dungeon" not in scan text,
        // and no recursion runs).
        let activated = activate_lorebook_entries(
            &[entry_a.clone(), entry_b.clone()],
            &messages,
            2,
            0,
            10000,
            &token_service(),
        );
        assert_eq!(
            activated.len(),
            1,
            "With max_recursive_depth=0, only A should activate"
        );
    }

    #[test]
    fn test_per_entry_token_budget_truncates() {
        let entry = make_test_entry_with_details(
            "Budget Entry",
            vec!["keyword"],
            "This is a longer piece of content that should be truncated when the token budget is very small.",
            4,
            true,
            Some(5), // Very small budget
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );

        let messages = vec![make_test_message(MessageRole::User, "A keyword appeared!")];

        let original_content_len = entry.content.len();
        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());

        assert_eq!(activated.len(), 1, "Entry should activate even with budget");
        assert!(
            activated[0].injected_content.len() < original_content_len,
            "Content should be truncated when token budget is small"
        );
    }

    #[test]
    fn test_entries_without_budget_share_remaining() {
        let entry_a = make_test_entry_with_details(
            "A",
            vec!["keyword_a"],
            "Short content for A.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let entry_b = make_test_entry_with_details(
            "B",
            vec!["keyword_b"],
            "Short content for B.",
            4,
            true,
            None,
            1,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );

        let messages = vec![make_test_message(MessageRole::User, "keyword_a keyword_b")];

        let activated =
            activate_lorebook_entries(&[entry_a, entry_b], &messages, 2, 2, 10, &token_service());

        assert!(!activated.is_empty(), "At least one entry should activate");
    }

    #[test]
    fn test_depth_from_bottom_insertion() {
        let entry_depth0 = make_test_entry_with_details(
            "Depth0",
            vec!["keyword0"],
            "Content at depth 0",
            0,
            true,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let entry_depth1 = make_test_entry_with_details(
            "Depth1",
            vec!["keyword1"],
            "Content at depth 1",
            1,
            true,
            None,
            1,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );

        let activated = activate_lorebook_entries(
            &[entry_depth0, entry_depth1],
            &[make_test_message(MessageRole::User, "keyword0 keyword1")],
            2,
            2,
            10000,
            &token_service(),
        );

        assert_eq!(activated.len(), 2);
        let depth0 = activated
            .iter()
            .find(|a| a.entry.id == "entry-Depth0")
            .unwrap();
        let depth1 = activated
            .iter()
            .find(|a| a.entry.id == "entry-Depth1")
            .unwrap();
        assert_eq!(depth0.effective_depth, 0);
        assert_eq!(depth1.effective_depth, 1);
    }

    #[test]
    fn test_secondary_keys_trigger_activation() {
        let entry = make_test_entry_with_details(
            "Secondary",
            vec!["primary_kw"],
            "Secondary term content.",
            4,
            true,
            None,
            0,
            vec!["secondary_kw"],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let messages = vec![make_test_message(
            MessageRole::User,
            "The secondary_kw appeared!",
        )];

        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());

        assert_eq!(
            activated.len(),
            1,
            "Secondary key should trigger activation"
        );
    }

    #[test]
    fn test_sort_order_respected() {
        let entry_b = make_test_entry_with_details(
            "B",
            vec!["keyword"],
            "Content B",
            4,
            true,
            None,
            2,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let entry_a = make_test_entry_with_details(
            "A",
            vec!["keyword"],
            "Content A",
            4,
            true,
            None,
            1,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );

        let messages = vec![make_test_message(MessageRole::User, "keyword")];

        let activated = activate_lorebook_entries(
            &[entry_b, entry_a],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        // Should be sorted by sort_order: A (1) then B (2)
        assert_eq!(activated[0].entry.id, "entry-A");
        assert_eq!(activated[1].entry.id, "entry-B");
    }

    #[test]
    fn test_activation_state_character_only() {
        let entry = make_test_entry_with_details(
            "CharOnly",
            vec!["magic"],
            "Magic content.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::CharacterOnly,
            LorebookPosition::AfterChar,
        );

        // User message only — should NOT activate CharacterOnly entry
        let user_messages = vec![make_test_message(MessageRole::User, "I use magic!")];
        let activated = activate_lorebook_entries(
            &[entry.clone()],
            &user_messages,
            2,
            2,
            10000,
            &token_service(),
        );
        assert!(
            activated.is_empty(),
            "CharacterOnly should not activate on user-only messages"
        );

        // Assistant message — should activate CharacterOnly entry
        let assistant_messages = vec![make_test_message(MessageRole::Assistant, "I cast magic!")];
        let activated =
            activate_lorebook_entries(&[entry], &assistant_messages, 2, 2, 10000, &token_service());
        assert_eq!(
            activated.len(),
            1,
            "CharacterOnly should activate on assistant messages"
        );
    }

    #[test]
    fn test_activation_state_user_only() {
        let entry = make_test_entry_with_details(
            "UserOnly",
            vec!["magic"],
            "Magic content.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::UserOnly,
            LorebookPosition::AfterChar,
        );

        // Assistant message only — should NOT activate UserOnly entry
        let assistant_messages = vec![make_test_message(MessageRole::Assistant, "I cast magic!")];
        let activated = activate_lorebook_entries(
            &[entry.clone()],
            &assistant_messages,
            2,
            2,
            10000,
            &token_service(),
        );
        assert!(
            activated.is_empty(),
            "UserOnly should not activate on assistant-only messages"
        );

        // User message — should activate UserOnly entry
        let user_messages = vec![make_test_message(MessageRole::User, "I use magic!")];
        let activated =
            activate_lorebook_entries(&[entry], &user_messages, 2, 2, 10000, &token_service());
        assert_eq!(
            activated.len(),
            1,
            "UserOnly should activate on user messages"
        );
    }

    #[test]
    fn test_activation_state_always_activates_on_both() {
        let entry = make_test_entry_with_details(
            "Always",
            vec!["magic"],
            "Magic content.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );

        // User message
        let user_messages = vec![make_test_message(MessageRole::User, "I use magic!")];
        let activated = activate_lorebook_entries(
            &[entry.clone()],
            &user_messages,
            2,
            2,
            10000,
            &token_service(),
        );
        assert_eq!(
            activated.len(),
            1,
            "Always should activate on user messages"
        );

        // Assistant message
        let assistant_messages = vec![make_test_message(MessageRole::Assistant, "I cast magic!")];
        let activated =
            activate_lorebook_entries(&[entry], &assistant_messages, 2, 2, 10000, &token_service());
        assert_eq!(
            activated.len(),
            1,
            "Always should activate on assistant messages"
        );
    }

    #[test]
    fn test_inject_before_char() {
        let mut entry = make_test_entry("Lore", vec!["kw"], "Lore content here");
        entry.position = LorebookPosition::BeforeChar;
        let activated = ActivatedEntry {
            entry,
            injected_content: "Lore content here".to_string(),
            effective_depth: 4,
        };
        let mut messages: Vec<rig::completion::Message> = vec![];

        let (before, after) = inject_lorebook_entries(
            "System prompt",
            "Character definition",
            &[activated],
            &mut messages,
        );

        assert!(before.contains("System prompt"));
        assert!(before.contains("Lore content here"));
        assert_eq!(after, "Character definition");
        assert!(messages.is_empty(), "BeforeChar should not modify messages");
    }

    #[test]
    fn test_inject_after_char() {
        let mut entry = make_test_entry("Lore", vec!["kw"], "Lore content here");
        entry.position = LorebookPosition::AfterChar;
        let activated = ActivatedEntry {
            entry,
            injected_content: "Lore content here".to_string(),
            effective_depth: 4,
        };
        let mut messages: Vec<rig::completion::Message> = vec![];

        let (before, after) = inject_lorebook_entries(
            "System prompt",
            "Character definition",
            &[activated],
            &mut messages,
        );

        assert_eq!(before, "System prompt");
        assert!(after.contains("Character definition"));
        assert!(after.contains("Lore content here"));
        assert!(messages.is_empty(), "AfterChar should not modify messages");
    }

    #[test]
    fn test_inject_at_depth() {
        let mut entry = make_test_entry_with_details(
            "Lore",
            vec!["kw"],
            "Lore injection",
            1,
            true,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AtDepth,
        );
        let activated = ActivatedEntry {
            entry,
            injected_content: "Lore injection".to_string(),
            effective_depth: 1,
        };
        let mut messages = vec![
            rig::completion::Message::user("First"),
            rig::completion::Message::assistant("Second"),
        ];

        let (before, after) =
            inject_lorebook_entries("System", "Character", &[activated], &mut messages);

        assert_eq!(before, "System");
        assert_eq!(after, "Character");
        assert_eq!(messages.len(), 3, "Should have inserted a system message");
        // The injection should be 1 message from end (index 1 in a 2-message list)
        // After insertion, the system message is at position 1
        assert!(matches!(
            messages[1],
            rig::completion::Message::System { .. }
        ));
    }

    #[test]
    fn test_keyword_matches_or_logic() {
        let keywords = vec!["dragon".to_string(), "fire".to_string()];
        assert!(keyword_matches(&keywords, &[], "the dragon flies"));
        assert!(keyword_matches(&keywords, &[], "the fire burns"));
        assert!(!keyword_matches(&keywords, &[], "the cat sleeps"));
    }

    #[test]
    fn test_keyword_matches_case_insensitive() {
        let keywords = vec!["Dragon".to_string()];
        assert!(keyword_matches(&keywords, &[], "the dragon flies"));
        assert!(keyword_matches(&keywords, &[], "the DRAGON flies"));
    }

    #[test]
    fn test_keyword_matches_empty_keyword() {
        let keywords = vec![String::new(), "valid".to_string()];
        assert!(keyword_matches(&keywords, &[], "a valid point"));
        // Empty keyword should not match
        let empty_only = vec![String::new()];
        assert!(!keyword_matches(&empty_only, &[], "any text"));
    }

    #[test]
    fn test_keyword_matches_comma_group_and_logic() {
        // Comma-separated group: all sub-keys must match (AND within group).
        let keys = vec!["dragon, fire".to_string()];
        assert!(keyword_matches(&keys, &[], "the dragon breathes fire"));
        assert!(
            !keyword_matches(&keys, &[], "the dragon sleeps"),
            "AND within a comma group: only one sub-key present"
        );
    }

    #[test]
    fn test_keyword_matches_multi_group_or_logic() {
        // OR across key groups.
        let keys = vec!["dragon".to_string(), "wizard, staff".to_string()];
        assert!(keyword_matches(&keys, &[], "a dragon"));
        assert!(keyword_matches(&keys, &[], "the wizard holds a staff"));
        assert!(!keyword_matches(&keys, &[], "the wizard sleeps"));
    }

    #[test]
    fn test_keyword_matches_secondary_only_no_activation() {
        // Secondary keys never activate alone.
        let keys = vec!["primary".to_string()];
        let secondary = vec!["secondary".to_string()];
        assert!(
            !keyword_matches(&keys, &secondary, "only the secondary appears"),
            "Secondary-only must not activate"
        );
    }

    #[test]
    fn test_keyword_matches_secondary_gates_primary() {
        // Primary matches but secondary present and absent → no activation.
        let keys = vec!["primary".to_string()];
        let secondary = vec!["secondary".to_string()];
        assert!(
            !keyword_matches(&keys, &secondary, "the primary is here"),
            "Primary match with absent gating secondary must not activate"
        );
        assert!(
            keyword_matches(&keys, &secondary, "primary and secondary both"),
            "Primary + matching secondary activates"
        );
    }

    #[test]
    fn test_always_entry_activates_without_keywords() {
        // `Always` entries activate unconditionally (SillyTavern "constant").
        let entry = make_test_entry_with_details(
            "Constant",
            vec![], // no keys
            "Constant content.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::Always,
            LorebookPosition::AfterChar,
        );
        let messages = vec![make_test_message(
            MessageRole::User,
            "totally unrelated text",
        )];
        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());
        assert_eq!(
            activated.len(),
            1,
            "Always entry must activate without keywords"
        );
    }

    #[test]
    fn test_character_only_keyword_gated_not_constant() {
        // CharacterOnly with no keyword hit does NOT activate (proves Always is
        // special-cased, not all entries).
        let entry = make_test_entry_with_details(
            "Gated",
            vec!["magic"],
            "Magic content.",
            4,
            true,
            None,
            0,
            vec![],
            LorebookActivation::CharacterOnly,
            LorebookPosition::AfterChar,
        );
        let messages = vec![make_test_message(
            MessageRole::User,
            "nothing relevant here",
        )];
        let activated =
            activate_lorebook_entries(&[entry], &messages, 2, 2, 10000, &token_service());
        assert!(
            activated.is_empty(),
            "CharacterOnly must require a keyword hit"
        );
    }
}
