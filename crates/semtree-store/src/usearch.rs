use std::collections::HashMap;
use std::sync::RwLock;

use async_trait::async_trait;
use semtree_embed::Embedding;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

use crate::{Hit, StoreError, VectorStore};

pub struct UsearchStore {
    index: Index,
    /// maps usearch integer key → chunk id string
    id_map: RwLock<HashMap<u64, String>>,
    next_key: RwLock<u64>,
}

impl UsearchStore {
    pub fn new(dimensions: usize) -> Result<Self, StoreError> {
        let options = IndexOptions {
            dimensions,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            ..Default::default()
        };
        let index = Index::new(&options).map_err(|e| StoreError::Init(e.to_string()))?;
        index.reserve(1024).map_err(|e| StoreError::Init(e.to_string()))?;
        Ok(Self {
            index,
            id_map: RwLock::new(HashMap::new()),
            next_key: RwLock::new(0),
        })
    }
}

#[async_trait]
impl VectorStore for UsearchStore {
    async fn insert(&self, id: &str, embedding: &Embedding) -> Result<(), StoreError> {
        let key = {
            let mut k = self.next_key.write().unwrap();
            let current = *k;
            *k += 1;
            current
        };
        self.index
            .add(key, embedding)
            .map_err(|e| StoreError::Insert(e.to_string()))?;
        self.id_map.write().unwrap().insert(key, id.to_string());
        Ok(())
    }

    async fn search(&self, query: &Embedding, top_k: usize) -> Result<Vec<Hit>, StoreError> {
        let results = self
            .index
            .search(query, top_k)
            .map_err(|e| StoreError::Search(e.to_string()))?;

        let map = self.id_map.read().unwrap();
        let hits = results
            .keys
            .iter()
            .zip(results.distances.iter())
            .filter_map(|(key, dist)| {
                map.get(key).map(|id| Hit {
                    id: id.clone(),
                    score: 1.0 - dist,
                })
            })
            .collect();
        Ok(hits)
    }

    async fn delete(&self, id: &str) -> Result<(), StoreError> {
        let map = self.id_map.read().unwrap();
        if let Some((&key, _)) = map.iter().find(|(_, v)| v.as_str() == id) {
            drop(map);
            self.index.remove(key).map_err(|e| StoreError::Insert(e.to_string()))?;
            self.id_map.write().unwrap().remove(&key);
        }
        Ok(())
    }
}
