pub mod context;
pub mod index;
pub mod init;
pub mod search;

use std::sync::Arc;
use anyhow::{bail, Result};

use semtree_core::{EmbedBackend, SemtreeConfig, StoreBackend};
use semtree_embed::fastembed::FastEmbedder;
use semtree_embed::Embedder;
use semtree_store::usearch::UsearchStore;
use semtree_store::VectorStore;

// fastembed AllMiniLML6V2 produces 384-dim embeddings
pub const EMBED_DIM: usize = 384;

pub struct Backends {
    pub embedder: Arc<dyn Embedder>,
    pub store: Arc<dyn VectorStore>,
}

pub fn make_backends(config: &SemtreeConfig) -> Result<Backends> {
    let embedder: Arc<dyn Embedder> = match &config.embed.backend {
        EmbedBackend::Fastembed => Arc::new(FastEmbedder::new()?),
        EmbedBackend::OpenAI => bail!(
            "openai backend not yet implemented — set OPENAI_API_KEY and use backend = \"openai\""
        ),
        EmbedBackend::Ollama => bail!(
            "ollama backend not yet implemented — set OPENAI_API_KEY and use backend = \"openai\""
        ),
    };

    let store: Arc<dyn VectorStore> = match &config.store.backend {
        StoreBackend::Usearch => Arc::new(UsearchStore::new(EMBED_DIM)?),
        StoreBackend::Qdrant => bail!("qdrant backend not yet implemented"),
    };

    Ok(Backends { embedder, store })
}
