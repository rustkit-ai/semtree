use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use async_trait::async_trait;
use reqwest::Client;
use semtree_embed::Embedding;
use serde::Deserialize;
use serde_json::json;

use crate::{Hit, StoreError, VectorStore};

pub struct QdrantStore {
    client: Client,
    base_url: String,
    collection: String,
    dimensions: usize,
    count: AtomicUsize,
    initialized: AtomicBool,
}

impl QdrantStore {
    pub fn new(
        dimensions: usize,
        url: Option<String>,
        collection: Option<String>,
    ) -> Result<Self, StoreError> {
        Ok(Self {
            client: Client::new(),
            base_url: url
                .or_else(|| std::env::var("QDRANT_URL").ok())
                .unwrap_or_else(|| "http://localhost:6333".to_string())
                .trim_end_matches('/')
                .to_string(),
            collection: collection.unwrap_or_else(|| "semtree".to_string()),
            dimensions,
            count: AtomicUsize::new(0),
            initialized: AtomicBool::new(false),
        })
    }

    async fn ensure_collection(&self) -> Result<(), StoreError> {
        if self.initialized.load(Ordering::Relaxed) {
            return Ok(());
        }

        let url = format!("{}/collections/{}", self.base_url, self.collection);

        // Check if collection exists
        let exists = self
            .client
            .get(&url)
            .send()
            .await
            .map(|r| r.status().is_success())
            .unwrap_or(false);

        if !exists {
            let body = json!({
                "vectors": {
                    "size": self.dimensions,
                    "distance": "Cosine"
                }
            });
            let resp = self
                .client
                .put(&url)
                .json(&body)
                .send()
                .await
                .map_err(|e| StoreError::Init(e.to_string()))?;
            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                return Err(StoreError::Init(format!(
                    "create collection {status}: {text}"
                )));
            }
        }

        self.initialized.store(true, Ordering::Relaxed);
        Ok(())
    }

    fn chunk_id_to_u64(id: &str) -> u64 {
        u64::from_str_radix(id, 16).unwrap_or_else(|_| {
            id.bytes().fold(0xcbf29ce484222325u64, |acc, b| {
                acc.wrapping_mul(0x100000001b3).wrapping_add(b as u64)
            })
        })
    }
}

#[derive(Deserialize)]
struct SearchResult {
    result: Vec<SearchHit>,
}

#[derive(Deserialize)]
struct SearchHit {
    id: u64,
    score: f32,
    payload: Option<serde_json::Value>,
}

#[async_trait]
impl VectorStore for QdrantStore {
    async fn insert(&self, id: &str, embedding: &Embedding) -> Result<(), StoreError> {
        self.ensure_collection().await?;

        let point_id = Self::chunk_id_to_u64(id);
        let body = json!({
            "points": [{
                "id": point_id,
                "vector": embedding,
                "payload": { "chunk_id": id }
            }]
        });

        let url = format!("{}/collections/{}/points", self.base_url, self.collection);
        let resp = self
            .client
            .put(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| StoreError::Insert(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(StoreError::Insert(format!("{status}: {text}")));
        }

        self.count.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<Hit>, StoreError> {
        self.ensure_collection().await?;

        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, self.collection
        );
        let body = json!({
            "vector": query,
            "limit": top_k,
            "with_payload": true
        });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| StoreError::Search(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let text = resp.text().await.unwrap_or_default();
            return Err(StoreError::Search(format!("{status}: {text}")));
        }

        let result: SearchResult = resp
            .json()
            .await
            .map_err(|e| StoreError::Search(e.to_string()))?;

        let hits = result
            .result
            .into_iter()
            .filter_map(|h| {
                let chunk_id = h
                    .payload
                    .and_then(|p| p["chunk_id"].as_str().map(|s| s.to_string()))
                    .unwrap_or_else(|| format!("{:x}", h.id));
                Some(Hit {
                    id: chunk_id,
                    score: h.score,
                })
            })
            .collect();

        Ok(hits)
    }

    async fn delete(&self, id: &str) -> Result<(), StoreError> {
        self.ensure_collection().await?;

        let point_id = Self::chunk_id_to_u64(id);
        let url = format!(
            "{}/collections/{}/points/delete",
            self.base_url, self.collection
        );
        let body = json!({ "points": [point_id] });

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await
            .map_err(|e| StoreError::Http(e.to_string()))?;

        if resp.status().is_success() {
            self.count.fetch_sub(1, Ordering::Relaxed);
        }

        Ok(())
    }

    fn save(&self, _path: &std::path::Path) -> Result<(), StoreError> {
        Ok(())
    }

    fn load(&mut self, _path: &std::path::Path) -> Result<(), StoreError> {
        Ok(())
    }

    fn len(&self) -> usize {
        self.count.load(Ordering::Relaxed)
    }
}
