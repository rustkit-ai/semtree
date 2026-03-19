use semtree_core::{ChunkKind, Language};
use tree_sitter::Node;

use super::shared::{make_chunk, walk};
use crate::extractor::Extractor;
use crate::parser::ParsedTree;
use semtree_core::Chunk;

pub struct RustExtractor;

impl Extractor for RustExtractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        walk(&tree.root(), tree, &mut chunks, &visit);
        chunks
    }
}

fn visit(node: &Node<'_>, tree: &ParsedTree) -> Option<Chunk> {
    let kind = match node.kind() {
        "function_item" => ChunkKind::Function,
        "struct_item" => ChunkKind::Struct,
        "enum_item" => ChunkKind::Enum,
        "trait_item" => ChunkKind::Trait,
        "impl_item" => ChunkKind::Impl,
        "mod_item" => ChunkKind::Module,
        _ => return None,
    };
    let name = node
        .child_by_field_name("name")
        .map(|n| tree.node_text(&n).to_string());
    Some(make_chunk(node, tree, Language::Rust, kind, name))
}
