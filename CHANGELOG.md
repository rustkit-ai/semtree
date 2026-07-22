# Changelog

## [0.4.0]

### Breaking changes

- `Embedder` now requires `dimension()` and `model_id()`, and gains `max_batch_size()` and a derived `fingerprint()`. Custom embedders must add the two required methods.
- `VectorStore` now requires `metric()`; the crate adds a `Metric` enum.
- `FileManifest::new` takes an embedder and a store fingerprint, and the manifest is versioned. Indexes built with 0.3 are treated as incompatible and rebuilt on the next run.
- Chunk IDs are now `blake3(path + span)` instead of a content hash, so existing indexes rebuild.
- `semtree-parse` drops the public `Extractor` trait; extraction is query-driven.

### New features

**Languages** (`semtree-parse`)
- 20 languages, up from 5: adds Java, C, C++, C#, Ruby, PHP, TSX, Kotlin, Scala, Swift, OCaml, Solidity, Lua, Zig and Emacs Lisp.
- Extraction is driven by tree-sitter query files (one `.scm` per language), so adding a language needs no Rust. Queries are compiled once per language and validated by a build-time test.

**Umbrella crate** (`semtree`)
- New crate re-exporting the default stack. `semtree::default_backends()` builds a fastembed embedder and a usearch store sized to it, and `semtree::prelude::*` brings in the indexing and search types.

**Embedder** (`semtree-embed`)
- `dimension()`, `model_id()` and a derived `fingerprint()` let an index reject vectors from a different model.
- `max_batch_size()` bounds `embed()` batch sizes so a large file no longer risks memory blow-up or an oversized remote request.

**Store** (`semtree-store`)
- `Metric` enum and `VectorStore::metric()`; usearch and qdrant are metric-configurable and refuse to reopen an index under a different metric.

**Incremental indexing** (`semtree-rag`)
- The manifest is versioned (schema and chunker version, embedder and store fingerprints) and rebuilds instead of mixing incompatible vectors.

### Fixes

- Stable, location-scoped chunk IDs fix cross-file collisions and clean deletion of a renamed file's chunks.
- The CLI sizes the vector store to the embedder's dimension (OpenAI/Ollama indexes were mis-sized at 384).
- `index_dir` defaults to `.semtree` when there is no config file (it was writing into the current directory).
- No phantom "incompatible index" warning on a first index.

## [0.3.0]

### New features

**Hybrid search** (`semtree-rag`)
- `HybridSearcher` fuses vector similarity (`SearchEngine`) with BM25 keyword matching (`LexicalIndex`) via reciprocal rank fusion - catches concepts a grep misses while keeping exact-identifier precision
- `SearchMode` selects `Hybrid` (default), `Semantic`, or `Lexical`
- `LexicalIndex::from_chunks` builds an in-memory BM25 index from a chunk registry

**CLI** (`semtree-cli`)
- `semtree search --mode hybrid|semantic|lexical` (defaults to hybrid)
- `semtree search --lang / --kind / --path` filters, applied after the vector lookup

### Documentation

- Crate-level docs (`//!`) on all six library crates - each now has a pitch, a components table, and an example rendered on docs.rs
- New runnable example `semtree-rag/examples/build_your_own.rs` - assemble the pipeline from its building blocks
- New example crate `examples/mcp-server` - expose semtree as a Model Context Protocol server (tool `search_code`) for AI agents, using the `rmcp` SDK over stdio
- README: composable-library positioning, hybrid-search callout, MCP-server section, docs.rs/downloads badges

## [0.2.1]

### Documentation

- Added a per-crate `README.md` for all seven crates, each rendered on its crates.io page via the new `readme` manifest field
- No code changes; this release also backfills `semtree-cli` (previously 0.1.0) and publishes `semtree-analyze` for the first time

## [0.2.0]

### Breaking changes

- `Indexer::index_file` now takes an additional `Option<&mut FileManifest>` argument for incremental indexing
- `Indexer::index_dir` now takes `Option<&mut FileManifest>` and a `on_progress: impl Fn(usize, usize)` callback
- `ComplexityReport` fields changed: added `path`, `start_line`, `cyclomatic`; `analyze_chunks` now sorts by cyclomatic complexity instead of line count

### New features

**Incremental indexing** (`semtree-rag`)
- `FileManifest` tracks per-file content hashes; unchanged files are skipped on re-index
- Stale chunks are automatically removed from the store and registry when a file changes or is deleted
- `collect_indexable_files(dir)` is now public

**New embedding backends** (`semtree-embed`)
- `OpenAIEmbedder` — calls `POST /v1/embeddings`, model and API key configurable
- `OllamaEmbedder` — calls `POST /api/embed`, model and base URL configurable
- New `EmbedConfig` field: `api_key` (falls back to `OPENAI_API_KEY` env var)
- New crate features: `openai-backend`, `ollama-backend`

**New vector store backend** (`semtree-store`)
- `QdrantStore` — HTTP REST client for Qdrant, lazy collection creation, deterministic point IDs
- New crate feature: `qdrant-backend`

**CLI** (`semtree-cli`)
- `semtree stats` — chunk count, breakdown by language and kind, index size on disk
- `semtree analyze` — top N functions by cyclomatic complexity and by line count
- `semtree index --full` — force a complete re-scan, ignoring the incremental manifest
- Global `--config <path>` flag to point to a custom `.semtree.toml`
- Progress bar with `indicatif` during indexing

**Language extractors** (`semtree-parse`)
- Go: fixed struct vs interface detection in `type_declaration` nodes
- TypeScript: added `enum_declaration`, `abstract_class_declaration`, `export_statement` (wrapping function/class/interface/enum/type)
- JavaScript: added `generator_function_declaration`, `export_statement`
- Added `.jsx` → JavaScript language mapping

**Analysis** (`semtree-analyze`)
- `ComplexityReport` now includes `path`, `start_line`, and approximate `cyclomatic` complexity
- `cyclomatic_complexity(content, language)` exposed as a public function

## [0.1.0]

Initial release.

- Tree-sitter parsing for Rust, Python, TypeScript, JavaScript, Go
- Local embeddings via fastembed (AllMiniLML6V2, 384-dim)
- In-process HNSW vector index via usearch
- `semtree index / search / context / init` CLI commands
- `semtree-rag`: `Indexer`, `SearchEngine`, `ContextBuilder`, `ChunkRegistry`
