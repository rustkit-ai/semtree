pub mod analyze;
pub mod context;
pub mod index;
pub mod init;
pub mod search;
pub mod stats;

use anyhow::Result;
use std::sync::Arc;

use semtree_core::{EmbedBackend, SemtreeConfig, StoreBackend};
use semtree_embed::Embedder;
use semtree_store::VectorStore;

pub struct Backends {
    pub embedder: Arc<dyn Embedder>,
    pub store: Arc<dyn VectorStore>,
}

pub fn make_backends(config: &SemtreeConfig) -> Result<Backends> {
    let embedder: Arc<dyn Embedder> = match &config.embed.backend {
        EmbedBackend::Fastembed => Arc::new(semtree_embed::fastembed::FastEmbedder::new()?),
        EmbedBackend::OpenAI => {
            let key = config
                .embed
                .api_key
                .clone()
                .or_else(|| std::env::var("OPENAI_API_KEY").ok())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "openai backend requires OPENAI_API_KEY env var or embed.api_key in config"
                    )
                })?;
            Arc::new(semtree_embed::openai::OpenAIEmbedder::new(
                key,
                config.embed.model.clone(),
            ))
        }
        EmbedBackend::Ollama => Arc::new(semtree_embed::ollama::OllamaEmbedder::new(
            config.embed.url.clone(),
            config.embed.model.clone(),
        )),
    };

    // Size the store to the embedder actually in use, not a hard-coded 384 -
    // OpenAI (1536) and Ollama (768) produce wider vectors than fastembed.
    let dim = embedder.dimension();

    let store: Arc<dyn VectorStore> = match &config.store.backend {
        StoreBackend::Usearch => Arc::new(semtree_store::usearch::UsearchStore::new(dim)?),
        StoreBackend::Qdrant => Arc::new(semtree_store::qdrant::QdrantStore::new(
            dim,
            config.store.url.clone(),
            config.store.collection.clone(),
        )?),
    };

    Ok(Backends { embedder, store })
}
