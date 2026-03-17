use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use semtree_core::{Chunk, ChunkKind, Language, Span};
use tree_sitter::Node;

use crate::parser::ParsedTree;

pub fn chunk_hash(s: &str) -> String {
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    format!("{:x}", h.finish())
}

pub fn make_chunk(
    node: &Node<'_>,
    tree: &ParsedTree,
    language: Language,
    kind: ChunkKind,
    name: Option<String>,
) -> Chunk {
    let span = Span::new(
        node.start_byte(),
        node.end_byte(),
        node.start_position().row,
        node.end_position().row,
    );
    let content = tree.node_text(node).to_string();
    let id = chunk_hash(&content);

    Chunk {
        id,
        path: std::path::PathBuf::new(),
        language,
        kind,
        name,
        content,
        span,
        doc: None,
    }
}

pub fn walk<F>(node: &Node<'_>, tree: &ParsedTree, chunks: &mut Vec<Chunk>, visit: &F)
where
    F: Fn(&Node<'_>, &ParsedTree) -> Option<Chunk>,
{
    if let Some(chunk) = visit(node, tree) {
        chunks.push(chunk);
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(&child, tree, chunks, visit);
    }
}
