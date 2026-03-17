use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::{EmbedError, Embedder, Embedding};

pub struct FastEmbedder {
    model: TextEmbedding,
}

impl FastEmbedder {
    pub fn new() -> Result<Self, EmbedError> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }

    pub fn with_model(model: EmbeddingModel) -> Result<Self, EmbedError> {
        let te = TextEmbedding::try_new(InitOptions::new(model))
            .map_err(|e| EmbedError::ModelLoad(e.to_string()))?;
        Ok(Self { model: te })
    }
}

#[async_trait]
impl Embedder for FastEmbedder {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        let texts: Vec<String> = texts.iter().map(|s| s.to_string()).collect();
        self.model
            .embed(texts, None)
            .map_err(|e| EmbedError::EmbedFailed(e.to_string()))
    }
}
