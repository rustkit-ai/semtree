use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::RagError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Hash of file content for change detection
    pub content_hash: u64,
    /// Chunk IDs produced from this file
    pub chunk_ids: Vec<String>,
}

/// Tracks per-file state to enable incremental re-indexing.
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct FileManifest {
    entries: HashMap<PathBuf, FileEntry>,
}

impl FileManifest {
    pub fn load(index_dir: &Path) -> Self {
        let path = index_dir.join("manifest.json");
        std::fs::read_to_string(&path)
            .ok()
            .and_then(|raw| serde_json::from_str(&raw).ok())
            .unwrap_or_default()
    }

    pub fn save(&self, index_dir: &Path) -> Result<(), RagError> {
        let path = index_dir.join("manifest.json");
        let data =
            serde_json::to_string(self).map_err(|e| RagError::Io(std::io::Error::other(e)))?;
        std::fs::write(path, data)?;
        Ok(())
    }

    /// Returns `true` if the file is new or its content has changed.
    pub fn is_changed(&self, path: &Path, content: &str) -> bool {
        let hash = content_hash(content);
        match self.entries.get(path) {
            Some(entry) => entry.content_hash != hash,
            None => true,
        }
    }

    /// Returns the chunk IDs that were last indexed from this file.
    pub fn chunk_ids(&self, path: &Path) -> &[String] {
        self.entries
            .get(path)
            .map(|e| e.chunk_ids.as_slice())
            .unwrap_or(&[])
    }

    /// Record the result of indexing a file.
    pub fn record(&mut self, path: PathBuf, content: &str, chunk_ids: Vec<String>) {
        self.entries.insert(
            path,
            FileEntry {
                content_hash: content_hash(content),
                chunk_ids,
            },
        );
    }

    /// Remove a file entry (e.g. when the file is deleted).
    pub fn remove(&mut self, path: &Path) -> Option<FileEntry> {
        self.entries.remove(path)
    }

    /// Returns all tracked paths.
    pub fn paths(&self) -> impl Iterator<Item = &PathBuf> {
        self.entries.keys()
    }
}

fn content_hash(content: &str) -> u64 {
    let mut h = DefaultHasher::new();
    content.hash(&mut h);
    h.finish()
}
