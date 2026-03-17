mod extractor;
mod rust;

pub use extractor::Extractor;

use semtree_core::{Chunk, Language};
use semtree_tree::ParsedTree;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ExtractError {
    #[error("no extractor for language: {0}")]
    UnsupportedLanguage(Language),
}

pub fn extract(tree: &ParsedTree) -> Result<Vec<Chunk>, ExtractError> {
    match tree.language {
        Language::Rust => Ok(rust::RustExtractor.extract(tree)),
        lang => Err(ExtractError::UnsupportedLanguage(lang)),
    }
}
