use semtree_store::Hit;

use crate::{LexicalIndex, RagError, SearchEngine};

// Reciprocal Rank Fusion constant. 60 is the value from the original
// Cormack et al. paper and the de-facto standard.
const RRF_K: f64 = 60.0;

/// How a query is matched against the index.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchMode {
    /// Fuse semantic (vector) and lexical (BM25) rankings via RRF. Default.
    Hybrid,
    /// Vector similarity only.
    Semantic,
    /// BM25 keyword matching only.
    Lexical,
}

impl SearchMode {
    /// Parses a mode name; returns `None` for unknown values.
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "hybrid" => Some(Self::Hybrid),
            "semantic" | "vector" => Some(Self::Semantic),
            "lexical" | "keyword" | "bm25" => Some(Self::Lexical),
            _ => None,
        }
    }
}

/// Combines a vector [`SearchEngine`] with a [`LexicalIndex`] so queries can be
/// matched by meaning, by keyword, or both (fused). Hybrid is what lets semtree
/// catch concepts a grep misses *and* keep the exact-identifier precision a
/// pure vector search loses.
pub struct HybridSearcher {
    engine: SearchEngine,
    lexical: LexicalIndex,
}

impl HybridSearcher {
    pub fn new(engine: SearchEngine, lexical: LexicalIndex) -> Self {
        Self { engine, lexical }
    }

    /// Returns up to `limit` hits for `query` under the given `mode`.
    pub async fn search(
        &self,
        query: &str,
        limit: usize,
        mode: SearchMode,
    ) -> Result<Vec<Hit>, RagError> {
        match mode {
            SearchMode::Semantic => self.engine.search(query, limit).await,
            SearchMode::Lexical => Ok(self
                .lexical
                .search(query, limit)
                .into_iter()
                .map(|(id, score)| Hit { id, score })
                .collect()),
            SearchMode::Hybrid => {
                let vector = self.engine.search(query, limit).await?;
                let lexical = self.lexical.search(query, limit);
                Ok(fuse_rrf(&vector, &lexical, limit))
            }
        }
    }
}

/// Reciprocal Rank Fusion: each ranker contributes `1 / (k + rank)` per
/// document, summed across rankers. Rank-based, so it's robust to the fact
/// that cosine scores and BM25 scores live on different scales.
fn fuse_rrf(vector: &[Hit], lexical: &[(String, f32)], limit: usize) -> Vec<Hit> {
    use std::collections::HashMap;

    let mut fused: HashMap<String, f64> = HashMap::new();
    for (rank, hit) in vector.iter().enumerate() {
        *fused.entry(hit.id.clone()).or_insert(0.0) += 1.0 / (RRF_K + (rank + 1) as f64);
    }
    for (rank, (id, _)) in lexical.iter().enumerate() {
        *fused.entry(id.clone()).or_insert(0.0) += 1.0 / (RRF_K + (rank + 1) as f64);
    }

    let mut ranked: Vec<(String, f64)> = fused.into_iter().collect();
    ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranked.truncate(limit);
    ranked
        .into_iter()
        .map(|(id, score)| Hit {
            id,
            score: score as f32,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hit(id: &str) -> Hit {
        Hit {
            id: id.to_string(),
            score: 0.0,
        }
    }

    #[test]
    fn rrf_rewards_agreement() {
        // "b" is ranked highly by both rankers; it should win even though it is
        // not #1 in either list.
        let vector = vec![hit("a"), hit("b"), hit("c")];
        let lexical = vec![("d".to_string(), 9.0), ("b".to_string(), 8.0)];
        let fused = fuse_rrf(&vector, &lexical, 10);
        assert_eq!(fused.first().map(|h| h.id.as_str()), Some("b"));
    }

    #[test]
    fn rrf_keeps_unique_hits_from_each_ranker() {
        let vector = vec![hit("a")];
        let lexical = vec![("b".to_string(), 1.0)];
        let fused = fuse_rrf(&vector, &lexical, 10);
        let ids: Vec<&str> = fused.iter().map(|h| h.id.as_str()).collect();
        assert!(ids.contains(&"a"));
        assert!(ids.contains(&"b"));
    }

    #[test]
    fn mode_parsing() {
        assert_eq!(SearchMode::parse("hybrid"), Some(SearchMode::Hybrid));
        assert_eq!(SearchMode::parse("SEMANTIC"), Some(SearchMode::Semantic));
        assert_eq!(SearchMode::parse("bm25"), Some(SearchMode::Lexical));
        assert_eq!(SearchMode::parse("nope"), None);
    }
}
