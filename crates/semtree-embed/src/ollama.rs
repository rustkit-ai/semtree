use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{EmbedError, Embedder, Embedding};

pub struct OllamaEmbedder {
    client: Client,
    base_url: String,
    model: String,
}

impl OllamaEmbedder {
    pub fn new(base_url: Option<String>, model: Option<String>) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url
                .unwrap_or_else(|| "http://localhost:11434".to_string())
                .trim_end_matches('/')
                .to_string(),
            model: model.unwrap_or_else(|| "nomic-embed-text".to_string()),
        }
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
}
