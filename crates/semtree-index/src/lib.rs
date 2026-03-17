use std::path::Path;
use std::sync::Arc;

use semtree_core::Language;
use semtree_embed::Embedder;
use semtree_extract::extract;
use semtree_store::VectorStore;
use semtree_tree::SemtreeParser;
use thiserror::Error;
use tracing::{debug, warn};

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("embed error: {0}")]
    Embed(#[from] semtree_embed::EmbedError),
    #[error("store error: {0}")]
    Store(#[from] semtree_store::StoreError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub struct Indexer {
    embedder: Arc<dyn Embedder>,
    store: Arc<dyn VectorStore>,
}

impl Indexer {
    pub fn new(embedder: Arc<dyn Embedder>, store: Arc<dyn VectorStore>) -> Self {
        Self { embedder, store }
    }

    pub async fn index_file(&self, path: &Path) -> Result<usize, IndexError> {
        let lang = Language::from_path(path);
        if lang == Language::Unknown {
            return Ok(0);
        }

        let parsed = match SemtreeParser::parse_file(path) {
            Ok(t) => t,
            Err(e) => {
                warn!("parse failed for {}: {e}", path.display());
                return Ok(0);
            }
        };

        let mut chunks = match extract(&parsed) {
            Ok(c) => c,
            Err(e) => {
                warn!("extract failed for {}: {e}", path.display());
                return Ok(0);
            }
        };

        // attach file path to chunks
        for chunk in &mut chunks {
            chunk.path = path.to_path_buf();
        }

        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedder.embed(&texts).await?;

        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            self.store.insert(&chunk.id, embedding).await?;
            debug!("indexed chunk {} ({})", chunk.id, chunk.name.as_deref().unwrap_or("?"));
        }

        Ok(chunks.len())
    }

    pub async fn index_dir(&self, dir: &Path) -> Result<usize, IndexError> {
        let mut total = 0;
        for entry in walkdir(dir) {
            total += self.index_file(&entry).await?;
        }
        Ok(total)
    }
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                paths.extend(walkdir(&path));
            } else {
                paths.push(path);
            }
        }
    }
    paths
}
