use thiserror::Error;

#[derive(Debug, Error)]
pub enum TreeError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("parse failed: tree-sitter returned no tree")]
    ParseFailed,
}
