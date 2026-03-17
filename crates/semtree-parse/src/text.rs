use std::path::Path;

use semtree_core::{Chunk, ChunkKind, Language, Span};

use crate::lang::shared::chunk_hash;

/// File types we handle as plain text (no AST)
pub fn is_text_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()).unwrap_or(""),
        "md" | "txt" | "rst" | "json" | "yaml" | "yml" | "toml" | "env" | "csv" | "html" | "xml" | "graphql"
    )
}

/// Split plain text into overlapping chunks of ~`chunk_lines` lines.
pub fn chunk_text(path: &Path, source: &str, chunk_lines: usize, overlap: usize) -> Vec<Chunk> {
    let lines: Vec<&str> = source.lines().collect();
    if lines.is_empty() {
        return vec![];
    }

    let step = chunk_lines.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let mut start = 0;

    while start < lines.len() {
        let end = (start + chunk_lines).min(lines.len());
        let content = lines[start..end].join("\n");

        let span = Span::new(
            0, // byte offsets not meaningful for text chunks
            0,
            start,
            end - 1,
        );

        chunks.push(Chunk {
            id: chunk_hash(&format!("{}:{start}", path.display())),
            path: path.to_path_buf(),
            language: Language::Unknown,
            kind: ChunkKind::File,
            name: Some(format!(
                "{} (lines {}-{})",
                path.file_name().and_then(|n| n.to_str()).unwrap_or("?"),
                start + 1,
                end
            )),
            content,
            span,
            doc: None,
        });

        if end == lines.len() {
            break;
        }
        start += step;
    }

    chunks
}
