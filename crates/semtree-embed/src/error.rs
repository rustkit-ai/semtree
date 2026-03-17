use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("model load failed: {0}")]
    ModelLoad(String),
    #[error("embed failed: {0}")]
    EmbedFailed(String),
}
