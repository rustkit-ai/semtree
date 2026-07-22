# semtree-analyze

Static analysis for [semtree](https://github.com/rustkit-ai/semtree): complexity metrics and large-function detection over extracted chunks.

Operates on the `Chunk`s produced by `semtree-parse`, so it reuses the same tree-sitter extraction as the rest of the pipeline. Backs the `semtree analyze` command.

## Usage

```toml
[dependencies]
semtree-analyze = "0.4"
semtree-parse   = "0.4"
```

```rust
use semtree_analyze::{analyze_chunks, cyclomatic_complexity, find_large_functions};
use semtree_parse::parse_and_extract_file;

let chunks = parse_and_extract_file("src/lib.rs".as_ref())?;

let reports = analyze_chunks(&chunks);           // Vec<ComplexityReport>
let big     = find_large_functions(&chunks, 80); // functions over 80 lines
```

## API

| Item | Purpose |
|---|---|
| `analyze_chunks(chunks)` | Produce a `ComplexityReport` per chunk |
| `find_large_functions(chunks, threshold)` | Chunks whose line count exceeds `threshold` |
| `cyclomatic_complexity(content, language)` | Approximate cyclomatic complexity of a snippet |
| `ComplexityReport` | Per-chunk metrics (name, span, line count, complexity) |

## License

MIT

Part of [rustkit-ai](https://github.com/rustkit-ai) - open source Rust tools for the AI development era.
