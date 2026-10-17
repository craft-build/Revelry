use crate::db::DbCommand;
use crate::domain::conversation::Message as DomainMessage;
use crate::domain::lorebook::LorebookActivationDetail;
use serde::Serialize;

/// Runtime generation params (D-03). `max_tokens` is `u64` here since API
/// values are non-negative; the domain type uses `i64` for SQLite storage.
#[derive(Clone, Debug, Default)]
pub struct GenerationParams {
    pub temperature: Option<f64>,
    pub top_p: Option<f64>,
    pub max_tokens: Option<u64>,
    pub frequency_penalty: Option<f64>,
    pub presence_penalty: Option<f64>,
    pub stop_sequences: Option<Vec<String>>,
}

impl From<crate::domain::provider::GenerationParams> for GenerationParams {
    fn from(params: crate::domain::provider::GenerationParams) -> Self {
        Self {
            temperature: params.temperature,
            top_p: params.top_p,
            max_tokens: params
                .max_tokens
                .and_then(|v| if v > 0 { Some(v as u64) } else { None }),
            frequency_penalty: params.frequency_penalty,
            presence_penalty: params.presence_penalty,
            stop_sequences: params.stop_sequences,
        }
    }
}

/// Budget info from PromptEngine to include in StreamChunk::Complete.
#[derive(Clone, Copy, Debug, Default)]
pub struct BudgetInfo {
    pub tokens_used: usize,
    pub tokens_limit: usize,
    pub truncated_count: usize,
    /// Memory-layer outcome for this prompt (summary / recall).
    pub memory: Option<MemoryInfo>,
}

/// Memory-layer outcome surfaced on the Complete chunk.
#[derive(Clone, Copy, Debug, Default, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MemoryInfo {
    pub summary_included: bool,
    pub recalled_count: usize,
    pub external_count: usize,
}

/// A chunk of data from a streaming AI response.
/// Serialized via Tauri Channel to the frontend using tagged enum pattern.
#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum StreamChunk {
    Token {
        content: String,
    },
    Complete {
        message_id: String,
        content: String,
        stop_reason: String,
        tokens_used: usize,
        tokens_limit: usize,
        truncated_count: usize,
        cancelled: bool,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        lorebook_details: Vec<LorebookActivationDetail>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        memory: Option<MemoryInfo>,
    },
    Error {
        error: String,
        error_type: String,
    },
}

/// Partial character data returned by the generator. Creative fields only —
/// id, created_at, updated_at are assigned on save.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct GeneratedCharacterData {
    pub name: String,
    pub description: String,
    pub personality: String,
    pub scenario: String,
    pub first_mes: String,
    pub mes_example: String,
    pub system_prompt: String,
    pub post_history_instructions: String,
    pub creator_notes: String,
    pub tags: Vec<String>,
    pub alternate_greetings: Vec<String>,
}

/// A chunk of data from the character generation stream.
/// Mirrors the frontend GenerateCharacterChunk discriminated union exactly.
#[derive(Clone, Serialize)]
#[serde(
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    tag = "event",
    content = "data"
)]
pub enum GenerateCharacterChunk {
    Token {
        content: String,
    },
    Complete {
        character: Box<GeneratedCharacterData>,
    },
    Error {
        error: String,
        error_type: String,
    },
}

/// Handle for persisting a streamed assistant message from within the AI layer.
/// Carries a clone of the DbWorker command sender (std mpsc — sends are
/// non-blocking) and the placeholder message saved before streaming started.
/// Content updates are throttled by the caller; each flush is fire-and-forget.
#[derive(Clone)]
pub struct MessagePersistence {
    pub sender: std::sync::mpsc::Sender<DbCommand>,
    pub message: DomainMessage,
}

impl MessagePersistence {
    pub fn new(sender: std::sync::mpsc::Sender<DbCommand>, message: DomainMessage) -> Self {
        Self { sender, message }
    }

    /// Fire-and-forget content flush. The oneshot receiver is dropped
    /// immediately; send errors (worker gone) are benign and ignored.
    pub fn flush(&self, content: &str) {
        let (reply_tx, _reply_rx) = tokio::sync::oneshot::channel();
        let _ = self.sender.send(DbCommand::UpdateMessageContent {
            id: self.message.id.clone(),
            content: content.to_string(),
            reply: reply_tx,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stream_chunk_serde_token() {
        let chunk = StreamChunk::Token {
            content: "Hello".to_string(),
        };
        let json = serde_json::to_string(&chunk).expect("serialize failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(parsed["event"], "token");
        assert_eq!(parsed["data"]["content"], "Hello");
    }

    #[test]
    fn test_stream_chunk_serde_complete() {
        let chunk = StreamChunk::Complete {
            message_id: "msg-123".to_string(),
            content: "Full response".to_string(),
            stop_reason: "end".to_string(),
            tokens_used: 4200,
            tokens_limit: 128000,
            truncated_count: 2,
            lorebook_details: Vec::new(),
            cancelled: false,
            memory: None,
        };
        let json = serde_json::to_string(&chunk).expect("serialize failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(parsed["event"], "complete");
        assert_eq!(parsed["data"]["messageId"], "msg-123");
        assert_eq!(parsed["data"]["tokensUsed"], 4200);
        assert_eq!(parsed["data"]["tokensLimit"], 128000);
        assert_eq!(parsed["data"]["truncatedCount"], 2);
        assert_eq!(parsed["data"]["cancelled"], false);
        assert!(
            parsed["data"].get("lorebookDetails").is_none()
                || parsed["data"]["lorebookDetails"]
                    .as_array()
                    .unwrap()
                    .is_empty()
        );
    }

    #[test]
    fn test_stream_chunk_serde_complete_cancelled() {
        let chunk = StreamChunk::Complete {
            message_id: "msg-cancel".to_string(),
            content: "Partial".to_string(),
            stop_reason: "cancelled".to_string(),
            tokens_used: 10,
            tokens_limit: 8000,
            truncated_count: 0,
            lorebook_details: Vec::new(),
            cancelled: true,
            memory: None,
        };
        let json = serde_json::to_string(&chunk).expect("serialize failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(parsed["event"], "complete");
        assert_eq!(parsed["data"]["messageId"], "msg-cancel");
        assert_eq!(parsed["data"]["content"], "Partial");
        assert_eq!(parsed["data"]["stopReason"], "cancelled");
        assert_eq!(parsed["data"]["cancelled"], true);
    }

    #[test]
    fn test_stream_chunk_serde_error() {
        let chunk = StreamChunk::Error {
            error: "Something went wrong".to_string(),
            error_type: "provider_error".to_string(),
        };
        let json = serde_json::to_string(&chunk).expect("serialize failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("deserialize failed");
        assert_eq!(parsed["event"], "error");
        assert_eq!(parsed["data"]["error"], "Something went wrong");
        assert_eq!(parsed["data"]["errorType"], "provider_error");
    }

    #[test]
    fn test_generation_params_default() {
        let params = GenerationParams::default();
        assert!(params.temperature.is_none());
        assert!(params.top_p.is_none());
        assert!(params.max_tokens.is_none());
        assert!(params.frequency_penalty.is_none());
        assert!(params.presence_penalty.is_none());
        assert!(params.stop_sequences.is_none());
    }
}
