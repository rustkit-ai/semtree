use semtree_core::{Chunk, ChunkKind, Language};
use tree_sitter::Node;

use super::shared::{make_chunk, walk};
use crate::extractor::Extractor;
use crate::parser::ParsedTree;

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
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::TypeScript,
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
                Language::TypeScript,
                ChunkKind::Method,
                name,
            ))
        }
        "class_declaration" | "abstract_class_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::TypeScript,
                ChunkKind::Class,
                name,
            ))
        }
        "interface_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::TypeScript,
                ChunkKind::Trait,
                name,
            ))
        }
        "type_alias_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::TypeScript,
                ChunkKind::Struct,
                name,
            ))
        }
        "enum_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some(make_chunk(
                node,
                tree,
                Language::TypeScript,
                ChunkKind::Enum,
                name,
            ))
        }
        "export_statement" => {
            // `export function foo() {}` or `export class Bar {}`
            let decl = node.child_by_field_name("declaration")?;
            let (kind, name) = extract_decl_info(decl, tree)?;
            Some(make_chunk(node, tree, Language::TypeScript, kind, name))
        }
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
                    Language::TypeScript,
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
        "class_declaration" | "abstract_class_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some((ChunkKind::Class, name))
        }
        "interface_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some((ChunkKind::Trait, name))
        }
        "type_alias_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some((ChunkKind::Struct, name))
        }
        "enum_declaration" => {
            let name = node
                .child_by_field_name("name")
                .map(|n| tree.node_text(&n).to_string());
            Some((ChunkKind::Enum, name))
        }
        _ => None,
    }
}
