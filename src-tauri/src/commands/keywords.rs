use crate::service::lorebook;

/// Tests whether lorebook keywords match against a given text using the same
/// canonical semantics as runtime activation (SillyTavern-style: primary keys
/// required, AND within a comma-separated group and OR across groups; secondary
/// keys gate activation but never activate alone). `Always` entries are
/// considered a constant match by the runtime, so this test command reports all
/// primary keys as matched when `activation_state == "always"` (the frontend
/// passes this to keep preview == runtime).
#[derive(serde::Serialize)]
pub struct KeywordTestResult {
    pub entry_id: String,
    pub matched_keys: Vec<String>,
}

#[tauri::command]
pub async fn test_lorebook_keywords(
    keys: Vec<String>,
    secondary_keys: Vec<String>,
    text: String,
    activation_state: Option<String>,
) -> Result<KeywordTestResult, String> {
    let text_lower = text.to_lowercase();
    let mut matched_keys = Vec::new();

    // `Always` entries activate unconditionally at runtime.
    if activation_state.as_deref() == Some("always") {
        matched_keys = keys
            .iter()
            .filter(|k| !k.trim().is_empty())
            .cloned()
            .collect();
    } else {
        for key in &keys {
            let sub_keys: Vec<&str> = key
                .split(',')
                .map(|s| s.trim())
                .filter(|s| !s.is_empty())
                .collect();
            if sub_keys.is_empty() {
                continue;
            }
            let all_match = sub_keys
                .iter()
                .all(|sk| text_lower.contains(&sk.to_lowercase()));
            if all_match {
                matched_keys.push(key.clone());
            }
        }

        // Secondary keys gate activation (any must match) but never activate
        // alone — matching the runtime semantics in lorebook::keyword_matches.
        if !secondary_keys.is_empty() && !matched_keys.is_empty() {
            let sec_match = secondary_keys
                .iter()
                .any(|sk| !sk.trim().is_empty() && text_lower.contains(&sk.to_lowercase()));
            if !sec_match {
                matched_keys.clear();
            }
        }
    }

    // Sanity check: keep this command consistent with the runtime helper.
    let runtime_matches = lorebook::keyword_matches(&keys, &secondary_keys, &text);
    let always = activation_state.as_deref() == Some("always");
    let expected = if always { true } else { runtime_matches };
    debug_assert_eq!(!matched_keys.is_empty(), expected);

    let entry_id = format!("test-{}", keys.join("-"));

    Ok(KeywordTestResult {
        entry_id,
        matched_keys,
    })
}
