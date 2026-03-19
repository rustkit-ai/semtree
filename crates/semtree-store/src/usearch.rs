use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

use async_trait::async_trait;
use semtree_embed::Embedding;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

use crate::{Hit, StoreError, VectorStore};

pub struct UsearchStore {
    dimensions: usize,
    index: Index,
    /// maps usearch integer key → chunk id string
    id_map: RwLock<HashMap<u64, String>>,
    next_key: RwLock<u64>,
}

impl UsearchStore {
    pub fn new(dimensions: usize) -> Result<Self, StoreError> {
        let index = Self::make_index(dimensions)?;
        Ok(Self {
            dimensions,
            index,
            id_map: RwLock::new(HashMap::new()),
            next_key: RwLock::new(0),
        })
    }

    fn make_index(dimensions: usize) -> Result<Index, StoreError> {
        let options = IndexOptions {
            dimensions,
            metric: MetricKind::Cos,
            quantization: ScalarKind::F32,
            ..Default::default()
        };
        let index = Index::new(&options).map_err(|e| StoreError::Init(e.to_string()))?;
        index
            .reserve(1024)
            .map_err(|e| StoreError::Init(e.to_string()))?;
        Ok(index)
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
        if self.index.size() == 0 {
            return Ok(vec![]);
        }
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
            self.index
                .remove(key)
                .map_err(|e| StoreError::Insert(e.to_string()))?;
            self.id_map.write().unwrap().remove(&key);
        }
        Ok(())
    }

    fn save(&self, path: &Path) -> Result<(), StoreError> {
        // save vector index
        let index_path = path.join("index.usearch");
        self.index
            .save(index_path.to_str().unwrap())
            .map_err(|e| StoreError::Init(e.to_string()))?;

        // save id_map + next_key as JSON
        let meta = serde_json::json!({
            "dimensions": self.dimensions,
            "next_key": *self.next_key.read().unwrap(),
            "id_map": self.id_map.read().unwrap().iter()
                .map(|(k, v)| (k.to_string(), v.clone()))
                .collect::<HashMap<String, String>>(),
        });
        let meta_path = path.join("meta.json");
        std::fs::write(meta_path, serde_json::to_string(&meta).unwrap())
            .map_err(|e| StoreError::Init(e.to_string()))?;

        Ok(())
    }

    fn load(&mut self, path: &Path) -> Result<(), StoreError> {
        let meta_path = path.join("meta.json");
        let raw =
            std::fs::read_to_string(meta_path).map_err(|e| StoreError::Init(e.to_string()))?;
        let meta: serde_json::Value =
            serde_json::from_str(&raw).map_err(|e| StoreError::Init(e.to_string()))?;

        let next_key: u64 = meta["next_key"].as_u64().unwrap_or(0);
        let id_map: HashMap<u64, String> = meta["id_map"]
            .as_object()
            .unwrap_or(&serde_json::Map::new())
            .iter()
            .filter_map(|(k, v)| Some((k.parse::<u64>().ok()?, v.as_str()?.to_string())))
            .collect();

        *self.next_key.write().unwrap() = next_key;
        *self.id_map.write().unwrap() = id_map;

        let index_path = path.join("index.usearch");
        self.index = Self::make_index(self.dimensions)?;
        self.index
            .load(index_path.to_str().unwrap())
            .map_err(|e| StoreError::Init(e.to_string()))?;

        Ok(())
    }

    fn len(&self) -> usize {
        self.index.size()
    }
}
