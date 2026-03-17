use semtree_core::{Chunk, ChunkKind, Language};
use tree_sitter::Node;

use crate::extractor::Extractor;
use crate::parser::ParsedTree;
use super::shared::{make_chunk, walk};

pub struct TypeScriptExtractor;

impl Extractor for TypeScriptExtractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        walk(&tree.root(), tree, &mut chunks, &visit);
        chunks
    }
}

fn visit(node: &Node<'_>, tree: &ParsedTree) -> Option<Chunk> {
    match node.kind() {
        "function_declaration" => {
            let name = node.child_by_field_name("name").map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(node, tree, Language::TypeScript, ChunkKind::Function, name))
        }
        "method_definition" => {
            let name = node.child_by_field_name("name").map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(node, tree, Language::TypeScript, ChunkKind::Method, name))
        }
        "class_declaration" => {
            let name = node.child_by_field_name("name").map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(node, tree, Language::TypeScript, ChunkKind::Class, name))
        }
        "interface_declaration" => {
            let name = node.child_by_field_name("name").map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(node, tree, Language::TypeScript, ChunkKind::Trait, name))
        }
        "type_alias_declaration" => {
            let name = node.child_by_field_name("name").map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(node, tree, Language::TypeScript, ChunkKind::Struct, name))
        }
        "lexical_declaration" | "variable_declaration" => {
            let declarator = node.named_child(0)?;
            let value = declarator.child_by_field_name("value")?;
            if matches!(value.kind(), "function" | "arrow_function") {
                let name = declarator
                    .child_by_field_name("name")
                    .map(|n| tree.node_text(&n).to_string());
                Some(make_chunk(node, tree, Language::TypeScript, ChunkKind::Function, name))
            } else {
                None
            }
        }
        _ => None,
    }
}
