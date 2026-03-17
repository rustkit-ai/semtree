mod rust;

use semtree_core::{Chunk, Language};
use crate::extractor::Extractor;
use crate::parser::ParsedTree;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("no extractor for language: {0}")]
pub struct ExtractError(pub Language);

pub fn extract(tree: &ParsedTree) -> Result<Vec<Chunk>, ExtractError> {
    match tree.language {
        Language::Rust => Ok(rust::RustExtractor.extract(tree)),
        lang => Err(ExtractError(lang)),
    }
}
