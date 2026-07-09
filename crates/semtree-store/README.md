# semtree-store

Vector store trait and backends for [semtree](https://github.com/rustkit-ai/semtree).

Defines the `VectorStore` trait (insert, search, delete, save/load) and ships two backends. The default (`usearch`) is an in-process HNSW index persisted to disk.

## Usage

```toml
[dependencies]
semtree-store = "0.2"
semtree-embed = "0.2"
```

```rust
use semtree_store::{VectorStore, usearch::UsearchStore};

let store = UsearchStore::new(384)?;               // dimension must match the embedder
store.insert("chunk-1", &embedding).await?;
let hits = store.search(&query, 5).await?;         // Vec<Hit> { id, score }
```

## Backends

| Backend | Notes |
|---|---|
| `usearch` | In-process HNSW, saved to disk. No external service. |
| `qdrant` | Remote Qdrant server. Set `QDRANT_URL` or `store.url`. |

## Custom backend

Implement `VectorStore` to target any vector database:

```rust
use async_trait::async_trait;
use semtree_store::{VectorStore, Hit, StoreError};
use semtree_embed::Embedding;

struct MyStore;

#[async_trait]
impl VectorStore for MyStore {
    async fn insert(&self, id: &str, emb: &Embedding) -> Result<(), StoreError> { todo!() }
    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<Hit>, StoreError> { todo!() }
    async fn delete(&self, id: &str) -> Result<(), StoreError> { todo!() }
    fn save(&self, path: &std::path::Path) -> Result<(), StoreError> { Ok(()) }
    fn load(&mut self, path: &std::path::Path) -> Result<(), StoreError> { Ok(()) }
    fn len(&self) -> usize { 0 }
}
```

## License

MIT
