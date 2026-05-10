use crate::domain::conversation::Message;
use crate::domain::lorebook::LorebookEntry;
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
/// Per D-09, D-10, D-11, D-12, D-13:
/// - OR keyword matching: entry activates if ANY keyword or secondary_term appears in recent messages
/// - Scan depth: only the last `scan_depth` messages are scanned per D-10
/// - Recursive activation: activated entries' content may trigger other entries, up to `max_recursive_depth` iterations per D-12
/// - Token budget enforcement: per-entry soft token budgets truncate content from the end per D-13
/// - Entries without explicit budgets share the remaining lorebook allocation
///
/// Returns activated entries sorted by sort_order, each with their effective depth_from_bottom.
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

    // Step 2: Take the last `scan_depth` messages as scan scope (per D-10)
    let scan_count = scan_depth.max(0) as usize;
    let scan_messages: Vec<&Message> = recent_messages
        .iter()
        .rev()
        .take(scan_count)
        .collect();

    // Concatenate scan messages into scan text
    let scan_text: String = scan_messages
        .iter()
        .map(|m| m.content.to_lowercase())
        .collect::<Vec<_>>()
        .join(" ");

    // Step 3: Initial keyword scan — check if ANY keyword or secondary_term appears in scan text (OR logic, case-insensitive, per D-09)
    let mut activated_ids: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut activated_entries: Vec<LorebookEntry> = Vec::new();

    for entry in &enabled_entries {
        if keyword_matches(&entry.keywords, &scan_text)
            || keyword_matches(&entry.secondary_terms, &scan_text)
        {
            activated_ids.insert(entry.id.clone());
            activated_entries.push((*entry).clone());
        }
    }

    // Step 4: Recursive activation (per D-12)
    // Iteratively check if content of newly-activated entries contains keywords of NOT-yet-activated entries
    for _iteration in 0..max_recursive_depth {
        let mut new_activations = Vec::new();

        // Concatenate content of all currently activated entries
        let activated_content: String = activated_entries
            .iter()
            .map(|e| e.content.to_lowercase())
            .collect::<Vec<_>>()
            .join(" ");

        // Check each enabled entry that hasn't been activated yet
        for entry in &enabled_entries {
            if !activated_ids.contains(&entry.id) {
                if keyword_matches(&entry.keywords, &activated_content)
                    || keyword_matches(&entry.secondary_terms, &activated_content)
                {
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

    // Step 6: Token budget enforcement (per D-13)
    let mut remaining_budget = lorebook_token_budget;
    let mut result = Vec::new();

    for entry in &activated_entries {
        let content_tokens = token_service.count_tokens(&entry.content) as i64;

        if let Some(budget) = entry.token_budget {
            // Per-entry explicit token budget: truncate from end if exceeds budget
            if content_tokens > budget {
                let truncated = truncate_to_token_budget(&entry.content, budget as usize, token_service);
                remaining_budget -= budget.min(content_tokens);
                result.push(ActivatedEntry {
                    entry: entry.clone(),
                    injected_content: truncated,
                    effective_depth: entry.depth_from_bottom,
                });
            } else {
                // Under budget — remaining tokens freed for others
                remaining_budget -= content_tokens;
                result.push(ActivatedEntry {
                    entry: entry.clone(),
                    injected_content: entry.content.clone(),
                    effective_depth: entry.depth_from_bottom,
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
                    effective_depth: entry.depth_from_bottom,
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
                    effective_depth: entry.depth_from_bottom,
                });
            }
        }
    }

    result
}

/// Injects activated lorebook entries into the prompt.
///
/// For depth_from_bottom = 0 entries: append to preamble (after character definition).
/// For depth > 0 entries: inject into chat history at N messages from end.
///
/// Returns the modified preamble. Modifies messages in place for depth-based injections.
pub fn inject_lorebook_entries(
    preamble: &str,
    entries: &[ActivatedEntry],
    messages: &mut Vec<Message>,
) -> String {
    let mut preamble_additions = Vec::new();
    let mut depth_injections: Vec<(i32, String)> = Vec::new();

    for activated in entries {
        if activated.effective_depth == 0 {
            // Depth 0: append to preamble (after character definition)
            preamble_additions.push(activated.injected_content.clone());
        } else {
            // Depth > 0: inject into chat history at N messages from end
            depth_injections.push((activated.effective_depth, activated.injected_content.clone()));
        }
    }

    // Build the modified preamble
    let mut result = preamble.to_string();
    if !preamble_additions.is_empty() {
        if !result.is_empty() {
            result.push_str("\n\n");
        }
        result.push_str(&preamble_additions.join("\n\n"));
    }

    // Inject depth-based entries into messages (from highest depth to lowest to preserve indices)
    // Sort by depth descending so we insert from the end first
    depth_injections.sort_by(|a, b| b.0.cmp(&a.0));

    for (depth, content) in depth_injections {
        let insert_pos = if depth as usize >= messages.len() {
            0 // If depth exceeds message count, insert at the beginning
        } else {
            messages.len() - depth as usize
        };
        // Insert as a system message
        let msg = Message {
            id: uuid::Uuid::new_v4().to_string(),
            conversation_id: String::new(),
            branch_id: None,
            role: crate::domain::conversation::MessageRole::System,
            content,
            timestamp: chrono::Local::now().naive_local(),
            sort_order: 0,
        };
        messages.insert(insert_pos, msg);
    }

    result
}

/// Checks if ANY keyword appears in the text (OR logic, case-insensitive, per D-09).
fn keyword_matches(keywords: &[String], text: &str) -> bool {
    let text_lower = text.to_lowercase();
    keywords.iter().any(|kw| {
        let kw_lower = kw.to_lowercase();
        !kw_lower.is_empty() && text_lower.contains(&kw_lower)
    })
}

/// Truncates text to fit within a token budget by removing characters from the end.
/// Uses iterative approach to find the longest prefix that fits within budget.
fn truncate_to_token_budget(content: &str, budget: usize, token_service: &TokenCountService) -> String {
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
    use crate::domain::conversation::MessageRole;
    use crate::domain::lorebook::{InsertionPosition, LorebookEntry};
    use chrono::NaiveDateTime;

    fn make_test_entry(
        name: &str,
        keywords: Vec<&str>,
        content: &str,
    ) -> LorebookEntry {
        let now = NaiveDateTime::parse_from_str("2024-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        LorebookEntry {
            id: format!("entry-{}", name),
            character_id: "char-1".to_string(),
            name: name.to_string(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            secondary_terms: Vec::new(),
            content: content.to_string(),
            insertion_position: InsertionPosition::AfterChar,
            depth_from_bottom: 4,
            enabled: true,
            token_budget: None,
            sort_order: 0,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_test_entry_with_details(
        name: &str,
        keywords: Vec<&str>,
        content: &str,
        depth: i32,
        enabled: bool,
        token_budget: Option<i64>,
        sort_order: i32,
        secondary_terms: Vec<&str>,
    ) -> LorebookEntry {
        let now = NaiveDateTime::parse_from_str("2024-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S").unwrap();
        LorebookEntry {
            id: format!("entry-{}", name),
            character_id: "char-1".to_string(),
            name: name.to_string(),
            keywords: keywords.iter().map(|s| s.to_string()).collect(),
            secondary_terms: secondary_terms.iter().map(|s| s.to_string()).collect(),
            content: content.to_string(),
            insertion_position: InsertionPosition::AfterChar,
            depth_from_bottom: depth,
            enabled,
            token_budget,
            sort_order,
            created_at: now,
            updated_at: now,
        }
    }

    fn make_test_message(role: MessageRole, content: &str) -> Message {
        Message {
            id: "test-id".to_string(),
            conversation_id: "conv-1".to_string(),
            branch_id: None,
            role,
            content: content.to_string(),
            timestamp: NaiveDateTime::parse_from_str("2024-01-01T00:00:00", "%Y-%m-%dT%H:%M:%S")
                .unwrap(),
            sort_order: 0,
        }
    }

    fn token_service() -> TokenCountService {
        TokenCountService::new()
    }

    #[test]
    fn test_keyword_in_recent_messages_activates_entry() {
        // Test 1: Entry with keyword "dragon" activates when "dragon" appears in recent messages
        let entry = make_test_entry("Dragon Lore", vec!["dragon"], "Dragons are ancient creatures.");
        let messages = vec![make_test_message(MessageRole::User, "I saw a dragon!")];

        let activated = activate_lorebook_entries(
            &[entry],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert_eq!(activated.len(), 1, "Entry should activate when keyword appears in recent messages");
        assert_eq!(activated[0].entry.name, "Dragon Lore");
    }

    #[test]
    fn test_keyword_not_present_no_activation() {
        // Test 2: Entry does NOT activate when keyword not in recent messages
        let entry = make_test_entry("Dragon Lore", vec!["dragon"], "Dragons are ancient creatures.");
        let messages = vec![make_test_message(MessageRole::User, "I saw a cat!")];

        let activated = activate_lorebook_entries(
            &[entry],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert!(activated.is_empty(), "Entry should not activate when keyword is not in recent messages");
    }

    #[test]
    fn test_or_logic_any_keyword_activates() {
        // Test 3: OR logic — entry with keywords ["dragon", "fire"] activates on either
        let entry = make_test_entry("Fire Lore", vec!["dragon", "fire"], "Fire breathes from dragons.");
        let messages = vec![make_test_message(MessageRole::User, "The fire is warm.")];

        let activated = activate_lorebook_entries(
            &[entry],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert_eq!(activated.len(), 1, "Entry should activate on any keyword match (OR logic)");
    }

    #[test]
    fn test_disabled_entry_not_activated() {
        // Test 4: Only enabled entries are considered for activation
        let entry = make_test_entry_with_details(
            "Disabled", vec!["keyword"], "Content", 4, false, None, 0, vec![],
        );
        let messages = vec![make_test_message(MessageRole::User, "A keyword appeared!")];

        let activated = activate_lorebook_entries(
            &[entry],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert!(activated.is_empty(), "Disabled entry should not be activated");
    }

    #[test]
    fn test_scan_depth_limits_scope() {
        // Test 5: Activation respects scan_depth — scans N most recent messages per D-10
        let entry = make_test_entry("Recent Only", vec!["dragon"], "Dragons exist.");
        // "dragon" is in the 3rd message from end, but scan_depth=2
        let messages = vec![
            make_test_message(MessageRole::User, "Hello there"),        // oldest
            make_test_message(MessageRole::Assistant, "Hi!"),
            make_test_message(MessageRole::User, "I love dragons"),      // keyword here
            make_test_message(MessageRole::Assistant, "Cool!"),         // most recent
        ];

        // scan_depth=1: only the most recent message is scanned, no "dragon"
        let activated = activate_lorebook_entries(
            &[entry.clone()],
            &messages,
            1,
            2,
            10000,
            &token_service(),
        );
        assert!(activated.is_empty(), "With scan_depth=1, keyword in 3rd-from-last message should not activate");

        // scan_depth=3: "dragon" is within scan scope
        let activated = activate_lorebook_entries(
            &[entry],
            &messages,
            3,
            2,
            10000,
            &token_service(),
        );
        assert_eq!(activated.len(), 1, "With scan_depth=3, keyword should be found");
    }

    #[test]
    fn test_recursive_activation() {
        // Test 6: Entry A's content contains entry B's keyword, B activates too
        let entry_a = make_test_entry_with_details(
            "A",
            vec!["castle"],
            "The castle has a dungeon.",
            4,
            true,
            None,
            0,
            vec![],
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
        );

        let messages = vec![make_test_message(MessageRole::User, "I approach the castle.")];

        let activated = activate_lorebook_entries(
            &[entry_a, entry_b],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert_eq!(activated.len(), 2, "Entry B should activate recursively because entry A's content contains 'dungeon'");
        let names: Vec<&str> = activated.iter().map(|a| a.entry.name.as_str()).collect();
        assert!(names.contains(&"A"), "Entry A should be activated");
        assert!(names.contains(&"B"), "Entry B should be activated recursively");
    }

    #[test]
    fn test_recursive_max_depth_prevents_infinite_loop() {
        // Test 7: Recursive max depth prevents infinite loops (default 2, per D-12)
        // Entry A mentions "castle", entry B mentions "dungeon", entry C mentions "castle"
        // With max_recursive_depth=0, only initial keyword match occurs
        let entry_a = make_test_entry_with_details(
            "A",
            vec!["castle"],
            "The castle has a dungeon.",
            4,
            true,
            None,
            0,
            vec![],
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
        );

        let messages = vec![make_test_message(MessageRole::User, "I approach the castle.")];

        // max_recursive_depth=0: only initial scan, no recursive activation
        let activated = activate_lorebook_entries(
            &[entry_a.clone(), entry_b.clone()],
            &messages,
            2,
            0,
            10000,
            &token_service(),
        );
        assert_eq!(activated.len(), 1, "With max_recursive_depth=0, only A should activate");
        assert_eq!(activated[0].entry.name, "A");
    }

    #[test]
    fn test_per_entry_token_budget_truncates() {
        // Test 8: Per-entry token_budget truncates content from end per D-13
        let entry = make_test_entry_with_details(
            "Budget Entry",
            vec!["keyword"],
            "This is a longer piece of content that should be truncated when the token budget is very small.",
            4,
            true,
            Some(5), // Very small budget — only ~5 tokens
            0,
            vec![],
        );

        let messages = vec![make_test_message(MessageRole::User, "A keyword appeared!")];

        let original_content_len = entry.content.len();
        let activated = activate_lorebook_entries(
            &[entry],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert_eq!(activated.len(), 1, "Entry should activate even with budget");
        // The injected content should be shorter than the original
        assert!(
            activated[0].injected_content.len() < original_content_len,
            "Content should be truncated when token budget is small"
        );
    }

    #[test]
    fn test_entries_without_budget_share_remaining() {
        // Test 9: Entries without explicit budget share remaining lorebook allocation
        let entry_a = make_test_entry_with_details(
            "A",
            vec!["keyword_a"],
            "Short content for A.",
            4,
            true,
            None, // No explicit budget
            0,
            vec![],
        );
        let entry_b = make_test_entry_with_details(
            "B",
            vec!["keyword_b"],
            "Short content for B.",
            4,
            true,
            None, // No explicit budget
            1,
            vec![],
        );

        let messages = vec![
            make_test_message(MessageRole::User, "keyword_a keyword_b"),
        ];

        // With a very small lorebook budget, only first entry fits
        let activated = activate_lorebook_entries(
            &[entry_a.clone(), entry_b.clone()],
            &messages,
            2,
            2,
            10, // Very small total budget
            &token_service(),
        );

        // At least the first entry should be activated (possibly truncated)
        assert!(!activated.is_empty(), "At least one entry should activate");
    }

    #[test]
    fn test_depth_from_bottom_insertion() {
        // Test 10: Depth-from-bottom insertion — depth 0 = after last message, depth 1 = one from end
        let entry_depth0 = make_test_entry_with_details(
            "Depth0",
            vec!["keyword0"],
            "Content at depth 0",
            0,
            true,
            None,
            0,
            vec![],
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
        // Check effective_depth is tracked
        let depth0 = activated.iter().find(|a| a.entry.name == "Depth0").unwrap();
        let depth1 = activated.iter().find(|a| a.entry.name == "Depth1").unwrap();
        assert_eq!(depth0.effective_depth, 0);
        assert_eq!(depth1.effective_depth, 1);
    }

    #[test]
    fn test_secondary_terms_trigger_activation() {
        // Test 11: Secondary terms also trigger activation (broader matching)
        let entry = make_test_entry_with_details(
            "Secondary",
            vec!["primary_kw"],
            "Secondary term content.",
            4,
            true,
            None,
            0,
            vec!["secondary_kw"],
        );
        let messages = vec![make_test_message(MessageRole::User, "The secondary_kw appeared!")];

        let activated = activate_lorebook_entries(
            &[entry],
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        assert_eq!(activated.len(), 1, "Secondary term should trigger activation");
    }

    #[test]
    fn test_inject_depth0_appends_to_preamble() {
        let activated = ActivatedEntry {
            entry: make_test_entry("Lore", vec!["kw"], "Lore content here"),
            injected_content: "Lore content here".to_string(),
            effective_depth: 0,
        };
        let mut messages = vec![make_test_message(MessageRole::User, "Hello")];

        let result = inject_lorebook_entries("System prompt", &[activated], &mut messages);

        assert!(result.contains("System prompt"));
        assert!(result.contains("Lore content here"));
        assert_eq!(messages.len(), 1, "Depth 0 should not modify messages");
    }

    #[test]
    fn test_inject_depth1_inserts_into_history() {
        let activated = ActivatedEntry {
            entry: make_test_entry_with_details("Lore", vec!["kw"], "Lore injection", 1, true, None, 0, vec![]),
            injected_content: "Lore injection".to_string(),
            effective_depth: 1,
        };
        let mut messages = vec![
            make_test_message(MessageRole::User, "First"),
            make_test_message(MessageRole::Assistant, "Second"),
        ];

        let result = inject_lorebook_entries("System prompt", &[activated], &mut messages);

        assert_eq!(messages.len(), 3, "Should have inserted a system message");
        // The injection should be 1 message from end (index 1 in a 2-message list)
        assert_eq!(messages[1].role, MessageRole::System);
        assert_eq!(messages[1].content, "Lore injection");
    }

    #[test]
    fn test_keyword_matches_or_logic() {
        let keywords = vec!["dragon".to_string(), "fire".to_string()];
        assert!(keyword_matches(&keywords, "the dragon flies"));
        assert!(keyword_matches(&keywords, "the fire burns"));
        assert!(!keyword_matches(&keywords, "the cat sleeps"));
    }

    #[test]
    fn test_keyword_matches_case_insensitive() {
        let keywords = vec!["Dragon".to_string()];
        assert!(keyword_matches(&keywords, "the dragon flies"));
        assert!(keyword_matches(&keywords, "the DRAGON flies"));
    }

    #[test]
    fn test_keyword_matches_empty_keyword() {
        let keywords = vec![String::new(), "valid".to_string()];
        assert!(keyword_matches(&keywords, "a valid point"));
        // Empty keyword should not match
        let empty_only = vec![String::new()];
        assert!(!keyword_matches(&empty_only, "any text"));
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
        );

        let messages = vec![make_test_message(MessageRole::User, "keyword")];

        let activated = activate_lorebook_entries(
            &[entry_b, entry_a], // Given in wrong order
            &messages,
            2,
            2,
            10000,
            &token_service(),
        );

        // Should be sorted by sort_order: A (1) then B (2)
        assert_eq!(activated[0].entry.name, "A");
        assert_eq!(activated[1].entry.name, "B");
    }
}
