use semtree_core::{Chunk, ChunkKind, Language};
use tree_sitter::Node;

use super::shared::{make_chunk, walk};
use crate::extractor::Extractor;
use crate::parser::ParsedTree;

pub struct GoExtractor;

impl Extractor for GoExtractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        walk(&tree.root(), tree, &mut chunks, &visit);
        chunks
    }
}

fn visit(node: &Node<'_>, tree: &ParsedTree) -> Option<Chunk> {
    match node.kind() {
        "function_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::Go,
                ChunkKind::Function,
                name,
            ))
        }
        "method_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::Go,
                ChunkKind::Method,
                name,
            ))
        }
        "type_declaration" => {
            // Go: type_declaration -> type_spec { name, type }
            let spec = node.named_child(0)?;
            if spec.kind() != "type_spec" {
                return None;
            }
            let type_node = spec.child_by_field_name("type");
            let kind = match type_node.as_ref().map(|n| n.kind()) {
                Some("interface_type") => ChunkKind::Trait,
                _ => ChunkKind::Struct,
            };
            let name = spec
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(node, tree, Language::Go, kind, name))
        }
        _ => None,
    }
}
