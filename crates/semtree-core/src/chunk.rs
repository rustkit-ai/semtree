use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::{Language, Span};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChunkKind {
    Function,
    Method,
    Struct,
    Enum,
    Trait,
    Impl,
    Module,
    Class,
    File,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chunk {
    /// Unique identifier (hash of path + span)
    pub id: String,
    /// Source file
    pub path: PathBuf,
    /// Programming language
    pub language: Language,
    /// Kind of code construct
    pub kind: ChunkKind,
    /// Name of the construct (e.g. function name)
    pub name: Option<String>,
    /// Raw source text
    pub content: String,
    /// Location in source file
    pub span: Span,
    /// Docstring / leading comment if any
    pub doc: Option<String>,
}
