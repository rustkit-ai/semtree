use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::RagError;

/// On-disk layout version. Bump when the manifest struct changes shape so an
/// older file is treated as incompatible rather than mis-parsed.
const MANIFEST_VERSION: u32 = 1;

/// Version of the parse/chunk logic. Bump whenever chunk boundaries or IDs can
/// change (grammar upgrade, new chunk kinds); a mismatch forces a full rebuild
/// so stale chunk IDs never linger in the store.
const CHUNKER_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileEntry {
    /// Stable content hash for change detection (blake3, hex).
    pub content_hash: String,
    /// Chunk IDs produced from this file.
    pub chunk_ids: Vec<String>,
}

/// Tracks per-file state to enable incremental re-indexing.
///
/// The header ([`manifest_version`], [`chunker_version`], [`embedder`], [`store`])
/// pins the assumptions the entries were built under. When any of them no longer
/// match the current pipeline, the index must be rebuilt from scratch - see
/// [`is_compatible_with`](FileManifest::is_compatible_with).
#[derive(Debug, Serialize, Deserialize)]
pub struct FileManifest {
    #[serde(default)]
    manifest_version: u32,
    #[serde(default)]
    chunker_version: u32,
    /// Fingerprint of the embedder that produced the stored vectors.
    #[serde(default)]
    embedder: String,
    /// Fingerprint of the store the vectors live in (e.g. its distance metric):
    /// switching it re-ranks results, so it invalidates the index too.
    #[serde(default)]
    store: String,
    entries: HashMap<PathBuf, FileEntry>,
}

impl Default for FileManifest {
    fn default() -> Self {
        Self {
            manifest_version: MANIFEST_VERSION,
            chunker_version: CHUNKER_VERSION,
            embedder: String::new(),
            store: String::new(),
            entries: HashMap::new(),
        }
    }
}

impl FileManifest {
    /// A fresh manifest pinned to the current pipeline, the given embedder
    /// fingerprint (see [`Embedder::fingerprint`](semtree_embed::Embedder::fingerprint)),
    /// and the store fingerprint (e.g. its [`Metric`](semtree_store::Metric)).
    pub fn new(
        embedder_fingerprint: impl Into<String>,
        store_fingerprint: impl Into<String>,
    ) -> Self {
        Self {
            embedder: embedder_fingerprint.into(),
            store: store_fingerprint.into(),
            ..Self::default()
        }
    }

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

    /// Whether the stored entries can be trusted for an incremental update given
    /// the current embedder and store. False means the schema, the chunker, the
    /// embedder, or the store changed and the index must be rebuilt from scratch.
    pub fn is_compatible_with(&self, embedder_fingerprint: &str, store_fingerprint: &str) -> bool {
        self.manifest_version == MANIFEST_VERSION
            && self.chunker_version == CHUNKER_VERSION
            && self.embedder == embedder_fingerprint
            && self.store == store_fingerprint
    }

    /// The embedder fingerprint the stored vectors were built with.
    pub fn embedder(&self) -> &str {
        &self.embedder
    }

    /// The store fingerprint the stored vectors were built with.
    pub fn store(&self) -> &str {
        &self.store
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

/// Stable content hash. blake3 is deterministic across releases and platforms,
/// so a manifest persisted today still matches an unchanged file tomorrow -
/// unlike `DefaultHasher`, whose output std makes no stability promise about.
fn content_hash(content: &str) -> String {
    blake3::hash(content.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_content_change() {
        let mut m = FileManifest::new("fastembed:AllMiniLML6V2/384d", "cosine");
        let path = Path::new("src/lib.rs");
        assert!(m.is_changed(path, "fn a() {}"), "new file is changed");

        m.record(path.to_path_buf(), "fn a() {}", vec!["id1".into()]);
        assert!(
            !m.is_changed(path, "fn a() {}"),
            "same content is unchanged"
        );
        assert!(m.is_changed(path, "fn b() {}"), "edited content is changed");
    }

    #[test]
    fn content_hash_is_stable_and_deterministic() {
        // A fixed input must always hash to the same value, or every restart
        // would look like a full-tree change.
        assert_eq!(content_hash("hello"), content_hash("hello"));
        assert_ne!(content_hash("hello"), content_hash("world"));
    }

    #[test]
    fn incompatible_on_embedder_change() {
        let m = FileManifest::new("fastembed:AllMiniLML6V2/384d", "cosine");
        assert!(m.is_compatible_with("fastembed:AllMiniLML6V2/384d", "cosine"));
        assert!(!m.is_compatible_with("openai:text-embedding-3-small/1536d", "cosine"));
    }

    #[test]
    fn incompatible_on_store_change() {
        // Same embedder, different distance metric: results would re-rank, so
        // the index is not reusable.
        let m = FileManifest::new("fastembed:AllMiniLML6V2/384d", "cosine");
        assert!(!m.is_compatible_with("fastembed:AllMiniLML6V2/384d", "euclidean"));
    }

    #[test]
    fn incompatible_on_version_change() {
        let mut m = FileManifest::new("e", "cosine");
        m.chunker_version = CHUNKER_VERSION + 1;
        assert!(
            !m.is_compatible_with("e", "cosine"),
            "bumped chunker forces rebuild"
        );

        let mut m = FileManifest::new("e", "cosine");
        m.manifest_version = MANIFEST_VERSION + 1;
        assert!(
            !m.is_compatible_with("e", "cosine"),
            "bumped schema forces rebuild"
        );
    }

    #[test]
    fn legacy_manifest_without_header_is_incompatible() {
        // A pre-header manifest deserializes with zeroed version fields (serde
        // default), so it never masquerades as compatible.
        let legacy: FileManifest = serde_json::from_str(r#"{"entries":{}}"#).unwrap();
        assert_eq!(legacy.manifest_version, 0);
        assert!(!legacy.is_compatible_with("fastembed:AllMiniLML6V2/384d", "cosine"));
    }
}
