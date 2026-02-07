use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("Audio error: {0}")]
    Audio(String),

    #[error("Whisper error: {0}")]
    Whisper(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Settings error: {0}")]
    Settings(String),

    #[error("History error: {0}")]
    History(String),

    #[error("Hotkey error: {0}")]
    Hotkey(String),

    #[error("Output error: {0}")]
    Output(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Other(String),
}

impl Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
