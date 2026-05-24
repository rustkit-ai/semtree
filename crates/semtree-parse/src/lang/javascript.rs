use semtree_core::{Chunk, ChunkKind, Language};
use tree_sitter::Node;

use super::shared::{make_chunk, walk};
use crate::extractor::Extractor;
use crate::parser::ParsedTree;

pub struct JavaScriptExtractor;

impl Extractor for JavaScriptExtractor {
    fn extract(&self, tree: &ParsedTree) -> Vec<Chunk> {
        let mut chunks = Vec::new();
        walk(&tree.root(), tree, &mut chunks, &visit);
        chunks
    }
}

fn visit(node: &Node<'_>, tree: &ParsedTree) -> Option<Chunk> {
    match node.kind() {
        "function_declaration" | "generator_function_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::JavaScript,
                ChunkKind::Function,
                name,
            ))
        }
        "method_definition" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::JavaScript,
                ChunkKind::Method,
                name,
            ))
        }
        "class_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::JavaScript,
                ChunkKind::Class,
                name,
            ))
        }
        "export_statement" => {
            let decl = node.child_by_field_name("declaration")?;
            let (kind, name) = extract_decl_info(decl, tree)?;
            Some(make_chunk(node, tree, Language::JavaScript, kind, name))
        }
        // `const foo = function() {}` or `const foo = () => {}`
        "lexical_declaration" | "variable_declaration" => {
            let declarator = node.named_child(0)?;
            let value = declarator.child_by_field_name("value")?;
            if matches!(value.kind(), "function" | "arrow_function") {
                let name = declarator
                    .child_by_field_name("name")
                    .map(|n| tree.node_text(&n).to_string());
                Some(make_chunk(
                    node,
                    tree,
                    Language::JavaScript,
                    ChunkKind::Function,
                    name,
                ))
            } else {
                None
            }
        }
        _ => None,
    }
}

fn extract_decl_info(
    node: Node<'_>,
    tree: &ParsedTree,
) -> Option<(ChunkKind, Option<String>)> {
    match node.kind() {
        "function_declaration" | "generator_function_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some((ChunkKind::Function, name))
        }
        "class_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some((ChunkKind::Class, name))
        }
        _ => None,
    }
}
