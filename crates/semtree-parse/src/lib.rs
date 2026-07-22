//! **Tree-sitter parsing and chunk extraction for semtree.**
//!
//! Turns source files into structured [`Chunk`]s -
//! functions, methods, structs, classes - aligned to real syntax boundaries
//! instead of arbitrary line windows. Supports Rust, Python, JavaScript,
//! TypeScript, TSX, Go, Java, C, C++, C#, Ruby, PHP, Kotlin, Scala, Swift,
//! OCaml, Solidity, Lua, Zig and Emacs Lisp; non-code text falls back to
//! fixed-size windows.
//!
//! Each language is a tree-sitter query in `src/lang/queries/`; adding one is a
//! grammar dependency plus a `.scm` file, no per-language Rust.
//!
//! ```no_run
//! use semtree_parse::extract_file;
//!
//! let chunks = extract_file(std::path::Path::new("src/lib.rs"))?;
//! for c in &chunks {
//!     println!("{:?} {:?}", c.kind, c.name);
//! }
//! # Ok::<(), semtree_parse::ParseError>(())
//! ```

mod error;
mod lang;
mod parser;
mod text;

pub use error::ParseError;
pub use parser::{ParsedTree, SemtreeParser};
pub use text::{chunk_text, is_text_file};

use semtree_core::{Chunk, Language};

pub fn parse_and_extract(source: &str, language: Language) -> Result<Vec<Chunk>, ParseError> {
    let tree = SemtreeParser::parse(source, language)?;
    Ok(lang::extract(&tree))
}

pub fn parse_and_extract_file(path: &std::path::Path) -> Result<Vec<Chunk>, ParseError> {
    let tree = SemtreeParser::parse_file(path)?;
    let mut chunks = lang::extract(&tree);
    lang::shared::finalize_paths(&mut chunks, path);
    Ok(chunks)
}

/// Extract chunks from any supported file - code or plain text.
pub fn extract_file(path: &std::path::Path) -> Result<Vec<Chunk>, ParseError> {
    if is_text_file(path) {
        let source = std::fs::read_to_string(path)?;
        return Ok(chunk_text(path, &source, 40, 5));
    }
    parse_and_extract_file(path)
}
