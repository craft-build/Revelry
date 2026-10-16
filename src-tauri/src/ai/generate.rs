use crate::domain::provider::{ProviderConfig, ProviderType};

use futures::StreamExt;
use rig::agent::{Agent, MultiTurnStreamItem};
use rig::completion::Message;
use rig::streaming::{Item, StreamEvent};
use tauri::ipc::Channel;
use tokio_util::sync::CancellationToken;

use super::providers::{build_agent, with_native_client};
use super::util::classify_error;
use super::AiService;
use super::{GenerateCharacterChunk, GeneratedCharacterData, GenerationParams};

/// System prompt for the character generation command.
/// Instructs the LLM to output a CharacterCardV2-compatible JSON object.
const GENERATE_CHARACTER_SYSTEM_PROMPT: &str = r#"You are a character creation assistant for an AI roleplay application. The user will describe a character idea, and you must generate a complete character card in JSON format.

Output ONLY valid JSON matching this schema — no explanation, no markdown, no commentary:

{
  "name": "Character's full name",
  "description": "A vivid 2-3 sentence description of the character's appearance, nature, and what makes them interesting",
  "personality": "Detailed personality description — traits, quirks, emotional patterns, how they behave with others",
  "scenario": "The starting scenario for the roleplay — set the scene, establish tension or intrigue, give the user a reason to engage",
  "first_mes": "The character's first message to the user — written in character, using *actions* and dialogue. Make it compelling and open-ended",
  "mes_example": "Example dialogue showing the character's speech patterns — use <START> to separate exchanges, {{user}} and {{char}} as placeholders",
  "system_prompt": "The system prompt that defines how the AI should portray this character — writing style, key behaviors, dos and don'ts",
  "post_history_instructions": "Optional instruction appended after chat history — leave empty string if not needed",
  "creator_notes": "Notes for the user about the character — suggested usage, tone, potential story directions",
  "tags": ["relevant", "descriptive", "tags"],
  "alternate_greetings": []
}

Rules:
- The "name" field is REQUIRED and must be non-empty
- "first_mes" should be at least 2-3 paragraphs, written in first person with *actions* in asterisks
- "personality" and "scenario" should be rich and detailed — aim for 3-5+ sentences each
- "mes_example" should demonstrate at least 2 exchanges
- "tags" should include 3-6 relevant genre/theme tags
- "alternate_greetings" can be empty array if not needed
- All string fields default to empty string if not applicable

- Be creative and specific — avoid generic templates"#;

impl AiService {
    /// Stream an AI-generated character card via a Tauri Channel.
    /// Tokens are streamed for preview, then the full response is parsed into
    /// GeneratedCharacterData on completion.
    pub fn stream_generate_character(
        provider: &ProviderConfig,
        idea: &str,
        api_key: &str,
        params: GenerationParams,
        channel: Channel<GenerateCharacterChunk>,
        cancel: CancellationToken,
    ) {
        let provider_type = provider.provider_type.clone();
        let base_url = provider.base_url.clone();
        let model_name = provider.model_name.clone();
        let key = api_key.to_string();
        let user_prompt = idea.to_string();

        let system_prompt = GENERATE_CHARACTER_SYSTEM_PROMPT.to_string();

        tokio::spawn(async move {
            if let Err(e) = Self::run_generate_stream(
                &provider_type,
                base_url.as_deref(),
                &model_name,
                &system_prompt,
                &user_prompt,
                &key,
                &params,
                &channel,
                &cancel,
            )
            .await
            {
                let _ = channel.send(GenerateCharacterChunk::Error {
                    error: e.clone(),
                    error_type: classify_error(&e),
                });
            }
        });
    }

    /// Dispatches to the correct rig provider client and runs the generation stream.
    /// On completion, parses the accumulated JSON into GeneratedCharacterData.
    #[allow(clippy::too_many_arguments)]
    async fn run_generate_stream(
        provider_type: &ProviderType,
        base_url: Option<&str>,
        model_name: &str,
        system_prompt: &str,
        idea: &str,
        api_key: &str,
        params: &GenerationParams,
        channel: &Channel<GenerateCharacterChunk>,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        let additional = Self::build_additional_params(provider_type, params);

        let prompt = Message::user(idea);

        with_native_client!(provider_type, base_url, api_key, model_name, model, {
            let agent = build_agent(model, system_prompt, params, &additional);
            Self::build_and_stream_generation(&agent, prompt, channel, cancel).await
        })
    }

    /// Stream a response from the built agent and parse it as GeneratedCharacterData.
    async fn build_and_stream_generation(
        agent: &Agent,
        prompt: Message,
        channel: &Channel<GenerateCharacterChunk>,
        cancel: &CancellationToken,
    ) -> Result<(), String> {
        let mut stream = agent.prompt(prompt).history(Vec::<Message>::new()).stream();

        let channel_clone = channel.clone();
        let cancel_clone = cancel.clone();

        let mut accumulated = String::new();

        tokio::select! {
            _ = cancel_clone.cancelled() => {
                let _ = channel_clone.send(GenerateCharacterChunk::Error {
                    error: "Generation cancelled.".to_string(),
                    error_type: "cancelled".to_string(),
                });
            }
            _ = async {
                while let Some(item) = stream.next().await {
                    match item {
                        Ok(MultiTurnStreamItem::StreamAssistantItem(
                            Item::Event(StreamEvent::Text { text, .. }),
                        )) => {
                            accumulated.push_str(&text);
                            if channel_clone
                                .send(GenerateCharacterChunk::Token { content: text })
                                .is_err()
                            {
                                break;
                            }
                        }
                        Ok(MultiTurnStreamItem::FinalResponse(_)) => {
                            // Parse accumulated text as GeneratedCharacterData
                            let parsed = Self::parse_generated_character(&accumulated);
                            match parsed {
                                Ok(character) => {
                                    let _ = channel_clone.send(GenerateCharacterChunk::Complete {
                                        character: Box::new(character),
                                    });
                                }
                                Err(parse_err) => {
                                    let _ = channel_clone.send(GenerateCharacterChunk::Error {
                                        error: format!(
                                            "Failed to parse generated character: {}",
                                            parse_err
                                        ),
                                        error_type: "parse_error".to_string(),
                                    });
                                }
                            }
                            break;
                        }
                        Ok(_) => {}
                        Err(e) => {
                            let err_str = e.to_string();
                            let _ = channel_clone.send(GenerateCharacterChunk::Error {
                                error: err_str.clone(),
                                error_type: classify_error(&err_str),
                            });
                            break;
                        }
                    }
                }
            } => {}
        }

        Ok(())
    }

    /// Parse the LLM's raw text output into GeneratedCharacterData.
    /// Extracts JSON (the LLM may wrap it in markdown fences or prefix text),
    /// then coerces each field liberally.
    fn parse_generated_character(raw: &str) -> Result<GeneratedCharacterData, String> {
        // Try to find JSON in the response — may be wrapped in ```json ... ```
        let json_text = Self::extract_json(raw)?;

        // Parse into a generic Value first, then coerce each field.
        // LLMs may output objects/arrays where strings are expected, so we
        // flatten/convert liberally rather than failing on type mismatches.
        let value: serde_json::Value =
            serde_json::from_str(&json_text).map_err(|e| format!("JSON parse error: {}", e))?;

        let obj = value
            .as_object()
            .ok_or_else(|| "Generated JSON is not an object".to_string())?;

        let coerce_string = |key: &str| -> String {
            match obj.get(key) {
                None | Some(serde_json::Value::Null) => String::new(),
                Some(serde_json::Value::String(s)) => s.clone(),
                Some(serde_json::Value::Number(n)) => n.to_string(),
                Some(serde_json::Value::Bool(b)) => b.to_string(),
                Some(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|v| {
                        v.as_str()
                            .map(|s| s.to_string())
                            .or_else(|| Some(v.to_string()))
                    })
                    .collect::<Vec<_>>()
                    .join(", "),
                Some(serde_json::Value::Object(map)) => {
                    // Flatten nested objects into key=value pairs
                    map.iter()
                        .map(|(k, v)| {
                            format!(
                                "{}={}",
                                k,
                                match v {
                                    serde_json::Value::String(s) => s.clone(),
                                    other => other.to_string().trim_matches('"').to_string(),
                                }
                            )
                        })
                        .collect::<Vec<_>>()
                        .join("; ")
                }
            }
        };

        let coerce_string_array = |key: &str| -> Vec<String> {
            match obj.get(key) {
                None | Some(serde_json::Value::Null) => Vec::new(),
                Some(serde_json::Value::Array(arr)) => arr
                    .iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect(),
                Some(serde_json::Value::String(s)) => {
                    // Comma-separated string → array
                    s.split(',')
                        .map(|t| t.trim().to_string())
                        .filter(|t| !t.is_empty())
                        .collect()
                }
                _ => Vec::new(),
            }
        };

        Ok(GeneratedCharacterData {
            name: coerce_string("name"),
            description: coerce_string("description"),
            personality: coerce_string("personality"),
            scenario: coerce_string("scenario"),
            first_mes: coerce_string("first_mes"),
            mes_example: coerce_string("mes_example"),
            system_prompt: coerce_string("system_prompt"),
            post_history_instructions: coerce_string("post_history_instructions"),
            creator_notes: coerce_string("creator_notes"),
            tags: coerce_string_array("tags"),
            alternate_greetings: coerce_string_array("alternate_greetings"),
        })
    }

    /// Extract JSON from potentially markdown-wrapped LLM output.
    /// Tries code fences, then brace matching, then a raw parse.
    fn extract_json(raw: &str) -> Result<String, String> {
        // Strategy 1: Extract from ```json ... ``` fences
        if let Some(start) = raw.find("```json") {
            let json_start = start + 7; // length of "```json"
                                        // Skip past the opening fence line (to newline or end)
            let content_start = raw[json_start..]
                .find('\n')
                .map(|i| json_start + i + 1)
                .unwrap_or(json_start);
            // Search for closing ``` after the opening fence content
            if let Some(end) = raw[content_start..].find("```") {
                let end_abs = content_start + end;
                let json = raw[content_start..end_abs].trim();
                if !json.is_empty() {
                    return Ok(json.to_string());
                }
            }
        }
        // Try generic code fences (``` ... ```)
        if let Some(start) = raw.find("```") {
            let after_fence = start + 3;
            // Skip optional language tag on first line
            let json_start = raw[after_fence..]
                .find('\n')
                .map(|i| after_fence + i + 1)
                .unwrap_or(after_fence);
            if let Some(end) = raw[json_start..].rfind("```") {
                let json = raw[json_start..json_start + end].trim();
                if !json.is_empty() {
                    return Ok(json.to_string());
                }
            }
        }

        // Strategy 2: Brace matching
        if let Some(first_brace) = raw.find('{') {
            if let Some(last_brace) = raw.rfind('}') {
                if last_brace > first_brace {
                    let json = &raw[first_brace..=last_brace];
                    if serde_json::from_str::<serde_json::Value>(json).is_ok() {
                        return Ok(json.to_string());
                    }
                }
            }
        }

        // Strategy 3: Raw parse
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            return Ok(trimmed.to_string());
        }

        Err("No JSON content found in LLM response".to_string())
    }
}
