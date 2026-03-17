use async_trait::async_trait;

use crate::{EmbedError, Embedding};

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError>;

    async fn embed_one(&self, text: &str) -> Result<Embedding, EmbedError> {
        self.embed(&[text])
            .await
            .map(|mut v| v.remove(0))
    }
}
