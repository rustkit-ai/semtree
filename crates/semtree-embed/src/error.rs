use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("model load failed: {0}")]
    ModelLoad(String),
    #[error("embed failed: {0}")]
    EmbedFailed(String),
    #[error("http error: {0}")]
    Http(String),
    #[error("api key missing - set {0} env var or embed.api_key in .semtree.toml")]
    MissingApiKey(String),
}
