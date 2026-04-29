use thiserror::Error;

#[derive(Debug, Error)]
pub enum SttError {
    #[error("STT model not loaded")]
    ModelNotReady,
    #[error("Transcription inference failed: {0}")]
    InferenceFailed(String),
    #[error("Audio buffer is empty")]
    EmptyBuffer,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("STT error: {0}")]
    Stt(#[from] SttError),
    #[error("Config error: {0}")]
    Config(String),
    #[error("Hotkey error: {0}")]
    Hotkey(String),
    #[error("Audio error: {0}")]
    Audio(String),
    #[error("Injection error: {0}")]
    Injection(String),
    #[error("Permission error: {0}")]
    Permission(String),
    #[error("Unknown error: {0}")]
    Unknown(String),
}
