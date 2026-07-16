use thiserror::Error;

pub type AppResult<T> = Result<T, AppError>;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("configuration error: {0}")]
    Config(String),
    #[error("audio error: {0}")]
    Audio(String),
    #[error("shortcut error: {0}")]
    Shortcut(String),
    #[error("transcription error: {0}")]
    Transcription(String),
    #[error("model error: {0}")]
    Model(String),
    #[error("text insertion error: {0}")]
    TextInsertion(String),
    #[error("permission error: {0}")]
    Permission(String),
    #[error("platform error: {0}")]
    Platform(String),
    #[error("state error: {0}")]
    State(String),
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("network error: {0}")]
    Network(#[from] reqwest::Error),
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
