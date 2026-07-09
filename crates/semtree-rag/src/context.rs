use std::sync::Arc;

use serde::{Deserialize, Serialize};

use crate::{ChunkRegistry, RagError, SearchEngine};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextSnippet {
    pub chunk_id: String,
    pub score: f32,
    pub path: String,
    pub name: Option<String>,
    /// 1-based start line in the source file.
    pub start_line: usize,
    /// Raw source text of the chunk.
    pub content: String,
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

    /// Builds a context window for `query`, resolving each hit against
    /// `registry` so the prompt carries real code (not just chunk ids).
    pub async fn build(
        &self,
        query: &str,
        registry: &ChunkRegistry,
    ) -> Result<ContextWindow, RagError> {
        let hits = self.engine.search(query, self.max_chunks).await?;

        let snippets: Vec<ContextSnippet> = hits
            .iter()
            .filter_map(|h| {
                registry.get(&h.id).map(|c| ContextSnippet {
                    chunk_id: h.id.clone(),
                    score: h.score,
                    path: c.path.display().to_string(),
                    name: c.name.clone(),
                    start_line: c.span.start_line + 1,
                    content: c.content.clone(),
                })
            })
            .collect();

        let context_block = snippets
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let header = match &s.name {
                    Some(name) => format!("[{}] {}:{} - {name}", i + 1, s.path, s.start_line),
                    None => format!("[{}] {}:{}", i + 1, s.path, s.start_line),
                };
                format!("{header}\n```\n{}\n```", s.content)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

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
