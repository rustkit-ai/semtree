# semtree

Local semantic code search for [semtree](https://github.com/rustkit-ai/semtree), batteries included.

semtree is a toolbox of small crates (`semtree-parse`, `semtree-embed`, `semtree-store`, `semtree-rag`, ...). This crate is the assembled default: it pulls in the on-device stack (tree-sitter chunking, fastembed embeddings, usearch HNSW storage, hybrid search) so the common case is a few lines instead of wiring four crates by hand.

Reach for the individual crates when you want to swap a backend or trim the dependency tree. Reach for this one when you just want search to work.

> Looking for the `semtree` command-line tool? That is the [`semtree-cli`](https://crates.io/crates/semtree-cli) crate (`cargo install semtree-cli`). This crate is the library.

## Usage

```toml
[dependencies]
semtree = "0.4"
tokio   = { version = "1", features = ["full"] }
```

```rust
use semtree::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // The default on-device stack, sized and wired so it agrees with itself.
    let (embedder, store) = semtree::default_backends()?;

    let mut registry = ChunkRegistry::default();
    Indexer::new(embedder.clone(), store.clone())
        .index_dir("./src".as_ref(), &mut registry, None, |_, _| {})
        .await?;

    let engine = SearchEngine::new(embedder, store);
    let lexical = LexicalIndex::from_chunks(registry.iter());
    let hits = HybridSearcher::new(engine, lexical)
        .search("where do we skip unchanged files", 5, SearchMode::Hybrid)
        .await?;

    for hit in &hits {
        if let Some(chunk) = registry.get(&hit.id) {
            println!("{:?} (score {:.3})", chunk.name, hit.score);
        }
    }
    Ok(())
}
```

## What you get

- `semtree::default_backends()` builds fastembed + usearch, with the store sized to the embedder's own dimension so they never disagree on vector width.
- `semtree::prelude::*` brings in the indexing and search types (`Indexer`, `SearchEngine`, `HybridSearcher`, `ChunkRegistry`, `Metric`, ...).
- `semtree::rag`, `semtree::embed`, `semtree::store`, `semtree::parse`, `semtree::analyze`, `semtree::core` re-export each layer, so you never add the sub-crates to your own `Cargo.toml`.

## Features

| Feature | Default | Effect |
|---|---|---|
| `fastembed` | yes | Local ONNX embeddings; required by `default_backends` |
| `usearch` | yes | Local HNSW store; required by `default_backends` |
| `openai` | no | OpenAI embedding backend |
| `ollama` | no | Ollama embedding backend |
| `qdrant` | no | Qdrant vector store backend |

## License

MIT

Part of [rustkit-ai](https://github.com/rustkit-ai) - open source Rust tools for the AI development era.
