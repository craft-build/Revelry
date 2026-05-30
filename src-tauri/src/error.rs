use std::path::PathBuf;

/// Unified error type for the application.
///
/// Using a structured error type instead of `String` provides:
/// - Pattern-matchable variants for the frontend
/// - Consistent error messages
/// - Proper error chaining via `source`
/// - `Serialize` for Tauri IPC (errors are serialized as their display string)
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Database error: {0}")]
    Database(String),

    #[error("Database worker unavailable: {0}")]
    DbWorkerUnavailable(String),

    #[error("Record not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Card import/export error: {0}")]
    CardIo(#[from] anyhow::Error),

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("AI service error: {0}")]
    AiService(String),

    #[error("Path traversal detected: {0}")]
    PathTraversal(PathBuf),

    #[error("Mutex poisoned: {0}")]
    Poisoned(String),

    #[error("{0}")]
    Other(String),
}

impl AppError {
    pub fn db(msg: impl Into<String>) -> Self {
        Self::Database(msg.into())
    }

    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn keyring(msg: impl Into<String>) -> Self {
        Self::Keyring(msg.into())
    }

    pub fn ai(msg: impl Into<String>) -> Self {
        Self::AiService(msg.into())
    }

    pub fn poisoned(msg: impl Into<String>) -> Self {
        Self::Poisoned(msg.into())
    }

    pub fn other(msg: impl Into<String>) -> Self {
        Self::Other(msg.into())
    }
}

/// Convert a poisoned mutex error into AppError.
impl<T> From<std::sync::PoisonError<T>> for AppError {
    fn from(e: std::sync::PoisonError<T>) -> Self {
        Self::Poisoned(e.to_string())
    }
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

/// Convenience alias for results using AppError.
pub type AppResult<T> = Result<T, AppError>;
