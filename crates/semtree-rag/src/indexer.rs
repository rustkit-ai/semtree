use std::path::Path;
use std::sync::Arc;

use semtree_embed::Embedder;
use semtree_parse::extract_file;
use semtree_store::VectorStore;
use tracing::{debug, warn};

use crate::{ChunkRegistry, RagError};

pub struct Indexer {
    embedder: Arc<dyn Embedder>,
    store: Arc<dyn VectorStore>,
}

impl Indexer {
    pub fn new(embedder: Arc<dyn Embedder>, store: Arc<dyn VectorStore>) -> Self {
        Self { embedder, store }
    }

    pub async fn index_file(
        &self,
        path: &Path,
        registry: &mut ChunkRegistry,
    ) -> Result<usize, RagError> {
        let mut chunks = match extract_file(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("skipping {}: {e}", path.display());
                return Ok(0);
            }
        };

        if chunks.is_empty() {
            return Ok(0);
        }

        for chunk in &mut chunks {
            chunk.path = path.to_path_buf();
        }

        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedder.embed(&texts).await?;

        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            self.store.insert(&chunk.id, embedding).await?;
            registry.insert(chunk.clone());
            debug!("indexed {}", chunk.name.as_deref().unwrap_or(&chunk.id));
        }

        Ok(chunks.len())
    }

    pub async fn index_dir(
        &self,
        dir: &Path,
        registry: &mut ChunkRegistry,
    ) -> Result<usize, RagError> {
        let mut total = 0;
        for path in walkdir(dir) {
            total += self.index_file(&path, registry).await?;
        }
        Ok(total)
    }
}

const IGNORED_DIRS: &[&str] = &[
    // dependencies
    "node_modules",
    "vendor",
    ".pnp",
    // build output
    "target",
    "dist",
    "build",
    "out",
    ".next",
    ".nuxt",
    "__pycache__",
    ".pytest_cache",
    ".mypy_cache",
    ".ruff_cache",
    // venvs
    ".venv",
    "venv",
    "env",
    ".env",
    // version control
    ".git",
    ".svn",
    ".hg",
    // index itself
    ".semtree",
    // misc
    ".idea",
    ".vscode",
    "coverage",
    ".turbo",
    ".cache",
];

fn is_ignored(dir_name: &str) -> bool {
    IGNORED_DIRS.contains(&dir_name)
}

fn walkdir(dir: &Path) -> Vec<std::path::PathBuf> {
    let mut paths = Vec::new();
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !is_ignored(name) {
                    paths.extend(walkdir(&path));
                }
            } else {
                paths.push(path);
            }
        }
    }
    paths
}
