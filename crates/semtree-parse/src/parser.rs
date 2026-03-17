use semtree_core::Language;
use tree_sitter::{Node, Parser, Tree};

use crate::ParseError;

pub struct ParsedTree {
    pub language: Language,
    pub source: Vec<u8>,
    pub tree: Tree,
}

impl ParsedTree {
    pub fn root(&self) -> Node<'_> {
        self.tree.root_node()
    }

    pub fn source_str(&self) -> &str {
        std::str::from_utf8(&self.source).unwrap_or("")
    }

    pub fn node_text(&self, node: &Node<'_>) -> &str {
        node.utf8_text(&self.source).unwrap_or("")
    }
}

pub struct SemtreeParser;

impl SemtreeParser {
    pub fn parse(source: &str, language: Language) -> Result<ParsedTree, ParseError> {
        let ts_language = ts_language(language)?;
        let mut parser = Parser::new();
        parser.set_language(&ts_language).expect("valid language");

        let tree = parser.parse(source, None).ok_or(ParseError::ParseFailed)?;

        Ok(ParsedTree {
            language,
            source: source.as_bytes().to_vec(),
            tree,
        })
    }

    pub fn parse_file(path: &std::path::Path) -> Result<ParsedTree, ParseError> {
        let language = Language::from_path(path);
        if language == Language::Unknown {
            return Err(ParseError::UnsupportedLanguage(language));
        }
        let source = std::fs::read_to_string(path)?;
        Self::parse(&source, language)
    }
}

fn ts_language(lang: Language) -> Result<tree_sitter::Language, ParseError> {
    match lang {
        Language::Rust => Ok(tree_sitter_rust::LANGUAGE.into()),
        Language::Python => Ok(tree_sitter_python::LANGUAGE.into()),
        Language::JavaScript => Ok(tree_sitter_javascript::LANGUAGE.into()),
        Language::TypeScript => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        Language::Go => Ok(tree_sitter_go::LANGUAGE.into()),
        Language::Unknown => Err(ParseError::UnsupportedLanguage(lang)),
    }
}
