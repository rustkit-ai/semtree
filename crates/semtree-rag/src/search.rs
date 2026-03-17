use std::sync::Arc;

use semtree_embed::Embedder;
use semtree_store::{Hit, VectorStore};

use crate::RagError;

pub struct SearchEngine {
    embedder: Arc<dyn Embedder>,
    store: Arc<dyn VectorStore>,
}

impl SearchEngine {
    pub fn new(embedder: Arc<dyn Embedder>, store: Arc<dyn VectorStore>) -> Self {
        Self { embedder, store }
    }

    pub async fn search(&self, query: &str, top_k: usize) -> Result<Vec<Hit>, RagError> {
        let embedding = self.embedder.embed_one(query).await?;
        let hits = self.store.search(&embedding, top_k).await?;
        Ok(hits)
    }
}
