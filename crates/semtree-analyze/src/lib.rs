// Augmented static analysis for semtree
// Future: dead code detection, call graph extraction, cyclomatic complexity, dependency graphs.

use semtree_core::{Chunk, ChunkKind};

/// Summary of a chunk's complexity metrics.
#[derive(Debug, Clone)]
pub struct ComplexityReport {
    /// Name of the construct (e.g. function name).
    pub name: String,
    /// Number of lines in the chunk's content.
    pub line_count: usize,
    /// Kind of the chunk.
    pub kind: ChunkKind,
}

/// Build a `ComplexityReport` for every chunk, sorted by `line_count` descending.
pub fn analyze_chunks(chunks: &[Chunk]) -> Vec<ComplexityReport> {
    let mut reports: Vec<ComplexityReport> = chunks
        .iter()
        .map(|c| ComplexityReport {
            name: c.name.clone().unwrap_or_else(|| c.id.clone()),
            line_count: c.content.lines().count(),
            kind: c.kind.clone(),
        })
        .collect();

    reports.sort_by(|a, b| b.line_count.cmp(&a.line_count));
    reports
}

/// Return references to chunks whose `kind` is `Function` and whose content
/// has more than `threshold` lines.
pub fn find_large_functions<'a>(chunks: &'a [Chunk], threshold: usize) -> Vec<&'a Chunk> {
    chunks
        .iter()
        .filter(|c| c.kind == ChunkKind::Function && c.content.lines().count() > threshold)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use semtree_core::{ChunkKind, Language, Span};
    use std::path::PathBuf;

    fn make_chunk(name: &str, kind: ChunkKind, content: &str) -> Chunk {
        Chunk {
            id: name.to_string(),
            path: PathBuf::from("test.rs"),
            language: Language::Rust,
            kind,
            name: Some(name.to_string()),
            content: content.to_string(),
            span: Span::new(0, content.len(), 0, content.lines().count().saturating_sub(1)),
            doc: None,
        }
    }

    #[test]
    fn test_analyze_chunks_sorted_descending() {
        let chunks = vec![
            make_chunk("short_fn", ChunkKind::Function, "fn a() {}\n"),
            make_chunk(
                "long_fn",
                ChunkKind::Function,
                "fn b() {\n  let x = 1;\n  let y = 2;\n  x + y\n}\n",
            ),
            make_chunk("medium_struct", ChunkKind::Struct, "struct S {\n  x: i32,\n}\n"),
        ];

        let reports = analyze_chunks(&chunks);
        assert_eq!(reports.len(), 3);
        assert!(
            reports[0].line_count >= reports[1].line_count,
            "should be sorted descending"
        );
        assert!(
            reports[1].line_count >= reports[2].line_count,
            "should be sorted descending"
        );
    }

    #[test]
    fn test_find_large_functions() {
        let small = make_chunk("tiny", ChunkKind::Function, "fn a() {}\n");
        let large = make_chunk(
            "big",
            ChunkKind::Function,
            (0..20).map(|i| format!("  let x{i} = {i};\n")).collect::<String>().as_str(),
        );
        let a_struct = make_chunk(
            "MyStruct",
            ChunkKind::Struct,
            (0..20).map(|i| format!("  field{i}: i32,\n")).collect::<String>().as_str(),
        );

        let chunks = vec![small, large, a_struct];
        let large_fns = find_large_functions(&chunks, 5);

        assert_eq!(large_fns.len(), 1, "only the large function qualifies");
        assert_eq!(large_fns[0].name.as_deref(), Some("big"));
    }
}
