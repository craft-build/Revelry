/// Tests whether lorebook keywords match against a given text.
/// Returns which entries matched and which of their keys triggered the match.
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
) -> Result<KeywordTestResult, String> {
    let text_lower = text.to_lowercase();
    let mut matched_keys = Vec::new();

    for key in &keys {
        if key.is_empty() {
            continue;
        }
        // Keys are comma-separated within a single key string
        // Each sub-key must match (AND logic within a key, OR between keys)
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

    // Secondary keys use OR logic — if any secondary key matches AND at least one primary key matches
    if !secondary_keys.is_empty() && !matched_keys.is_empty() {
        let sec_match = secondary_keys.iter().any(|sk| {
            if sk.is_empty() {
                return false;
            }
            text_lower.contains(&sk.to_lowercase())
        });
        if !sec_match {
            matched_keys.clear();
        }
    }

    // Generate a deterministic entry_id for this test
    let entry_id = format!("test-{}", keys.join("-"));

    Ok(KeywordTestResult {
        entry_id,
        matched_keys,
    })
}
