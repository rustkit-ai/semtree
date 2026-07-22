# semtree-embed

Embedding trait and backends for [semtree](https://github.com/rustkit-ai/semtree).

Defines the `Embedder` trait and ships three implementations. The default (`fastembed`) runs on-device via ONNX, with no API key or daemon required.

## Usage

```toml
[dependencies]
semtree-embed = "0.2"
```

```rust
use semtree_embed::{Embedder, fastembed::FastEmbedder};

let embedder = FastEmbedder::new()?;              // AllMiniLML6V2, 384-dim
let vectors = embedder.embed(&["hello", "world"]).await?;
```

## Backends

| Backend | Default model | Notes |
|---|---|---|
| `fastembed` | `AllMiniLML6V2` (384-dim) | On-device. No key needed. Model cached after first use. |
| `openai` | `text-embedding-3-small` | Set `OPENAI_API_KEY` or pass a key. |
| `ollama` | `nomic-embed-text` | Requires a local Ollama server. |

## Custom backend

Implement `Embedder` to plug in any model or API:

```rust
use async_trait::async_trait;
use semtree_embed::{Embedder, Embedding, EmbedError};

struct MyEmbedder;

#[async_trait]
impl Embedder for MyEmbedder {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        todo!()
    }
    fn dimension(&self) -> usize { 384 }
    fn model_id(&self) -> &str { "my-embedder" }
}
```

`Embedding` is a plain `Vec<f32>`.

## License

MIT

Part of [rustkit-ai](https://github.com/rustkit-ai) - open source Rust tools for the AI development era.
