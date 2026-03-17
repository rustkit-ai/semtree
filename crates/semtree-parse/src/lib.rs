mod error;
mod parser;
mod extractor;
mod lang;

pub use error::ParseError;
pub use parser::{ParsedTree, SemtreeParser};
pub use extractor::Extractor;

use semtree_core::{Chunk, Language};

pub fn parse_and_extract(source: &str, language: Language) -> Result<Vec<Chunk>, ParseError> {
    let tree = SemtreeParser::parse(source, language)?;
    lang::extract(&tree).map_err(|e| ParseError::Extract(e.to_string()))
}

pub fn parse_and_extract_file(path: &std::path::Path) -> Result<Vec<Chunk>, ParseError> {
    let tree = SemtreeParser::parse_file(path)?;
    lang::extract(&tree).map_err(|e| ParseError::Extract(e.to_string()))
}
