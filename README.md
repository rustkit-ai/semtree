<h1 align="center">semtree</h1>

<p align="center">
  Semantic code intelligence for any codebase — parse, embed, search, inject.
</p>

<p align="center">
  <a href="https://github.com/rustkit-ai/semtree/actions/workflows/ci.yml"><img src="https://github.com/rustkit-ai/semtree/actions/workflows/ci.yml/badge.svg" alt="CI"/></a>
  <a href="https://crates.io/crates/semtree-rag"><img src="https://img.shields.io/crates/v/semtree-rag.svg" alt="crates.io"/></a>
  <a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"/></a>
</p>

---

**Code search tools force a tradeoff.** Grep finds exact strings. Language servers require a running daemon and IDE integration. Cloud AI search sends your code to a third party. None of those work well inside a Rust library or a local tool.

`semtree` uses [tree-sitter](https://tree-sitter.github.io/tree-sitter/) to parse your codebase into structured chunks (functions, structs, methods), embeds them locally via [fastembed](https://github.com/Anush008/fastembed-rs), and stores them in an HNSW vector index — all on-device, no API key required, no daemon.

```
$ semtree index ./my-project
⠿ [========================================] 87/87 files  (3s)
Done (incremental). Indexed 312 chunks → .semtree/

$ semtree search "how is authentication handled"
1. [Function] validate_token  (score: 0.921)
   src/auth/jwt.rs:14
   pub fn validate_token(token: &str, secret: &[u8]) -> Result<Claims> {

2. [Function] middleware  (score: 0.887)
   src/auth/middleware.rs:28
   pub async fn middleware(req: Request, next: Next) -> Response {

$ semtree stats
=== Index: .semtree ===

  Chunks : 312
  Files  : 87
  Size   : 1.8 MB

By language:
  rust:          200  (64%)
  typescript:     80  (26%)
  go:             32  (10%)
```

**No daemon. No Python.** Embeddings run on CPU via ONNX, cached after first use. Supports OpenAI and Ollama as drop-in embedding backends when you need higher quality.

---

## Install

**CLI:**
```bash
cargo install semtree-cli
```

**Library:**
```toml
[dependencies]
semtree-rag   = "0.1"
semtree-embed = "0.1"
semtree-store = "0.1"
```

---

## CLI

```bash
semtree init                                   # create .semtree.toml
semtree index ./my-project                     # index (incremental by default)
semtree index ./my-project --full              # force full re-scan
semtree search "error handling strategy" -k 5  # semantic search
semtree context "authentication flow"          # RAG context block for LLMs
semtree stats                                  # chunks, languages, index size
semtree analyze                                # complexity metrics, largest functions
```

All commands accept `--config <path>` to point to a custom `.semtree.toml`.

### Incremental indexing

Re-running `semtree index` only processes files whose content has changed. A manifest (`manifest.json`) is stored alongside the index to track per-file hashes. Pass `--full` to force a complete re-scan.

---

## Configuration

`semtree init` creates a `.semtree.toml` in the current directory:

```toml
[embed]
backend = "fastembed"   # fastembed | openai | ollama
# model   = "text-embedding-3-small"
# url     = "http://localhost:11434"   # ollama only
# api_key = "sk-..."                   # or set OPENAI_API_KEY

[store]
backend    = "usearch"   # usearch | qdrant
# url        = "http://localhost:6333"
# collection = "semtree"

index_dir = ".semtree"
```

### Embedding backends

| Backend | Default model | Notes |
|---|---|---|
| `fastembed` (default) | `AllMiniLML6V2` (384-dim) | On-device, no key needed |
| `openai` | `text-embedding-3-small` | Set `OPENAI_API_KEY` or `embed.api_key` |
| `ollama` | `nomic-embed-text` | Requires local Ollama server |

### Vector store backends

| Backend | Notes |
|---|---|
| `usearch` (default) | In-process HNSW, saved to disk |
| `qdrant` | Remote Qdrant server — set `QDRANT_URL` or `store.url` |

---

## Library

```rust
use std::sync::Arc;
use semtree_embed::fastembed::FastEmbedder;
use semtree_store::usearch::UsearchStore;
use semtree_rag::{ChunkRegistry, FileManifest, Indexer, SearchEngine};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let embedder = Arc::new(FastEmbedder::new()?);
    let store    = Arc::new(UsearchStore::new(384)?);

    let indexer = Indexer::new(embedder.clone(), store.clone());
    let mut registry = ChunkRegistry::default();
    let mut manifest = FileManifest::default();

    let n = indexer
        .index_dir("./src".as_ref(), &mut registry, Some(&mut manifest), |done, total| {
            eprint!("\r{done}/{total}");
        })
        .await?;
    println!("\nIndexed {n} chunks");

    let engine = SearchEngine::new(embedder, store);
    let hits = engine.search("error handling", 5).await?;
    for hit in &hits {
        if let Some(chunk) = registry.get(&hit.id) {
            println!("{} — {}:{} (score: {:.3})",
                chunk.name.as_deref().unwrap_or("?"),
                chunk.path.display(),
                chunk.span.start_line + 1,
                hit.score);
        }
    }
    Ok(())
}
```

---

## Architecture

Each crate is independently published to [crates.io](https://crates.io) — use only what you need.

```
semtree-core     # shared types: Language, Span, Chunk, ChunkKind
semtree-parse    # tree-sitter parsing + chunk extraction
semtree-embed    # Embedder trait + fastembed / OpenAI / Ollama backends
semtree-store    # VectorStore trait + usearch / Qdrant backends
semtree-rag      # index, search, LLM context, incremental manifest
semtree-analyze  # complexity metrics, large-function detection
semtree-cli      # CLI binary (semtree)
```

---

## Supported languages

| Language   | Parse | Extract |
|---|---|---|
| Rust       | ✅ | ✅ functions, structs, enums, traits, impls, modules |
| Python     | ✅ | ✅ functions, classes, decorators |
| TypeScript | ✅ | ✅ functions, classes, interfaces, enums, type aliases, exports |
| JavaScript | ✅ | ✅ functions, classes, generators, exports |
| Go         | ✅ | ✅ functions, methods, structs, interfaces |

Plain text files (`.md`, `.json`, `.toml`, `.yaml`, …) are chunked into overlapping 40-line windows.

---

## Custom backends

**Custom embedder:**
```rust
use async_trait::async_trait;
use semtree_embed::{Embedder, Embedding, EmbedError};

struct MyEmbedder;

#[async_trait]
impl Embedder for MyEmbedder {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        todo!() // call your API or local model
    }
}
```

**Custom vector store:**
```rust
use semtree_store::{VectorStore, Hit, StoreError};
use semtree_embed::Embedding;

struct MyStore;

#[async_trait]
impl VectorStore for MyStore {
    async fn insert(&self, id: &str, emb: &Embedding) -> Result<(), StoreError> { todo!() }
    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<Hit>, StoreError> { todo!() }
    async fn delete(&self, id: &str) -> Result<(), StoreError> { todo!() }
    fn save(&self, _path: &std::path::Path) -> Result<(), StoreError> { Ok(()) }
    fn load(&mut self, _path: &std::path::Path) -> Result<(), StoreError> { Ok(()) }
    fn len(&self) -> usize { 0 }
}
```

---

## License

MIT — see [LICENSE](LICENSE)
