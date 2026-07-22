use async_trait::async_trait;
use semtree_embed::Embedding;
use serde::{Deserialize, Serialize};

use crate::StoreError;

#[derive(Debug, Clone)]
pub struct Hit {
    pub id: String,
    pub score: f32,
}

/// How a store measures nearness between vectors.
///
/// An index built under one metric ranks differently under another, so this is
/// part of what an index records to reject an incompatible reopen - alongside
/// the embedder fingerprint. It is deliberately a small, closed set: the common
/// trait exposes the metric because every vector store has one, and stops there
/// rather than surfacing backend-specific knobs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Metric {
    Cosine,
    Euclidean,
    DotProduct,
}

impl std::fmt::Display for Metric {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Metric::Cosine => "cosine",
            Metric::Euclidean => "euclidean",
            Metric::DotProduct => "dot_product",
        };
        f.write_str(s)
    }
}

#[async_trait]
pub trait VectorStore: Send + Sync {
    async fn insert(&self, id: &str, embedding: &Embedding) -> Result<(), StoreError>;
    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<Hit>, StoreError>;
    async fn delete(&self, id: &str) -> Result<(), StoreError>;
    fn save(&self, path: &std::path::Path) -> Result<(), StoreError>;
    fn load(&mut self, path: &std::path::Path) -> Result<(), StoreError>;
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Distance metric this store ranks by. Recorded by an index so reopening it
    /// under a different metric triggers a rebuild instead of silent mis-ranking.
    fn metric(&self) -> Metric;
}
