mod error;
mod parser;
mod extractor;
mod lang;
mod text;

pub use error::ParseError;
pub use parser::{ParsedTree, SemtreeParser};
pub use extractor::Extractor;
pub use text::{chunk_text, is_text_file};

use semtree_core::{Chunk, Language};

pub fn parse_and_extract(source: &str, language: Language) -> Result<Vec<Chunk>, ParseError> {
    let tree = SemtreeParser::parse(source, language)?;
    lang::extract(&tree).map_err(|e| ParseError::Extract(e.to_string()))
}

pub fn parse_and_extract_file(path: &std::path::Path) -> Result<Vec<Chunk>, ParseError> {
    let tree = SemtreeParser::parse_file(path)?;
    lang::extract(&tree).map_err(|e| ParseError::Extract(e.to_string()))
}

/// Extract chunks from any supported file — code or plain text.
pub fn extract_file(path: &std::path::Path) -> Result<Vec<Chunk>, ParseError> {
    if is_text_file(path) {
        let source = std::fs::read_to_string(path)?;
        return Ok(chunk_text(path, &source, 40, 5));
    }
    parse_and_extract_file(path)
}
