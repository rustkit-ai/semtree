use async_trait::async_trait;
use semtree_embed::Embedding;

use crate::StoreError;

#[derive(Debug, Clone)]
pub struct Hit {
    pub id: String,
    pub score: f32,
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn insert(&self, id: &str, embedding: &Embedding) -> Result<(), StoreError>;
    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<Hit>, StoreError>;
    async fn delete(&self, id: &str) -> Result<(), StoreError>;
}
