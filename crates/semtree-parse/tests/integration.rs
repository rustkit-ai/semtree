use semtree_core::{ChunkKind, Language};
use semtree_parse::{chunk_text, parse_and_extract};
use std::path::Path;

// ── Rust ─────────────────────────────────────────────────────────────────────

#[test]
fn test_rust_extracts_function_struct_impl() {
    let source = r#"
pub struct Foo {
    x: i32,
}

impl Foo {
    pub fn new(x: i32) -> Self {
        Self { x }
    }
}

pub fn standalone() -> i32 {
    42
}
"#;
    let chunks = parse_and_extract(source, Language::Rust).expect("parse rust");
    assert!(!chunks.is_empty(), "should have at least one chunk");

    let kinds: Vec<&ChunkKind> = chunks.iter().map(|c| &c.kind).collect();

    assert!(
        kinds.contains(&&ChunkKind::Function),
        "should contain a function_item: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&&ChunkKind::Struct),
        "should contain a struct_item: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&&ChunkKind::Impl),
        "should contain an impl_item: {:?}",
        kinds
    );

    // Check names
    let names: Vec<&str> = chunks.iter().filter_map(|c| c.name.as_deref()).collect();
    assert!(names.contains(&"Foo"), "should have struct named Foo");
    assert!(
        names.contains(&"standalone"),
        "should have fn named standalone"
    );
}

// ── Python ────────────────────────────────────────────────────────────────────

#[test]
fn test_python_extracts_function_and_class() {
    let source = r#"
class Animal:
    def __init__(self, name):
        self.name = name

def greet(name: str) -> str:
    return f"Hello, {name}"
"#;
    let chunks = parse_and_extract(source, Language::Python).expect("parse python");
    assert!(!chunks.is_empty(), "should have at least one chunk");

    let kinds: Vec<&ChunkKind> = chunks.iter().map(|c| &c.kind).collect();
    assert!(
        kinds.contains(&&ChunkKind::Function),
        "should contain function_definition: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&&ChunkKind::Class),
        "should contain class_definition: {:?}",
        kinds
    );

    let names: Vec<&str> = chunks.iter().filter_map(|c| c.name.as_deref()).collect();
    assert!(names.contains(&"Animal"), "should have class named Animal");
    assert!(names.contains(&"greet"), "should have fn named greet");
}

// ── TypeScript ────────────────────────────────────────────────────────────────

#[test]
fn test_typescript_extracts_function_and_interface() {
    let source = r#"
interface Greeter {
    greet(name: string): string;
}

function hello(name: string): string {
    return `Hello, ${name}`;
}
"#;
    let chunks = parse_and_extract(source, Language::TypeScript).expect("parse typescript");
    assert!(!chunks.is_empty(), "should have at least one chunk");

    let kinds: Vec<&ChunkKind> = chunks.iter().map(|c| &c.kind).collect();
    assert!(
        kinds.contains(&&ChunkKind::Function),
        "should contain function_declaration: {:?}",
        kinds
    );
    // interface_declaration maps to ChunkKind::Trait
    assert!(
        kinds.contains(&&ChunkKind::Trait),
        "should contain interface_declaration (as Trait): {:?}",
        kinds
    );

    let names: Vec<&str> = chunks.iter().filter_map(|c| c.name.as_deref()).collect();
    assert!(names.contains(&"hello"), "should have fn named hello");
    assert!(
        names.contains(&"Greeter"),
        "should have interface named Greeter"
    );
}

// ── JavaScript ────────────────────────────────────────────────────────────────

#[test]
fn test_javascript_extracts_function_and_class() {
    let source = r#"
class Dog {
    constructor(name) {
        this.name = name;
    }
    speak() {
        return `${this.name} barks`;
    }
}

function fetch(url) {
    return null;
}
"#;
    let chunks = parse_and_extract(source, Language::JavaScript).expect("parse javascript");
    assert!(!chunks.is_empty(), "should have at least one chunk");

    let kinds: Vec<&ChunkKind> = chunks.iter().map(|c| &c.kind).collect();
    assert!(
        kinds.contains(&&ChunkKind::Function),
        "should contain function_declaration: {:?}",
        kinds
    );
    assert!(
        kinds.contains(&&ChunkKind::Class),
        "should contain class_declaration: {:?}",
        kinds
    );

    let names: Vec<&str> = chunks.iter().filter_map(|c| c.name.as_deref()).collect();
    assert!(names.contains(&"Dog"), "should have class named Dog");
    assert!(names.contains(&"fetch"), "should have fn named fetch");
}

// ── Go ────────────────────────────────────────────────────────────────────────

#[test]
fn test_go_extracts_function_declaration() {
    let source = r#"
package main

import "fmt"

func Greet(name string) string {
    return fmt.Sprintf("Hello, %s", name)
}

func Add(a, b int) int {
    return a + b
}
"#;
    let chunks = parse_and_extract(source, Language::Go).expect("parse go");
    assert!(!chunks.is_empty(), "should have at least one chunk");

    let kinds: Vec<&ChunkKind> = chunks.iter().map(|c| &c.kind).collect();
    assert!(
        kinds.contains(&&ChunkKind::Function),
        "should contain function_declaration: {:?}",
        kinds
    );

    let names: Vec<&str> = chunks.iter().filter_map(|c| c.name.as_deref()).collect();
    assert!(names.contains(&"Greet"), "should have fn named Greet");
    assert!(names.contains(&"Add"), "should have fn named Add");
}

// ── chunk_text (markdown) ─────────────────────────────────────────────────────

#[test]
fn test_chunk_text_markdown() {
    // Build a markdown text with 100 lines so we get multiple chunks
    // with default chunk_lines=40 and overlap=5: step = 35
    // chunks at: 0, 35, 70 → 3 chunks expected
    let source: String = (1..=100)
        .map(|i| format!("Line {i}: some content here."))
        .collect::<Vec<_>>()
        .join("\n");

    let path = Path::new("test.md");
    let chunks = chunk_text(path, &source, 40, 5);

    assert!(!chunks.is_empty(), "should produce at least one chunk");
    // With 100 lines, chunk_lines=40, step=35: starts at 0, 35, 70 → 3 chunks
    assert_eq!(
        chunks.len(),
        3,
        "expected 3 chunks for 100 lines with chunk_lines=40 overlap=5"
    );

    for chunk in &chunks {
        assert_eq!(
            chunk.kind,
            ChunkKind::File,
            "text chunks should have kind File"
        );
        assert!(
            !chunk.content.is_empty(),
            "chunk content should not be empty"
        );
    }
}
