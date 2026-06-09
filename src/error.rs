use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("failed to read config file: {0}")]
    Io(#[from] std::io::Error),
    #[error("failed to parse config file: {0}")]
    Parse(#[from] toml::de::Error),
}

#[derive(Debug, Error)]
pub enum AgentError {
    #[error("agent output is invalid: {0}")]
    InvalidOutput(String),
    #[error("model call failed: {0}")]
    Model(#[from] ModelError),
}

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("unsupported model provider: {0}")]
    UnsupportedProvider(String),
    #[error("{provider} provider error: {message}")]
    Provider { provider: String, message: String },
}

#[derive(Debug, Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
    #[error("failed to serialize json: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid persisted enum value `{value}` for {kind}")]
    InvalidEnum { kind: &'static str, value: String },
    #[error("invalid persisted timestamp `{0}`")]
    InvalidTimestamp(String),
}

#[derive(Debug, Error)]
pub enum WorkflowError {
    #[error("storage error: {0}")]
    Storage(#[from] StorageError),
    #[error("novel `{0}` was not found")]
    NovelNotFound(String),
    #[error("chapter {chapter} for novel `{novel_id}` was not found")]
    ChapterNotFound { novel_id: String, chapter: u32 },
    #[error("invalid workflow input: {0}")]
    InvalidInput(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
