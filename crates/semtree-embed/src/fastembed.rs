use async_trait::async_trait;
use fastembed::{EmbeddingModel, InitOptions, TextEmbedding};

use crate::{EmbedError, Embedder, Embedding};

pub struct FastEmbedder {
    model: TextEmbedding,
    model_id: String,
    dimension: usize,
}

impl FastEmbedder {
    pub fn new() -> Result<Self, EmbedError> {
        Self::with_model(EmbeddingModel::AllMiniLML6V2)
    }

    pub fn with_model(model: EmbeddingModel) -> Result<Self, EmbedError> {
        // `{model:?}` yields the variant name (e.g. `AllMiniLML6V2`), which is
        // stable across releases and unique per model.
        let model_id = format!("fastembed:{model:?}");
        let te = TextEmbedding::try_new(InitOptions::new(model))
            .map_err(|e| EmbedError::ModelLoad(e.to_string()))?;

        // Probe the real dimension instead of hard-coding a per-model table.
        let dimension = te
            .embed(vec!["dimension probe".to_string()], None)
            .map_err(|e| EmbedError::EmbedFailed(e.to_string()))?
            .first()
            .map(|v| v.len())
            .ok_or_else(|| EmbedError::EmbedFailed("empty probe embedding".to_string()))?;

        Ok(Self {
            model: te,
            model_id,
            dimension,
        })
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

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}
