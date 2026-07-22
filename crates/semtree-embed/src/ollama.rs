use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{EmbedError, Embedder, Embedding};

pub struct OllamaEmbedder {
    client: Client,
    base_url: String,
    model: String,
    model_id: String,
    dimension: usize,
}

impl OllamaEmbedder {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
        let model = model.unwrap_or_else(|| "nomic-embed-text".to_string());
        Self {
            client: Client::new(),
            base_url: base_url
                .unwrap_or_else(|| "http://localhost:11434".to_string())
                .trim_end_matches('/')
                .to_string(),
            model_id: format!("ollama:{model}"),
            dimension: default_dimension(&model),
            model,
        }
    }
}

/// Output width of the common Ollama embedding models. Best-effort: `model_id`
/// is the authoritative half of the fingerprint, so an unknown model still gets
/// a distinct identity even when this falls back.
fn default_dimension(model: &str) -> usize {
    match model {
        m if m.starts_with("nomic-embed-text") => 768,
        m if m.starts_with("mxbai-embed-large") => 1024,
        m if m.starts_with("all-minilm") => 384,
        _ => 768,
    }
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a [&'a str],
}

#[derive(Deserialize)]
struct EmbedResponse {
    embeddings: Vec<Vec<f32>>,
}

#[async_trait]
impl Embedder for OllamaEmbedder {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        let url = format!("{}/api/embed", self.base_url);
        let req = EmbedRequest {
            model: &self.model,
            input: texts,
        };

        let resp = self
            .client
            .post(&url)
            .json(&req)
            .send()
            .await
            .map_err(|e| EmbedError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(EmbedError::Http(format!("{status}: {body}")));
        }

        let body: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| EmbedError::Http(e.to_string()))?;

        Ok(body.embeddings)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}
