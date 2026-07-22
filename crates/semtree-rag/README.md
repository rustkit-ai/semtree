# semtree-rag

The RAG pipeline for [semtree](https://github.com/rustkit-ai/semtree): index, search, and build LLM context blocks from a codebase.

This is the crate most library users depend on. It ties `semtree-parse`, `semtree-embed`, and `semtree-store` together into an indexer, a search engine, and a context builder, with incremental re-indexing and hybrid (semantic + keyword) retrieval.

## Usage

```toml
[dependencies]
semtree-rag   = "0.4"
semtree-embed = "0.4"
semtree-store = "0.4"
```

```rust
use std::sync::Arc;
use semtree_embed::Embedder;
use semtree_embed::fastembed::FastEmbedder;
use semtree_store::VectorStore;
use semtree_store::usearch::UsearchStore;
use semtree_rag::{ChunkRegistry, FileManifest, Indexer, SearchEngine};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let embedder = Arc::new(FastEmbedder::new()?);
    let store    = Arc::new(UsearchStore::new(embedder.dimension())?);

    let indexer = Indexer::new(embedder.clone(), store.clone());
    let mut registry = ChunkRegistry::default();
    // Pin the manifest to this embedder and store, so a later run with a
    // different model or metric rebuilds instead of mixing incompatible vectors.
    let mut manifest = FileManifest::new(embedder.fingerprint(), store.metric().to_string());

    indexer
        .index_dir("./src".as_ref(), &mut registry, Some(&mut manifest), |done, total| {
            eprint!("\r{done}/{total}");
        })
        .await?;

    let engine = SearchEngine::new(embedder, store);
    let hits = engine.search("error handling", 5).await?;
    for hit in &hits {
        if let Some(chunk) = registry.get(&hit.id) {
            println!("{:?} (score {:.3})", chunk.name, hit.score);
        }
    }
    Ok(())
}
```

## API

| Item | Purpose |
|---|---|
| `Indexer` | Parse, embed, and store a directory of source files |
| `collect_indexable_files` | Enumerate the files an index run would process |
| `SearchEngine` | Vector similarity search over the store |
| `HybridSearcher` / `SearchMode` | Fuse semantic and BM25 lexical rankings via Reciprocal Rank Fusion |
| `LexicalIndex` | Standalone BM25 keyword index |
| `ContextBuilder` / `ContextWindow` / `ContextSnippet` | Assemble a token-bounded context block for an LLM prompt |
| `ChunkRegistry` | Map chunk IDs back to their source chunks |
| `FileManifest` | Per-file content hashes used for incremental indexing |

## Search modes

`SearchMode` selects how a query matches the index:

- `Hybrid` (default): fuses vector similarity and BM25 keyword matching. Catches concepts a grep misses while keeping the exact-identifier precision a pure vector search loses.
- `Semantic`: vector similarity only.
- `Lexical`: BM25 keyword matching only.

## Incremental indexing

Passing a `FileManifest` to `index_dir` makes re-runs process only files whose content changed. Persist the manifest alongside the index to keep incremental behavior across runs.

## License

MIT

Part of [rustkit-ai](https://github.com/rustkit-ai) - open source Rust tools for the AI development era.
