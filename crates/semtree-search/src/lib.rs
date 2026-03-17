use std::sync::Arc;

use semtree_embed::Embedder;
use semtree_store::{Hit, VectorStore};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SearchError {
    #[error("embed error: {0}")]
    Embed(#[from] semtree_embed::EmbedError),
    #[error("store error: {0}")]
    Store(#[from] semtree_store::StoreError),
}

pub struct SearchEngine {
    embedder: Arc<dyn Embedder>,
    store: Arc<dyn VectorStore>,
}

impl SearchEngine {
    pub fn new(embedder: Arc<dyn Embedder>, store: Arc<dyn VectorStore>) -> Self {
        Self { embedder, store }
    }

    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<Hit>, SearchError> {
        let embedding = self.embedder.embed_one(query).await?;
        let hits = self.store.search(&embedding, top_k).await?;
        Ok(hits)
    }
}
