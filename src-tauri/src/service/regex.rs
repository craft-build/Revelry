use crate::domain::regex_script::RegexScript;
use regex::RegexBuilder;

/// Validates a regex pattern for ReDoS vulnerability per D-06.
/// Uses `regex::RegexBuilder::size_limit(1_000_000)` to cap backtracking.
/// Returns compiled Regex if safe, or error message if dangerous/invalid.
pub fn validate_regex_pattern(pattern: &str, flags: &str) -> Result<regex::Regex, String> {
    let mut builder = RegexBuilder::new(pattern);
    builder.size_limit(1_000_000); // 1MB compiled program limit per D-06
    if flags.contains('i') {
        builder.case_insensitive(true);
    }
    if flags.contains('m') {
        builder.multi_line(true);
    }
    if flags.contains('s') {
        builder.dot_matches_new_line(true);
    }
    // Note: 'g' flag is handled at replacement time, not compilation
    builder.build().map_err(|e| e.to_string())
}

/// Validates only that flags contain valid characters (i, g, m, s only).
pub fn validate_regex_flags(flags: &str) -> Result<(), String> {
    for c in flags.chars() {
        if !matches!(c, 'i' | 'g' | 'm' | 's') {
            return Err(format!(
                "Invalid flag '{}'. Only i, g, m, s are allowed.",
                c
            ));
        }
    }
    Ok(())
}

/// Applies a list of regex scripts to input text per D-05.
/// Scripts run in sort_order sequence (pipe chain) — each script's output
/// becomes the next script's input. Disabled scripts are skipped.
pub fn apply_regex_scripts(text: &str, scripts: &[RegexScript]) -> String {
    let mut sorted: Vec<&RegexScript> = scripts.iter().filter(|s| s.enabled).collect();
    sorted.sort_by_key(|s| s.sort_order);

    let mut result = text.to_string();
    for script in sorted {
        match validate_regex_pattern(&script.find_pattern, &script.flags) {
            Ok(re) => {
                let is_global = script.flags.contains('g');
                if is_global {
                    result = re
                        .replace_all(&result, script.replace_pattern.as_str())
                        .to_string();
                } else {
                    result = re
                        .replace(&result, script.replace_pattern.as_str())
                        .to_string();
                }
            }
            Err(_) => {
                // Skip scripts with invalid patterns at runtime
                // (should have been caught at save time, but be safe)
                tracing::warn!("Skipping regex script '{}' — invalid pattern", script.name);
            }
        }
    }
    result
}

/// Applies a single regex find/replace to input text (for test panel preview).
/// Returns (output_text, match_count). match_count reflects the number of
/// replacements made: all matches for the `g` flag, otherwise 0 or 1.
pub fn apply_single_regex(
    text: &str,
    pattern: &str,
    replace: &str,
    flags: &str,
) -> Result<(String, usize), String> {
    let re = validate_regex_pattern(pattern, flags)?;
    let is_global = flags.contains('g');
    if is_global {
        let match_count = re.find_iter(text).count();
        Ok((re.replace_all(text, replace).to_string(), match_count))
    } else {
        let matched = re.is_match(text);
        Ok((
            re.replace(text, replace).to_string(),
            if matched { 1 } else { 0 },
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::regex_script::RegexScript;
    use jiff::Timestamp;

    fn make_script(
        name: &str,
        find: &str,
        replace: &str,
        flags: &str,
        sort_order: i32,
        enabled: bool,
    ) -> RegexScript {
        let now = Timestamp::now();
        RegexScript {
            id: uuid::Uuid::new_v4().to_string(),
            character_id: "test-char".to_string(),
            name: name.to_string(),
            find_pattern: find.to_string(),
            replace_pattern: replace.to_string(),
            flags: flags.to_string(),
            sort_order,
            enabled,
            created_at: now,
            updated_at: now,
        }
    }

    #[test]
    fn test_validate_safe_pattern() {
        let result = validate_regex_pattern("hello", "gi");
        assert!(result.is_ok(), "Safe pattern should compile: {:?}", result);
    }

    #[test]
    fn test_validate_redos_pattern() {
        // The regex crate prevents catastrophic backtracking at runtime by design.
        // size_limit catches patterns that compile to huge automatons (memory DoS).
        // We test with a deeply nested pattern that creates an exponentially large
        // NFA, which exceeds the 1MB size_limit.
        let mut huge_pattern = String::new();
        for _ in 0..500 {
            huge_pattern = format!("({}a?)", huge_pattern);
        }
        let result = validate_regex_pattern(&huge_pattern, "i");
        assert!(
            result.is_err(),
            "Huge automaton pattern should be rejected by size_limit"
        );
    }

    #[test]
    fn test_validate_invalid_syntax() {
        let result = validate_regex_pattern("[unclosed", "i");
        assert!(
            result.is_err(),
            "Invalid syntax should be rejected: {:?}",
            result
        );
        let err = result.unwrap_err();
        assert!(!err.is_empty(), "Error message should not be empty");
    }

    #[test]
    fn test_apply_two_enabled_scripts_in_order() {
        let scripts = vec![
            make_script("Replace hello", "hello", "hi", "gi", 1, true),
            make_script("Replace world", "world", "earth", "gi", 2, true),
        ];
        let result = apply_regex_scripts("hello world", &scripts);
        assert_eq!(
            result, "hi earth",
            "Scripts should apply in sort_order pipe chain"
        );
    }

    #[test]
    fn test_apply_skips_disabled_scripts() {
        let scripts = vec![
            make_script("Replace hello", "hello", "hi", "gi", 1, true),
            make_script("Replace world", "world", "earth", "gi", 2, false),
        ];
        let result = apply_regex_scripts("hello world", &scripts);
        assert_eq!(result, "hi world", "Disabled scripts should be skipped");
    }

    #[test]
    fn test_apply_global_and_case_insensitive() {
        let scripts = vec![make_script("Replace Hello", "hello", "hi", "gi", 1, true)];
        let result = apply_regex_scripts("Hello HELLO hello", &scripts);
        assert_eq!(
            result, "hi hi hi",
            "Global + case-insensitive should replace all"
        );
    }

    #[test]
    fn test_apply_capture_group_replacement() {
        let re_result = validate_regex_pattern("(\\w+)", "g");
        assert!(
            re_result.is_ok(),
            "Pattern should validate: {:?}",
            re_result
        );

        let scripts = vec![make_script("Wrap words", "(\\w+)", "[$1]", "g", 1, true)];
        let result = apply_regex_scripts("hello world", &scripts);
        assert_eq!(
            result, "[hello] [world]",
            "Capture groups with $1 should work"
        );
    }

    #[test]
    fn test_validate_flags_valid() {
        assert!(validate_regex_flags("gi").is_ok());
        assert!(validate_regex_flags("igms").is_ok());
        assert!(validate_regex_flags("").is_ok());
    }

    #[test]
    fn test_validate_flags_invalid() {
        let result = validate_regex_flags("gx");
        assert!(result.is_err(), "Invalid flag should be rejected");
    }

    #[test]
    fn test_apply_single_regex() {
        let result = apply_single_regex("hello world", "hello", "hi", "gi");
        assert_eq!(result, Ok(("hi world".to_string(), 1)));
    }

    #[test]
    fn test_apply_single_regex_invalid_pattern() {
        let result = apply_single_regex("hello world", "[unclosed", "hi", "gi");
        assert!(result.is_err(), "Invalid pattern should return error");
    }

    #[test]
    fn test_apply_non_global_replaces_once() {
        let scripts = vec![make_script("Replace once", "a", "b", "i", 1, true)];
        let result = apply_regex_scripts("a a a", &scripts);
        assert_eq!(
            result, "b a a",
            "Non-global should replace only first occurrence"
        );
    }

    #[test]
    fn test_apply_empty_scripts_list() {
        let result = apply_regex_scripts("hello", &[]);
        assert_eq!(
            result, "hello",
            "Empty scripts list should return input unchanged"
        );
    }

    #[test]
    fn test_pipe_chain_output_becomes_input() {
        let scripts = vec![
            make_script("Step 1", "foo", "bar", "g", 1, true),
            make_script("Step 2", "bar", "baz", "g", 2, true),
        ];
        let result = apply_regex_scripts("foo", &scripts);
        assert_eq!(result, "baz", "Pipe chain should transform foo→bar→baz");
    }
}
