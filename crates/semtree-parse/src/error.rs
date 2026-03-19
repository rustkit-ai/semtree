use semtree_core::Language;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ParseError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(Language),
    #[error("parse failed: tree-sitter returned no tree")]
    ParseFailed,
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("extract error: {0}")]
    Extract(String),
}
