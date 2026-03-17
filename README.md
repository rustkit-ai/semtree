# semtree

Semantic code intelligence for any codebase — built in Rust.

Parse code into structured chunks using [tree-sitter](https://tree-sitter.github.io/tree-sitter/), embed them locally, and power semantic search, LLM context injection, and static analysis.

```
semtree index ./my-project
semtree search "how is authentication handled"
semtree context "explain the error handling strategy"
```

## Features

- **Semantic search** — find code by meaning, not just keywords
- **LLM context injection** — retrieve relevant chunks for RAG pipelines
- **Static analysis** — dead code detection, call graphs, complexity *(coming soon)*
- **Multi-language** — Rust, Python, JavaScript, TypeScript, Go *(more via tree-sitter)*
- **Local-first** — embeddings run on-device via [fastembed](https://github.com/Anush008/fastembed-rs), no API keys required
- **Swappable backends** — bring your own embedder or vector store

## Architecture

```
semtree-core     # shared types: Language, Span, Chunk
semtree-parse    # tree-sitter parsing + AST extraction
semtree-embed    # Embedder trait + fastembed backend
semtree-store    # VectorStore trait + usearch backend
semtree-rag      # index, search, and context pipeline
semtree-analyze  # static analysis (coming soon)
semtree-cli      # CLI
```

Each crate is published independently on [crates.io](https://crates.io).

## Usage

### As a library

```toml
[dependencies]
semtree-rag   = "0.1"
semtree-embed = "0.1"
semtree-store = "0.1"
```

```rust
use std::sync::Arc;
use semtree_embed::fastembed::FastEmbedder;
use semtree_store::usearch::UsearchStore;
use semtree_rag::{Indexer, SearchEngine};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let embedder = Arc::new(FastEmbedder::new()?);
    let store    = Arc::new(UsearchStore::new(384)?);

    // Index a directory
    let indexer = Indexer::new(embedder.clone(), store.clone());
    let n = indexer.index_dir("./src".as_ref()).await?;
    println!("Indexed {n} chunks");

    // Search
    let engine = SearchEngine::new(embedder, store);
    let hits = engine.search("error handling", 5).await?;
    for hit in hits {
        println!("{} (score: {:.3})", hit.id, hit.score);
    }

    Ok(())
}
```

### CLI

```bash
cargo install semtree-cli

semtree index ./my-project
semtree search "authentication logic"
semtree context "how are errors propagated"
```

## Supported languages

| Language   | Parsing | Extraction |
|------------|---------|------------|
| Rust       | ✅      | ✅         |
| Python     | ✅      | 🚧         |
| TypeScript | ✅      | 🚧         |
| JavaScript | ✅      | 🚧         |
| Go         | ✅      | 🚧         |

## Extending

### Custom embedder

```rust
use async_trait::async_trait;
use semtree_embed::{Embedder, Embedding, EmbedError};

struct MyEmbedder;

#[async_trait]
impl Embedder for MyEmbedder {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        // call your API or model
        todo!()
    }
}
```

### Custom vector store

```rust
use async_trait::async_trait;
use semtree_store::{VectorStore, Hit, StoreError};
use semtree_embed::Embedding;

struct MyStore;

#[async_trait]
impl VectorStore for MyStore {
    async fn insert(&self, id: &str, embedding: &Embedding) -> Result<(), StoreError> { todo!() }
    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<Hit>, StoreError> { todo!() }
    async fn delete(&self, id: &str) -> Result<(), StoreError> { todo!() }
}
```

## License

MIT — see [LICENSE](LICENSE)
