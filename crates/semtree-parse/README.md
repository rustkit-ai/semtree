# semtree-parse

Tree-sitter parsing and chunk extraction for [semtree](https://github.com/rustkit-ai/semtree).

Turns source files into structured `Chunk`s (functions, structs, methods, classes) ready to embed. Non-code files fall back to overlapping fixed-size text windows.

## Usage

```toml
[dependencies]
semtree-parse = "0.4"
semtree-core  = "0.4"
```

```rust
use semtree_core::Language;
use semtree_parse::{parse_and_extract, parse_and_extract_file};

// From a string with a known language:
let chunks = parse_and_extract(source, Language::Rust)?;

// Or from a path, with the language inferred from the extension:
let chunks = parse_and_extract_file("src/lib.rs".as_ref())?;
```

## API

| Item | Purpose |
|---|---|
| `parse_and_extract(source, lang)` | Parse a string and extract chunks |
| `parse_and_extract_file(path)` | Parse a file, inferring the language from its extension |
| `extract_file(path)` | Extract chunks, using the text-window fallback for non-code files |
| `SemtreeParser` / `ParsedTree` | Lower-level tree-sitter parsing |
| `chunk_text` / `is_text_file` | Fixed-window chunking for plain-text and config files |

## Supported languages

Rust, Python, JavaScript, TypeScript, TSX, Go, Java, C, C++, C#, Ruby, PHP, Kotlin, Scala, Swift, OCaml, Solidity, Lua, Zig, and Emacs Lisp extract structured chunks (functions, types, methods). Other files (`.md`, `.json`, `.toml`, `.yaml`, ...) are chunked into overlapping line windows. See the [workspace README](https://github.com/rustkit-ai/semtree) for the full support matrix.

Each language is a tree-sitter query in [`src/lang/queries/`](src/lang/queries); adding one is a grammar dependency plus a `.scm` file, with no per-language Rust code.

## License

MIT

Part of [rustkit-ai](https://github.com/rustkit-ai) - open source Rust tools for the AI development era.
