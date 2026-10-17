use crate::ai::embeddings::EmbeddingRuntime;
use crate::db::{DbCommand, DbWorker};
use crate::domain::lorebook::LorebookActivationMode;
use crate::service::lorebook;
use crate::service::vector::cosine_similarity;
use tauri::State;
use tokio::sync::oneshot;

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

/// Per-entry semantic activation preview for one lorebook against a sample
/// text — the semantic counterpart of `test_lorebook_keywords`, using the
/// same runtime semantics (mode + threshold + role query + keyword fallback)
/// as `activate_lorebook_entries`. Requires embeddings to be configured.
#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SemanticTestResult {
    pub entry_id: String,
    pub keys: Vec<String>,
    pub activation_mode: String,
    /// Cosine similarity against the sample text (None when no vector).
    pub similarity: Option<f32>,
    pub threshold: f32,
    /// Whether the entry would activate right now.
    pub activated: bool,
    /// "keyword" | "semantic" | "always" — mirroring the runtime reason.
    pub reason: Option<String>,
}

#[tauri::command]
pub async fn test_semantic_activation(
    db: State<'_, DbWorker>,
    runtime: State<'_, EmbeddingRuntime>,
    lorebook_id: String,
    text: String,
) -> Result<Vec<SemanticTestResult>, String> {
    // Load the lorebook's entries.
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::LoadLorebookEntries {
        lorebook_id: lorebook_id.clone(),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    let entries = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {e}"))?
        .map_err(|e| e.to_string())?;

    // Embed the sample text as the "all roles" query.
    let sender = db.sender();
    let runtime = (*runtime).clone();
    let query = runtime.embed_query(&sender, &text).await?;

    // Load entry vectors for the current model.
    let loaded = runtime.resolve(&sender).await?;
    let namespace = format!("lorebook:{lorebook_id}");
    let (reply_tx, reply_rx) = oneshot::channel();
    db.send(DbCommand::LoadEmbeddings {
        namespace,
        model_key: Some(loaded.model_key.clone()),
        reply: reply_tx,
    })
    .map_err(|e| e.to_string())?;
    let vectors = reply_rx
        .await
        .map_err(|e| format!("DB worker dropped: {e}"))?
        .map_err(|e| e.to_string())?;
    let vector_map: std::collections::HashMap<String, Vec<f32>> = vectors
        .into_iter()
        .map(|record| (record.ref_id, record.vector))
        .collect();

    let text_lower = text.to_lowercase();
    let mut results = Vec::new();
    for entry in &entries {
        if !entry.enabled {
            continue;
        }
        let keyword_hit =
            lorebook::keyword_matches(&entry.keys, &entry.secondary_keys, &text_lower);

        let (similarity, activated, reason) =
            if entry.activation_state == crate::domain::lorebook::LorebookActivation::Always {
                (None, true, Some("always".to_string()))
            } else {
                match entry.activation_mode {
                    LorebookActivationMode::Keyword => (
                        None,
                        keyword_hit,
                        keyword_hit.then(|| "keyword".to_string()),
                    ),
                    LorebookActivationMode::Semantic | LorebookActivationMode::Hybrid => {
                        let sim = vector_map
                            .get(&entry.id)
                            .map(|v| cosine_similarity(v, &query));
                        let semantic_hit = sim.is_some_and(|s| s >= entry.semantic_threshold);
                        match entry.activation_mode {
                            LorebookActivationMode::Hybrid => {
                                let reason = if keyword_hit {
                                    "keyword"
                                } else if semantic_hit {
                                    "semantic"
                                } else {
                                    ""
                                };
                                (
                                    sim,
                                    keyword_hit || semantic_hit,
                                    (!reason.is_empty()).then(|| reason.to_string()),
                                )
                            }
                            // Preview-only nuance: semantic mode falls back to
                            // keyword when the entry has no vector (runtime does
                            // the same), otherwise requires the semantic hit.
                            _ => {
                                if semantic_hit {
                                    (sim, true, Some("semantic".to_string()))
                                } else if sim.is_none() {
                                    (sim, keyword_hit, keyword_hit.then(|| "keyword".to_string()))
                                } else {
                                    (sim, false, None)
                                }
                            }
                        }
                    }
                }
            };

        results.push(SemanticTestResult {
            entry_id: entry.id.clone(),
            keys: entry.keys.clone(),
            activation_mode: entry.activation_mode.to_string(),
            similarity,
            threshold: entry.semantic_threshold,
            activated,
            reason,
        });
    }

    // Most interesting first: activated, then by similarity.
    results.sort_by(|a, b| {
        b.activated.cmp(&a.activated).then(
            b.similarity
                .partial_cmp(&a.similarity)
                .unwrap_or(std::cmp::Ordering::Equal),
        )
    });

    Ok(results)
}
