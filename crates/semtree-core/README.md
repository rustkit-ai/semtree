# semtree-core

Shared types for the [semtree](https://github.com/rustkit-ai/semtree) workspace.

This crate has no heavy dependencies. It defines the types every other `semtree-*` crate uses: languages, source spans, extracted chunks, and configuration.

## Types

| Type | Purpose |
|---|---|
| `Language` | Supported source languages (Rust, Python, TypeScript, JavaScript, Go, ...) |
| `Span` | A byte/line range within a source file |
| `Chunk` / `ChunkKind` | An extracted unit (function, struct, method, text window) with its span and metadata |
| `SemtreeConfig` | Parsed `.semtree.toml`: embedding and store backends, index dir |
| `EmbedConfig` / `EmbedBackend` | Embedding backend selection (fastembed, openai, ollama) |
| `StoreConfig` / `StoreBackend` | Vector store selection (usearch, qdrant) |

## Usage

```toml
[dependencies]
semtree-core = "0.2"
```

```rust
use semtree_core::{Chunk, ChunkKind, Language, Span};
```

You rarely depend on this crate directly, since it comes in transitively via `semtree-parse`, `semtree-rag`, and the others. Depend on it explicitly when you implement your own parser or store and need the shared types.

## License

MIT. See the [workspace README](https://github.com/rustkit-ai/semtree) for the full picture.

Part of [rustkit-ai](https://github.com/rustkit-ai) - open source Rust tools for the AI development era.
