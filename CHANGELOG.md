# Changelog

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
