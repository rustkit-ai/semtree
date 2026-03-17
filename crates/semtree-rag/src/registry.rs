use std::collections::HashMap;
use std::path::Path;

use semtree_core::Chunk;

use crate::RagError;

/// Persists chunk metadata alongside the vector index.
#[derive(Default)]
pub struct ChunkRegistry {
    chunks: HashMap<String, Chunk>,
}

impl ChunkRegistry {
    pub fn insert(&mut self, chunk: Chunk) {
        self.chunks.insert(chunk.id.clone(), chunk);
    }

    pub fn get(&self, id: &str) -> Option<&Chunk> {
        self.chunks.get(id)
    }

    pub fn len(&self) -> usize {
        self.chunks.len()
    }

    pub fn is_empty(&self) -> bool {
        self.chunks.is_empty()
    }

    pub fn save(&self, path: &Path) -> Result<(), RagError> {
        let data = serde_json::to_string(&self.chunks)
            .map_err(|e| RagError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        std::fs::write(path.join("chunks.json"), data)?;
        Ok(())
    }

    pub fn load(&mut self, path: &Path) -> Result<(), RagError> {
        let raw = std::fs::read_to_string(path.join("chunks.json"))?;
        self.chunks = serde_json::from_str(&raw)
            .map_err(|e| RagError::Io(std::io::Error::new(std::io::ErrorKind::Other, e)))?;
        Ok(())
    }
}
