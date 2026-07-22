//! Language-agnostic chunk extraction driven by tree-sitter query files.
//!
//! Each language is a `queries/<lang>.scm` file (data, not code): patterns whose
//! capture name maps to a [`ChunkKind`] via [`kind_for`], with an `@name` capture
//! for the identifier. Adding a language is a grammar dependency, a `.scm` file,
//! and one arm in [`query_for`] - no per-language Rust.
//!
//! The one exception is C/C++ functions, whose name is buried in a declarator
//! chain rather than a `name` field: those queries capture the declarator as
//! `@decl` and [`extract`] resolves the identifier generically.

pub(crate) mod shared;

use std::sync::OnceLock;

use semtree_core::{Chunk, ChunkKind, Language};
use tree_sitter::{Query, QueryCursor, StreamingIterator};

use crate::parser::ParsedTree;
use shared::{find_identifier, make_chunk};

/// Maps a query capture name to the chunk kind it represents. Captures that
/// don't name a kind (`name`, `decl`) return `None` and are handled separately.
fn kind_for(capture: &str) -> Option<ChunkKind> {
    Some(match capture {
        "function" => ChunkKind::Function,
        "method" => ChunkKind::Method,
        "class" => ChunkKind::Class,
        "struct" => ChunkKind::Struct,
        "enum" => ChunkKind::Enum,
        "trait" => ChunkKind::Trait,
        "impl" => ChunkKind::Impl,
        "module" => ChunkKind::Module,
        _ => return None,
    })
}

/// The compiled query for a language, built once and reused. Returns `None` for
/// languages without a query (only [`Language::Unknown`] in practice, since the
/// parser rejects unknown languages before extraction).
fn query_for(language: Language) -> Option<&'static Query> {
    macro_rules! cached {
        ($cell:ident, $lang:expr, $src:literal) => {{
            static $cell: OnceLock<Query> = OnceLock::new();
            Some($cell.get_or_init(|| {
                Query::new(&$lang, include_str!($src)).expect(concat!("invalid query: ", $src))
            }))
        }};
    }
    match language {
        Language::Rust => cached!(RUST, tree_sitter_rust::LANGUAGE.into(), "queries/rust.scm"),
        Language::Python => {
            cached!(
                PY,
                tree_sitter_python::LANGUAGE.into(),
                "queries/python.scm"
            )
        }
        Language::JavaScript => cached!(
            JS,
            tree_sitter_javascript::LANGUAGE.into(),
            "queries/javascript.scm"
        ),
        Language::TypeScript => cached!(
            TS,
            tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into(),
            "queries/typescript.scm"
        ),
        Language::Go => cached!(GO, tree_sitter_go::LANGUAGE.into(), "queries/go.scm"),
        Language::Java => cached!(JAVA, tree_sitter_java::LANGUAGE.into(), "queries/java.scm"),
        Language::C => cached!(C, tree_sitter_c::LANGUAGE.into(), "queries/c.scm"),
        Language::Cpp => cached!(CPP, tree_sitter_cpp::LANGUAGE.into(), "queries/cpp.scm"),
        Language::CSharp => cached!(
            CSHARP,
            tree_sitter_c_sharp::LANGUAGE.into(),
            "queries/csharp.scm"
        ),
        Language::Ruby => cached!(RUBY, tree_sitter_ruby::LANGUAGE.into(), "queries/ruby.scm"),
        Language::Php => cached!(PHP, tree_sitter_php::LANGUAGE_PHP.into(), "queries/php.scm"),
        Language::Tsx => cached!(
            TSX,
            tree_sitter_typescript::LANGUAGE_TSX.into(),
            "queries/tsx.scm"
        ),
        Language::Kotlin => cached!(
            KOTLIN,
            tree_sitter_kotlin_ng::LANGUAGE.into(),
            "queries/kotlin.scm"
        ),
        Language::Scala => cached!(
            SCALA,
            tree_sitter_scala::LANGUAGE.into(),
            "queries/scala.scm"
        ),
        Language::Swift => cached!(
            SWIFT,
            tree_sitter_swift::LANGUAGE.into(),
            "queries/swift.scm"
        ),
        Language::OCaml => cached!(
            OCAML,
            tree_sitter_ocaml::LANGUAGE_OCAML.into(),
            "queries/ocaml.scm"
        ),
        Language::Solidity => cached!(
            SOLIDITY,
            tree_sitter_solidity::LANGUAGE.into(),
            "queries/solidity.scm"
        ),
        Language::Lua => cached!(LUA, tree_sitter_lua::LANGUAGE.into(), "queries/lua.scm"),
        Language::Zig => cached!(ZIG, tree_sitter_zig::LANGUAGE.into(), "queries/zig.scm"),
        Language::Elisp => cached!(
            ELISP,
            tree_sitter_elisp::LANGUAGE.into(),
            "queries/elisp.scm"
        ),
        Language::Unknown => None,
    }
}

/// Extract chunks from a parsed tree by running its language's query. Each match
/// yields one definition capture (its kind) and an optional name.
pub fn extract(tree: &ParsedTree) -> Vec<Chunk> {
    let Some(query) = query_for(tree.language) else {
        return Vec::new();
    };
    let capture_names = query.capture_names();
    let source = tree.source.as_slice();

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(query, tree.root(), source);
    let mut chunks = Vec::new();

    while let Some(m) = matches.next() {
        let mut def: Option<(tree_sitter::Node, ChunkKind)> = None;
        let mut name: Option<String> = None;

        for cap in m.captures {
            match capture_names[cap.index as usize] {
                "name" => name = Some(tree.node_text(&cap.node).to_string()),
                "decl" => name = find_identifier(&cap.node, tree),
                other => {
                    if let Some(kind) = kind_for(other) {
                        def = Some((cap.node, kind));
                    }
                }
            }
        }

        if let Some((node, kind)) = def {
            chunks.push(make_chunk(&node, tree, tree.language, kind, name));
        }
    }

    chunks
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shipped query must compile against its grammar. Query errors are a
    /// runtime failure, so this turns them into a build-time (CI) failure.
    #[test]
    fn all_queries_compile() {
        let languages = [
            Language::Rust,
            Language::Python,
            Language::JavaScript,
            Language::TypeScript,
            Language::Tsx,
            Language::Go,
            Language::Java,
            Language::C,
            Language::Cpp,
            Language::CSharp,
            Language::Ruby,
            Language::Php,
            Language::Kotlin,
            Language::Scala,
            Language::Swift,
            Language::OCaml,
            Language::Solidity,
            Language::Lua,
            Language::Zig,
            Language::Elisp,
        ];
        for lang in languages {
            assert!(query_for(lang).is_some(), "{lang} query failed to compile");
        }
    }
}
