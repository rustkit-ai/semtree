pub mod context;
pub mod index;
pub mod init;
pub mod search;

use std::sync::Arc;
use anyhow::Result;

use semtree_embed::fastembed::FastEmbedder;
use semtree_store::usearch::UsearchStore;

// fastembed AllMiniLML6V2 produces 384-dim embeddings
pub const EMBED_DIM: usize = 384;

pub fn make_embedder() -> Result<Arc<FastEmbedder>> {
    Ok(Arc::new(FastEmbedder::new()?))
}

pub fn make_store() -> Result<Arc<UsearchStore>> {
    Ok(Arc::new(UsearchStore::new(EMBED_DIM)?))
}
