//! Rolling story-summary generation (memory layer, Phase 11). Uses the
//! conversation's own chat provider for a background non-streaming completion
//! — works with zero embedding setup.

use crate::domain::conversation::{Message, MessageRole};
use crate::domain::provider::{ProviderConfig, ProviderType};

use super::providers::{build_agent, with_native_client};
use super::GenerationParams;

/// Minimum total messages before summary maintenance kicks in at all — short
/// conversations never need a summary.
pub const SUMMARY_MIN_TOTAL_MESSAGES: usize = 30;

/// Messages accumulated beyond `summary_upto` before regenerating.
pub const SUMMARY_TRIGGER_NEW_MESSAGES: usize = 12;

/// Timeout for the background summary completion call.
const SUMMARY_TIMEOUT_SECS: u64 = 120;

const SUMMARY_PREAMBLE: &str = "You maintain a running summary of an ongoing roleplay story. \
Write a concise summary (at most 500 words) covering the durable facts: key events, decisions, \
discoveries, relationship changes, promises, and unresolved threads. Write in plain prose, \
third person. Do not add new events. Preserve names exactly. When given a previous summary, \
carry its content forward, updated with the new messages.";

/// Generates (or extends) the conversation summary. `messages` must be the
/// messages strictly after `summary_upto` (oldest-first). Fire-and-forget at
/// the call sites — errors are logged, never surfaced mid-chat.
pub async fn generate_summary(
    provider: &ProviderConfig,
    model_name: &str,
    api_key: &str,
    previous_summary: Option<&str>,
    messages: &[Message],
) -> Result<String, String> {
    let mut transcript = String::new();
    for message in messages {
        if message.content.trim().is_empty() {
            continue;
        }
        let speaker = match message.role {
            MessageRole::User => "User",
            MessageRole::Assistant => "Character",
            MessageRole::System => continue, // Steering noise — not story.
        };
        transcript.push_str(&format!("{speaker}: {}\n", message.content));
    }
    if transcript.is_empty() {
        return Err("no messages to summarize".to_string());
    }

    let prompt = match previous_summary {
        Some(prev) if !prev.trim().is_empty() => format!(
            "Previous summary:\n{prev}\n\nNew messages since the summary:\n{transcript}\n\n\
             Produce the updated summary."
        ),
        _ => format!("Story so far:\n{transcript}\n\nProduce the summary."),
    };

    let params = GenerationParams {
        temperature: Some(0.3),
        max_tokens: Some(1024),
        ..Default::default()
    };
    let provider_type = provider.provider_type.clone();
    let base_url = provider.base_url.clone();

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(SUMMARY_TIMEOUT_SECS),
        async {
            with_native_client!(
                &provider_type,
                base_url.as_deref(),
                api_key,
                model_name,
                model,
                {
                    let agent =
                        build_agent(model, SUMMARY_PREAMBLE, &params, &serde_json::Map::new());
                    let response = agent
                        .prompt(prompt.as_str())
                        .await
                        .map_err(|e| e.to_string())?;
                    let output = response.output();
                    if output.trim().is_empty() {
                        Err("summary came back empty".to_string())
                    } else {
                        Ok(output)
                    }
                }
            )
        },
    )
    .await
    .map_err(|_| "summary generation timed out".to_string())?;

    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summary_constants_sane() {
        assert!(SUMMARY_MIN_TOTAL_MESSAGES > SUMMARY_TRIGGER_NEW_MESSAGES);
        assert!(SUMMARY_TRIGGER_NEW_MESSAGES >= 4);
    }
}
