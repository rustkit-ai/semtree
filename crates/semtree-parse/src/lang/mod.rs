mod go;
mod javascript;
mod python;
mod rust;
pub(crate) mod shared;
mod typescript;

use semtree_core::{Chunk, Language};
use thiserror::Error;

use crate::extractor::Extractor;
use crate::parser::ParsedTree;

#[derive(Debug, Error)]
#[error("no extractor for language: {0}")]
pub struct ExtractError(pub Language);

pub fn extract(tree: &ParsedTree) -> Result<Vec<Chunk>, ExtractError> {
    match tree.language {
        Language::Rust => Ok(rust::RustExtractor.extract(tree)),
        Language::Python => Ok(python::PythonExtractor.extract(tree)),
        Language::JavaScript => Ok(javascript::JavaScriptExtractor.extract(tree)),
        Language::TypeScript => Ok(typescript::TypeScriptExtractor.extract(tree)),
        Language::Go => Ok(go::GoExtractor.extract(tree)),
        lang => Err(ExtractError(lang)),
    }
}
