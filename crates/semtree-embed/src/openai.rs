use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{EmbedError, Embedder, Embedding};

pub struct OpenAIEmbedder {
    client: Client,
    api_key: String,
    model: String,
    model_id: String,
    dimension: usize,
}

impl OpenAIEmbedder {
    pub fn new(api_key: impl Into<String>, model: Option<String>) -> Self {
        let model = model.unwrap_or_else(|| "text-embedding-3-small".to_string());
        Self {
            client: Client::new(),
            api_key: api_key.into(),
            model_id: format!("openai:{model}"),
            dimension: default_dimension(&model),
            model,
        }
    }

    pub fn from_env(model: Option<String>) -> Result<Self, EmbedError> {
        let key = std::env::var("OPENAI_API_KEY")
            .map_err(|_| EmbedError::MissingApiKey("OPENAI_API_KEY".to_string()))?;
        Ok(Self::new(key, model))
    }
}

/// Native output width of the known OpenAI embedding models. Best-effort: the
/// `model_id` is the authoritative half of the fingerprint, so an unknown model
/// still gets a distinct identity even if this falls back.
fn default_dimension(model: &str) -> usize {
    match model {
        "text-embedding-3-large" => 3072,
        "text-embedding-3-small" | "text-embedding-ada-002" => 1536,
        _ => 1536,
    }
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    input: &'a [&'a str],
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
    index: usize,
}

#[async_trait]
impl Embedder for OpenAIEmbedder {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
        let req = EmbedRequest {
            model: &self.model,
            input: texts,
        };

        let resp = self
            .client
            .post("https://api.openai.com/v1/embeddings")
            .bearer_auth(&self.api_key)
            .json(&req)
            .send()
            .await
            .map_err(|e| EmbedError::Http(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(EmbedError::Http(format!("{status}: {body}")));
        }

        let mut body: EmbedResponse = resp
            .json()
            .await
            .map_err(|e| EmbedError::Http(e.to_string()))?;

        body.data.sort_by_key(|d| d.index);
        Ok(body.data.into_iter().map(|d| d.embedding).collect())
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> &str {
        &self.model_id
    }
}
