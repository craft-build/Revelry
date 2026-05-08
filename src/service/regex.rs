use regex::RegexBuilder;
use crate::domain::regex_script::RegexScript;

/// Validates a regex pattern for ReDoS vulnerability per D-06.
/// Uses regex::RegexBuilder::size_limit(1_000_000) to cap backtracking.
/// Returns compiled Regex if safe, or error message if dangerous/invalid.
pub fn validate_regex_pattern(pattern: &str, flags: &str) -> Result<regex::Regex, String> {
    todo!("RED phase — not yet implemented")
}

/// Validates only that flags contain valid characters (i, g, m, s only).
pub fn validate_regex_flags(flags: &str) -> Result<(), String> {
    todo!("RED phase — not yet implemented")
}

/// Applies a list of regex scripts to input text per D-05.
/// Scripts run in sort_order sequence (pipe chain) — each script's output
/// becomes the next script's input. Disabled scripts are skipped.
pub fn apply_regex_scripts(text: &str, scripts: &[RegexScript]) -> String {
    todo!("RED phase — not yet implemented")
}

/// Applies a single regex find/replace to input text (for test panel preview).
pub fn apply_single_regex(text: &str, pattern: &str, replace: &str, flags: &str) -> Result<String, String> {
    todo!("RED phase — not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::regex_script::RegexScript;

    fn make_script(name: &str, find: &str, replace: &str, flags: &str, sort_order: i32, enabled: bool) -> RegexScript {
        let now = chrono::Local::now().naive_local();
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
        // A simple, safe pattern should compile successfully
        let result = validate_regex_pattern("hello", "gi");
        assert!(result.is_ok(), "Safe pattern should compile: {:?}", result);
    }

    #[test]
    fn test_validate_redos_pattern() {
        // Classic ReDoS pattern with nested quantifiers
        // The size_limit should cause compilation to fail
        let result = validate_regex_pattern("(a+)+$", "i");
        assert!(result.is_err(), "ReDoS pattern should be rejected: {:?}", result);
    }

    #[test]
    fn test_validate_invalid_syntax() {
        // Unclosed bracket should fail with syntax error
        let result = validate_regex_pattern("[unclosed", "i");
        assert!(result.is_err(), "Invalid syntax should be rejected: {:?}", result);
        // Error message should be informative
        let err = result.unwrap_err();
        assert!(!err.is_empty(), "Error message should not be empty");
    }

    #[test]
    fn test_apply_two_enabled_scripts_in_order() {
        // Two scripts in sort_order pipe chain
        let scripts = vec![
            make_script("Replace hello", "hello", "hi", "gi", 1, true),
            make_script("Replace world", "world", "earth", "gi", 2, true),
        ];

        let result = apply_regex_scripts("hello world", &scripts);
        assert_eq!(result, "hi earth", "Scripts should apply in sort_order pipe chain");
    }

    #[test]
    fn test_apply_skips_disabled_scripts() {
        // One enabled, one disabled
        let scripts = vec![
            make_script("Replace hello", "hello", "hi", "gi", 1, true),
            make_script("Replace world", "world", "earth", "gi", 2, false),
        ];

        let result = apply_regex_scripts("hello world", &scripts);
        assert_eq!(result, "hi world", "Disabled scripts should be skipped");
    }

    #[test]
    fn test_apply_global_and_case_insensitive() {
        // Flags "gi" should replace all occurrences, case-insensitive
        let scripts = vec![
            make_script("Replace Hello", "hello", "hi", "gi", 1, true),
        ];

        let result = apply_regex_scripts("Hello HELLO hello", &scripts);
        assert_eq!(result, "hi hi hi", "Global + case-insensitive should replace all");
    }

    #[test]
    fn test_apply_capture_group_replacement() {
        // Capture group with $1 replacement
        let scripts = vec![
            make_script("Wrap words", "(\\w+)", "[$1]", "g", 1, true),
        ];

        let result = apply_regex_scripts("hello world", &scripts);
        assert_eq!(result, "[hello] [world]", "Capture groups with $1 should work");
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
        assert_eq!(result, Ok("hi world".to_string()));
    }

    #[test]
    fn test_apply_single_regex_invalid_pattern() {
        let result = apply_single_regex("hello world", "[unclosed", "hi", "gi");
        assert!(result.is_err(), "Invalid pattern should return error");
    }

    #[test]
    fn test_apply_non_global_replaces_once() {
        // Without 'g' flag, only first occurrence is replaced
        let scripts = vec![
            make_script("Replace once", "a", "b", "i", 1, true),
        ];

        let result = apply_regex_scripts("a a a", &scripts);
        assert_eq!(result, "b a a", "Non-global should replace only first occurrence");
    }

    #[test]
    fn test_apply_empty_scripts_list() {
        let result = apply_regex_scripts("hello", &[]);
        assert_eq!(result, "hello", "Empty scripts list should return input unchanged");
    }

    #[test]
    fn test_pipe_chain_output_becomes_input() {
        // Each script's output becomes the next script's input
        let scripts = vec![
            make_script("Step 1", "foo", "bar", "g", 1, true),
            make_script("Step 2", "bar", "baz", "g", 2, true),
        ];

        let result = apply_regex_scripts("foo", &scripts);
        assert_eq!(result, "baz", "Pipe chain should transform foo→bar→baz");
    }
}
