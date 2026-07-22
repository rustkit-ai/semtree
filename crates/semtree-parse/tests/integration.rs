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

// ── Helpers for the newer languages ──────────────────────────────────────────

fn extract(source: &str, lang: Language) -> Vec<semtree_core::Chunk> {
    let chunks = parse_and_extract(source, lang).expect("parse");
    assert!(!chunks.is_empty(), "{lang} produced no chunks");
    chunks
}

fn assert_has_kind(chunks: &[semtree_core::Chunk], kind: ChunkKind) {
    let kinds: Vec<&ChunkKind> = chunks.iter().map(|c| &c.kind).collect();
    assert!(kinds.contains(&&kind), "expected {kind:?} in {kinds:?}");
}

fn assert_has_name(chunks: &[semtree_core::Chunk], name: &str) {
    let names: Vec<&str> = chunks.iter().filter_map(|c| c.name.as_deref()).collect();
    assert!(names.contains(&name), "expected name {name:?} in {names:?}");
}

// ── Java ─────────────────────────────────────────────────────────────────────

#[test]
fn test_java_extracts_class_interface_method() {
    let source = r#"
interface Greeter {
    String greet();
}

public class Hello implements Greeter {
    public String greet() { return "hi"; }
}

enum Color { RED, GREEN }
"#;
    let chunks = extract(source, Language::Java);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Trait);
    assert_has_kind(&chunks, ChunkKind::Method);
    assert_has_kind(&chunks, ChunkKind::Enum);
    assert_has_name(&chunks, "Hello");
    assert_has_name(&chunks, "greet");
}

// ── C ────────────────────────────────────────────────────────────────────────

#[test]
fn test_c_extracts_function_struct_enum() {
    let source = r#"
struct Point { int x; int y; };
enum Dir { NORTH, SOUTH };

int add(int a, int b) {
    return a + b;
}
"#;
    let chunks = extract(source, Language::C);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_kind(&chunks, ChunkKind::Struct);
    assert_has_kind(&chunks, ChunkKind::Enum);
    // The function name is buried in the declarator chain, not a `name` field.
    assert_has_name(&chunks, "add");
    assert_has_name(&chunks, "Point");
}

// ── C++ ──────────────────────────────────────────────────────────────────────

#[test]
fn test_cpp_extracts_class_namespace_function() {
    let source = r#"
namespace geo {
    class Shape {
    public:
        double area();
    };
}

int* clamp(int* p) { return p; }
"#;
    let chunks = extract(source, Language::Cpp);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Module);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "Shape");
    assert_has_name(&chunks, "geo");
    // Pointer return type must not steal the name slot from the declarator.
    assert_has_name(&chunks, "clamp");
}

// ── C# ───────────────────────────────────────────────────────────────────────

#[test]
fn test_csharp_extracts_class_interface_method() {
    let source = r#"
namespace App {
    interface IGreeter { string Greet(); }

    public class Hello : IGreeter {
        public string Greet() => "hi";
    }

    enum Color { Red, Green }
}
"#;
    let chunks = extract(source, Language::CSharp);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Trait);
    assert_has_kind(&chunks, ChunkKind::Method);
    assert_has_kind(&chunks, ChunkKind::Module);
    assert_has_name(&chunks, "Hello");
    assert_has_name(&chunks, "Greet");
}

// ── Ruby ─────────────────────────────────────────────────────────────────────

#[test]
fn test_ruby_extracts_module_class_method() {
    let source = r#"
module Greetable
  def greet
    "hi"
  end
end

class Person
  include Greetable
  def name
    @name
  end
end
"#;
    let chunks = extract(source, Language::Ruby);
    assert_has_kind(&chunks, ChunkKind::Module);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Method);
    assert_has_name(&chunks, "Person");
    assert_has_name(&chunks, "greet");
}

// ── PHP ──────────────────────────────────────────────────────────────────────

#[test]
fn test_php_extracts_class_interface_function() {
    let source = r#"<?php
interface Greeter {
    public function greet(): string;
}

class Hello implements Greeter {
    public function greet(): string { return "hi"; }
}

function standalone(): int { return 42; }
"#;
    let chunks = extract(source, Language::Php);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Trait);
    assert_has_kind(&chunks, ChunkKind::Method);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "Hello");
    assert_has_name(&chunks, "standalone");
}

// ── Kotlin ───────────────────────────────────────────────────────────────────

#[test]
fn test_kotlin_extracts_class_function_object() {
    let source = r#"
class Greeter {
    fun greet(): String { return "hi" }
}

object Config
"#;
    let chunks = extract(source, Language::Kotlin);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_kind(&chunks, ChunkKind::Module);
    assert_has_name(&chunks, "Greeter");
    assert_has_name(&chunks, "greet");
}

// ── Scala ────────────────────────────────────────────────────────────────────

#[test]
fn test_scala_extracts_class_trait_object() {
    let source = r#"
class Greeter {
  def greet(): String = "hi"
}

trait Named

object Config
"#;
    let chunks = extract(source, Language::Scala);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Trait);
    assert_has_kind(&chunks, ChunkKind::Module);
    assert_has_name(&chunks, "Greeter");
    assert_has_name(&chunks, "greet");
}

// ── Swift ────────────────────────────────────────────────────────────────────

#[test]
fn test_swift_extracts_class_protocol_function() {
    let source = r#"
class Greeter {
    func greet() -> String { return "hi" }
}

protocol Named {}
"#;
    let chunks = extract(source, Language::Swift);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Trait);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "Greeter");
    assert_has_name(&chunks, "greet");
}

// ── Solidity ─────────────────────────────────────────────────────────────────

#[test]
fn test_solidity_extracts_contract_interface_function() {
    let source = r#"
interface IERC20 {}

contract Token {
    function transfer(address to) public {}
}
"#;
    let chunks = extract(source, Language::Solidity);
    assert_has_kind(&chunks, ChunkKind::Class);
    assert_has_kind(&chunks, ChunkKind::Trait);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "Token");
    assert_has_name(&chunks, "transfer");
}

// ── Lua ──────────────────────────────────────────────────────────────────────

#[test]
fn test_lua_extracts_functions() {
    let source = r#"
function greet(name)
  return "hi"
end

local function helper()
end
"#;
    let chunks = extract(source, Language::Lua);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "greet");
    assert_has_name(&chunks, "helper");
}

// ── OCaml ────────────────────────────────────────────────────────────────────

#[test]
fn test_ocaml_extracts_value_type_module() {
    let source = r#"
let greet name = "hi"

type color = Red | Green

module Config = struct end
"#;
    let chunks = extract(source, Language::OCaml);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_kind(&chunks, ChunkKind::Struct);
    assert_has_kind(&chunks, ChunkKind::Module);
    assert_has_name(&chunks, "greet");
    assert_has_name(&chunks, "color");
    assert_has_name(&chunks, "Config");
}

// ── Zig ──────────────────────────────────────────────────────────────────────

#[test]
fn test_zig_extracts_functions() {
    let source = r#"
fn greet() void {}

pub fn add(a: i32, b: i32) i32 {
    return a + b;
}
"#;
    let chunks = extract(source, Language::Zig);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "greet");
    assert_has_name(&chunks, "add");
}

// ── Emacs Lisp ───────────────────────────────────────────────────────────────

#[test]
fn test_elisp_extracts_defun_defmacro() {
    let source = r#"
(defun greet (name) "hi")

(defmacro my-macro () nil)
"#;
    let chunks = extract(source, Language::Elisp);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "greet");
    assert_has_name(&chunks, "my-macro");
}

// ── TSX ──────────────────────────────────────────────────────────────────────

#[test]
fn test_tsx_extracts_interface_and_function() {
    let source = r#"
interface Props { name: string }

function App(props: Props) {
    return <div>{props.name}</div>;
}
"#;
    let chunks = extract(source, Language::Tsx);
    assert_has_kind(&chunks, ChunkKind::Trait);
    assert_has_kind(&chunks, ChunkKind::Function);
    assert_has_name(&chunks, "Props");
    assert_has_name(&chunks, "App");
}
