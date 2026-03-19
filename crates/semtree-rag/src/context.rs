use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{RagError, SearchEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnippet {
    pub chunk_id: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextWindow {
    pub query: String,
    pub snippets: Vec<ContextSnippet>,
    pub prompt: String,
}

pub struct ContextBuilder {
    engine: Arc<SearchEngine>,
    max_chunks: usize,
}

impl ContextBuilder {
    pub fn new(engine: Arc<SearchEngine>) -> Self {
        Self {
            engine,
            max_chunks: 5,
        }
    }

    pub fn with_max_chunks(mut self, n: usize) -> Self {
        self.max_chunks = n;
        self
    }

    pub async fn build(&self, query: &str) -> Result<ContextWindow, RagError> {
        let hits = self.engine.search(query, self.max_chunks).await?;

        let snippets: Vec<ContextSnippet> = hits
            .iter()
            .map(|h| ContextSnippet {
                chunk_id: h.id.clone(),
                score: h.score,
            })
            .collect();

        let context_block = snippets
            .iter()
            .enumerate()
            .map(|(i, s)| format!("[{}] chunk_id={} (score={:.3})", i + 1, s.chunk_id, s.score))
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "Use the following code context to answer the question.\n\n{context_block}\n\nQuestion: {query}"
        );

        Ok(ContextWindow {
            query: query.to_string(),
            snippets,
            prompt,
        })
    }
}
