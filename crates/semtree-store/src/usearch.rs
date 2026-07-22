use std::collections::HashMap;
use std::path::Path;
use std::sync::RwLock;

use async_trait::async_trait;
use semtree_embed::Embedding;
use usearch::{Index, IndexOptions, MetricKind, ScalarKind};

use crate::{Hit, Metric, StoreError, VectorStore};

pub struct UsearchStore {
    dimensions: usize,
    metric: Metric,
    index: Index,
    /// maps usearch integer key → chunk id string
    id_map: RwLock<HashMap<u64, String>>,
    next_key: RwLock<u64>,
}

impl UsearchStore {
    /// A store using cosine similarity, the default for normalized text embeddings.
    pub fn new(dimensions: usize) -> Result<Self, StoreError> {
        Self::with_metric(dimensions, Metric::Cosine)
    }

    pub fn with_metric(dimensions: usize, metric: Metric) -> Result<Self, StoreError> {
        let index = Self::make_index(dimensions, metric)?;
        Ok(Self {
            dimensions,
            metric,
            index,
            id_map: RwLock::new(HashMap::new()),
            next_key: RwLock::new(0),
        })
    }

    fn make_index(dimensions: usize, metric: Metric) -> Result<Index, StoreError> {
        let options = IndexOptions {
            dimensions,
            metric: match metric {
                Metric::Cosine => MetricKind::Cos,
                Metric::Euclidean => MetricKind::L2sq,
                Metric::DotProduct => MetricKind::IP,
            },
            quantization: ScalarKind::F32,
            ..Default::default()
        };
        let index = Index::new(&options).map_err(|e| StoreError::Init(e.to_string()))?;
        index
            .reserve(1024)
            .map_err(|e| StoreError::Init(e.to_string()))?;
        Ok(index)
    }

    /// Turn a usearch distance into a higher-is-better similarity score. Cosine
    /// and dot-product distances are `1 - similarity`; L2² has no upper bound,
    /// so it is folded into `(0, 1]` monotonically.
    fn distance_to_score(&self, distance: f32) -> f32 {
        match self.metric {
            Metric::Cosine | Metric::DotProduct => 1.0 - distance,
            Metric::Euclidean => 1.0 / (1.0 + distance),
        }
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
                    score: self.distance_to_score(*dist),
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
            "metric": self.metric,
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

        // Reject a reopen under a different metric: the persisted vectors were
        // ranked one way and cannot be re-ranked another without a rebuild.
        if let Some(saved) = meta.get("metric")
            && let Ok(saved) = serde_json::from_value::<Metric>(saved.clone())
            && saved != self.metric
        {
            return Err(StoreError::Init(format!(
                "index metric mismatch: on disk {saved}, requested {}",
                self.metric
            )));
        }

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
        self.index = Self::make_index(self.dimensions, self.metric)?;
        self.index
            .load(index_path.to_str().unwrap())
            .map_err(|e| StoreError::Init(e.to_string()))?;

        Ok(())
    }

    fn len(&self) -> usize {
        self.index.size()
    }

    fn metric(&self) -> Metric {
        self.metric
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn score_is_higher_for_nearer_vectors() {
        // Cosine/dot: score = 1 - distance. Euclidean: closer (smaller L2²) must
        // still rank higher, i.e. a larger score.
        let cos = UsearchStore::with_metric(4, Metric::Cosine).unwrap();
        assert!(cos.distance_to_score(0.1) > cos.distance_to_score(0.9));

        let l2 = UsearchStore::with_metric(4, Metric::Euclidean).unwrap();
        assert!(l2.distance_to_score(0.1) > l2.distance_to_score(5.0));
        assert!(l2.distance_to_score(0.0) <= 1.0);
    }

    #[test]
    fn reload_rejects_metric_mismatch() {
        let dir = std::env::temp_dir().join("semtree_store_metric_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();

        let saved = UsearchStore::with_metric(4, Metric::Cosine).unwrap();
        saved.save(&dir).unwrap();

        // Same metric reloads fine...
        let mut same = UsearchStore::with_metric(4, Metric::Cosine).unwrap();
        assert!(same.load(&dir).is_ok());

        // ...a different metric is refused rather than silently mis-ranking.
        let mut other = UsearchStore::with_metric(4, Metric::Euclidean).unwrap();
        assert!(other.load(&dir).is_err());

        let _ = std::fs::remove_dir_all(&dir);
    }
}
