use semtree_core::{Chunk, ChunkKind, Language};
use tree_sitter::Node;

use super::shared::{make_chunk, walk};
use crate::extractor::Extractor;
use crate::parser::ParsedTree;

pub struct PythonExtractor;

impl Extractor for PythonExtractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        walk(&tree.root(), tree, &mut chunks, &visit);
        chunks
    }
}

fn visit(node: &Node<'_>, tree: &ParsedTree) -> Option<Chunk> {
    match node.kind() {
        "function_definition" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::Python,
                ChunkKind::Function,
                name,
            ))
        }
        "class_definition" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::Python,
                ChunkKind::Class,
                name,
            ))
        }
        "decorated_definition" => {
            // unwrap to inner function or class
            let inner = node.child_by_field_name("definition")?;
            let kind = match inner.kind() {
                "function_definition" => ChunkKind::Function,
                "class_definition" => ChunkKind::Class,
                _ => return None,
            };
            let name = inner
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(node, tree, Language::Python, kind, name))
        }
        _ => None,
    }
}
