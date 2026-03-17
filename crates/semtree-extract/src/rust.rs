use semtree_core::{Chunk, ChunkKind, Language, Span};
use semtree_tree::ParsedTree;
use tree_sitter::Node;

use crate::extractor::Extractor;

pub struct RustExtractor;

impl Extractor for RustExtractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        let root = tree.root();
        walk(&root, tree, &mut chunks);
        chunks
    }
}

fn walk(node: &Node<'_>, tree: &ParsedTree, chunks: &mut Vec<Chunk>) {
    let kind = match node.kind() {
        "function_item" => Some(ChunkKind::Function),
        "struct_item" => Some(ChunkKind::Struct),
        "enum_item" => Some(ChunkKind::Enum),
        "trait_item" => Some(ChunkKind::Trait),
        "impl_item" => Some(ChunkKind::Impl),
        "mod_item" => Some(ChunkKind::Module),
        _ => None,
    };

    if let Some(kind) = kind {
        let name = node
            .child_by_field_name("name")
            .map(|n| tree.node_text(&n).to_string());

        let span = Span::new(
            node.start_byte(),
            node.end_byte(),
            node.start_position().row,
            node.end_position().row,
        );

        let content = tree.node_text(node).to_string();
        let id = format!("{:x}", md5_hash(&content));

        chunks.push(Chunk {
            id,
            path: std::path::PathBuf::new(),
            language: Language::Rust,
            kind,
            name,
            content,
            span,
            doc: None,
        });
    }

    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(&child, tree, chunks);
    }
}

fn md5_hash(s: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    s.hash(&mut h);
    h.finish()
}
