use std::path::Path;

use semtree_core::{Chunk, ChunkKind, Language, Span};
use tree_sitter::Node;

use crate::parser::ParsedTree;

/// Stable content hash. blake3 is deterministic across releases and platforms,
/// so a chunk keeps the same id from one run - and one toolchain - to the next.
pub fn chunk_hash(s: &str) -> String {
    blake3::hash(s.as_bytes()).to_hex().to_string()
}

/// Id for a chunk at `span` in `path`. Location-based, not content-based: two
/// files with an identical function get distinct ids, so one never overwrites or
/// deletes the other in the store. Matches the "hash of path + span" contract.
pub fn chunk_id(path: &Path, span: &Span) -> String {
    chunk_hash(&format!(
        "{}:{}-{}",
        path.display(),
        span.start_byte,
        span.end_byte
    ))
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
    // Provisional id from the span alone (unique within this tree). Once the
    // path is known, `finalize_paths` re-stamps it so it is unique across files.
    let id = chunk_id(Path::new(""), &span);

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

/// Attach `path` to freshly extracted chunks and give each a path-scoped id.
/// Extraction runs before the source path is known; this stamps it in.
pub fn finalize_paths(chunks: &mut [Chunk], path: &Path) {
    for chunk in chunks {
        chunk.id = chunk_id(path, &chunk.span);
        chunk.path = path.to_path_buf();
    }
}

/// First identifier-like node in a subtree. C and C++ functions bury their name
/// in a `declarator` chain rather than a `name` field, so the extractor descends
/// from that declarator to pull the function name out.
pub fn find_identifier(node: &Node<'_>, tree: &ParsedTree) -> Option<String> {
    match node.kind() {
        "identifier"
        | "field_identifier"
        | "type_identifier"
        | "qualified_identifier"
        | "operator_name"
        | "destructor_name" => return Some(tree.node_text(node).to_string()),
        _ => {}
    }
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        if let Some(name) = find_identifier(&child, tree) {
            return Some(name);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: usize, end: usize) -> Span {
        Span::new(start, end, 0, 0)
    }

    #[test]
    fn same_location_is_stable() {
        let p = Path::new("src/a.rs");
        assert_eq!(chunk_id(p, &span(0, 10)), chunk_id(p, &span(0, 10)));
    }

    #[test]
    fn identical_code_in_different_files_gets_distinct_ids() {
        // The bug this guards against: content-based ids collide across files,
        // so one file's chunk overwrites or deletes the other's in the store.
        let s = span(0, 10);
        assert_ne!(
            chunk_id(Path::new("src/a.rs"), &s),
            chunk_id(Path::new("src/b.rs"), &s),
        );
    }

    #[test]
    fn a_rename_changes_the_id() {
        // A renamed file's chunks get new ids, so the old path's ids are cleanly
        // deletable without touching the new file's chunks.
        let s = span(0, 10);
        assert_ne!(
            chunk_id(Path::new("old/name.rs"), &s),
            chunk_id(Path::new("new/name.rs"), &s),
        );
    }
}
