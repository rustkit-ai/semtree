use std::path::{Path, PathBuf};
use std::sync::Arc;

use semtree_embed::Embedder;
use semtree_parse::extract_file;
use semtree_store::VectorStore;
use tracing::{debug, warn};

use crate::{ChunkRegistry, FileManifest, RagError};

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
        manifest: Option<&mut FileManifest>,
    ) -> Result<usize, RagError> {
        let content = match std::fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("skipping {}: {e}", path.display());
                return Ok(0);
            }
        };

        if let Some(ref manifest) = manifest
            && !manifest.is_changed(path, &content)
        {
            debug!("unchanged, skipping {}", path.display());
            return Ok(0);
        }

        // Remove stale chunks for this file before re-indexing
        if let Some(ref manifest) = manifest {
            for old_id in manifest.chunk_ids(path) {
                registry.remove(old_id);
                let _ = self.store.delete(old_id).await;
            }
        }

        let chunks = match extract_file(path) {
            Ok(c) => c,
            Err(e) => {
                warn!("skipping {}: {e}", path.display());
                return Ok(0);
            }
        };

        if chunks.is_empty() {
            if let Some(manifest) = manifest {
                manifest.record(path.to_path_buf(), &content, vec![]);
            }
            return Ok(0);
        }

        // `extract_file` already stamps each chunk with its path and a
        // path-scoped id, so chunks from different files never collide.
        let texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();

        // Embed in batches the backend is comfortable with, rather than handing
        // it every chunk at once: a large generated file could otherwise blow up
        // local memory or exceed a remote API's request limit.
        let batch = self.embedder.max_batch_size().max(1);
        let mut embeddings = Vec::with_capacity(texts.len());
        for group in texts.chunks(batch) {
            embeddings.extend(self.embedder.embed(group).await?);
        }

        let mut chunk_ids = Vec::with_capacity(chunks.len());
        for (chunk, embedding) in chunks.iter().zip(embeddings.iter()) {
            self.store.insert(&chunk.id, embedding).await?;
            registry.insert(chunk.clone());
            chunk_ids.push(chunk.id.clone());
            debug!("indexed {}", chunk.name.as_deref().unwrap_or(&chunk.id));
        }

        if let Some(manifest) = manifest {
            manifest.record(path.to_path_buf(), &content, chunk_ids);
        }

        Ok(chunks.len())
    }

    /// Index a directory, skipping unchanged files when a manifest is provided.
    /// Calls `on_progress(files_done, total_files)` after each file.
    pub async fn index_dir(
        &self,
        dir: &Path,
        registry: &mut ChunkRegistry,
        manifest: Option<&mut FileManifest>,
        on_progress: impl Fn(usize, usize),
    ) -> Result<usize, RagError> {
        let paths = collect_indexable_files(dir);
        let total = paths.len();
        let mut total_chunks = 0;

        // Collect manifest as Option<&mut FileManifest> once, then use it per-file.
        // We can't borrow it mutably in a loop via the outer Option easily, so we
        // take ownership and pass per-file slices of info manually.
        let mut manifest = manifest;

        for (done, path) in paths.iter().enumerate() {
            total_chunks += self
                .index_file(path, registry, manifest.as_deref_mut())
                .await?;
            on_progress(done + 1, total);
        }

        // Remove manifest entries for files that no longer exist
        if let Some(ref mut m) = manifest {
            let stale: Vec<PathBuf> = m.paths().filter(|p| !p.exists()).cloned().collect();
            for path in stale {
                for old_id in m.chunk_ids(&path).to_vec() {
                    registry.remove(&old_id);
                    let _ = self.store.delete(&old_id).await;
                }
                m.remove(&path);
            }
        }

        Ok(total_chunks)
    }
}

/// Collect all indexable files under `dir`, respecting the ignore list.
pub fn collect_indexable_files(dir: &Path) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    walkdir(dir, &mut paths);
    paths
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

fn walkdir(dir: &Path, paths: &mut Vec<PathBuf>) {
    if let Ok(entries) = std::fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                if !is_ignored(name) {
                    walkdir(&path, paths);
                }
            } else {
                paths.push(path);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use async_trait::async_trait;
    use semtree_embed::{EmbedError, Embedding};
    use semtree_store::{Hit, Metric, StoreError};

    use super::*;

    /// Records the size of every `embed` call so a test can assert batching.
    struct BatchRecorder {
        calls: Mutex<Vec<usize>>,
    }

    #[async_trait]
    impl Embedder for BatchRecorder {
        async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
            self.calls.lock().unwrap().push(texts.len());
            Ok(texts.iter().map(|_| vec![0.0; 3]).collect())
        }
        fn dimension(&self) -> usize {
            3
        }
        fn model_id(&self) -> &str {
            "test"
        }
        fn max_batch_size(&self) -> usize {
            2
        }
    }

    #[derive(Default)]
    struct MemStore {
        ids: Mutex<Vec<String>>,
    }

    #[async_trait]
    impl VectorStore for MemStore {
        async fn insert(&self, id: &str, _embedding: &Embedding) -> Result<(), StoreError> {
            self.ids.lock().unwrap().push(id.to_string());
            Ok(())
        }
        async fn search(&self, _query: &Embedding, _top_k: usize) -> Result<Vec<Hit>, StoreError> {
            Ok(vec![])
        }
        async fn delete(&self, id: &str) -> Result<(), StoreError> {
            self.ids.lock().unwrap().retain(|x| x != id);
            Ok(())
        }
        fn save(&self, _path: &Path) -> Result<(), StoreError> {
            Ok(())
        }
        fn load(&mut self, _path: &Path) -> Result<(), StoreError> {
            Ok(())
        }
        fn len(&self) -> usize {
            self.ids.lock().unwrap().len()
        }
        fn metric(&self) -> Metric {
            Metric::Cosine
        }
    }

    #[tokio::test]
    async fn embeds_in_batches_bounded_by_max_batch_size() {
        let dir = std::env::temp_dir().join("semtree_indexer_batch_test");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let file = dir.join("a.rs");
        std::fs::write(&file, "fn a() {}\nfn b() {}\nfn c() {}\nfn d() {}\n").unwrap();

        let embedder = Arc::new(BatchRecorder {
            calls: Mutex::new(vec![]),
        });
        let store = Arc::new(MemStore::default());
        let mut registry = ChunkRegistry::default();

        let n = Indexer::new(embedder.clone(), store.clone())
            .index_file(&file, &mut registry, None)
            .await
            .unwrap();

        let calls = embedder.calls.lock().unwrap().clone();
        assert!(n > 0, "file produced chunks");
        assert_eq!(store.len(), n, "every chunk was stored");
        assert_eq!(
            calls.iter().sum::<usize>(),
            n,
            "every chunk was embedded exactly once"
        );
        assert!(
            calls.iter().all(|&c| (1..=2).contains(&c)),
            "no call exceeds max_batch_size: {calls:?}"
        );

        let _ = std::fs::remove_dir_all(&dir);
    }
}
