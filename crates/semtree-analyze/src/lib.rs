use semtree_core::{Chunk, ChunkKind, Language};
use std::path::PathBuf;

/// Summary of a chunk's complexity metrics.
#[derive(Debug, Clone)]
pub struct ComplexityReport {
    pub name: String,
    pub path: PathBuf,
    pub start_line: usize,
    pub line_count: usize,
    /// Approximate cyclomatic complexity (keyword-based, 1-indexed).
    pub cyclomatic: usize,
    pub kind: ChunkKind,
}

/// Build a `ComplexityReport` for every chunk, sorted by `cyclomatic` descending.
pub fn analyze_chunks(chunks: &[Chunk]) -> Vec<ComplexityReport> {
    let mut reports: Vec<ComplexityReport> = chunks
        .iter()
        .map(|c| ComplexityReport {
            name: c.name.clone().unwrap_or_else(|| c.id.clone()),
            path: c.path.clone(),
            start_line: c.span.start_line + 1,
            line_count: c.content.lines().count(),
            cyclomatic: cyclomatic_complexity(&c.content, c.language),
            kind: c.kind.clone(),
        })
        .collect();

    reports.sort_by(|a, b| b.cyclomatic.cmp(&a.cyclomatic).then(b.line_count.cmp(&a.line_count)));
    reports
}

/// Return references to chunks whose `kind` is `Function` and whose content
/// has more than `threshold` lines.
pub fn find_large_functions(chunks: &[Chunk], threshold: usize) -> Vec<&Chunk> {
    chunks
        .iter()
        .filter(|c| c.kind == ChunkKind::Function && c.content.lines().count() > threshold)
        .collect()
}

/// Approximate cyclomatic complexity by counting branch keywords.
/// Returns at minimum 1 (the function itself is one path).
pub fn cyclomatic_complexity(content: &str, language: Language) -> usize {
    let keywords: &[&str] = match language {
        Language::Rust => &[
            " if ", " else ", " while ", " for ", " loop ", " match ",
            " && ", " || ", " => ",
        ],
        Language::Python => &[
            " if ", " elif ", "else:", " while ", " for ", " except",
            " and ", " or ",
        ],
        Language::TypeScript | Language::JavaScript => &[
            " if ", " else ", " while ", " for ", " switch ", " case ",
            " catch ", " && ", " || ", " ?? ",
        ],
        Language::Go => &[
            " if ", " else ", " for ", " switch ", " case ",
            " && ", " || ",
        ],
        Language::Unknown => &[],
    };

    let count: usize = keywords.iter().map(|kw| content.matches(kw).count()).sum();
    1 + count
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
            span: Span::new(
                0,
                content.len(),
                0,
                content.lines().count().saturating_sub(1),
            ),
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
                "fn b() {\n  if x { return 1; }\n  if y { return 2; }\n  x + y\n}\n",
            ),
            make_chunk(
                "medium_struct",
                ChunkKind::Struct,
                "struct S {\n  x: i32,\n}\n",
            ),
        ];

        let reports = analyze_chunks(&chunks);
        assert_eq!(reports.len(), 3);
        assert!(
            reports[0].cyclomatic >= reports[1].cyclomatic,
            "should be sorted descending by cyclomatic"
        );
    }

    #[test]
    fn test_find_large_functions() {
        let small = make_chunk("tiny", ChunkKind::Function, "fn a() {}\n");
        let large = make_chunk(
            "big",
            ChunkKind::Function,
            (0..20)
                .map(|i| format!("  let x{i} = {i};\n"))
                .collect::<String>()
                .as_str(),
        );
        let a_struct = make_chunk(
            "MyStruct",
            ChunkKind::Struct,
            (0..20)
                .map(|i| format!("  field{i}: i32,\n"))
                .collect::<String>()
                .as_str(),
        );

        let chunks = vec![small, large, a_struct];
        let large_fns = find_large_functions(&chunks, 5);

        assert_eq!(large_fns.len(), 1, "only the large function qualifies");
        assert_eq!(large_fns[0].name.as_deref(), Some("big"));
    }

    #[test]
    fn test_cyclomatic_complexity() {
        let simple = "fn a() { 1 + 2 }";
        assert_eq!(cyclomatic_complexity(simple, Language::Rust), 1);

        let branchy = "fn b() { if x { 1 } else { 2 } }";
        assert!(cyclomatic_complexity(branchy, Language::Rust) > 1);
    }
}
