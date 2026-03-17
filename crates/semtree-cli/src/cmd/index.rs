use std::path::Path;

use anyhow::Result;
use semtree_core::SemtreeConfig;
use semtree_rag::{ChunkRegistry, Indexer};

use super::make_backends;

pub async fn run(path: &Path, index_dir: &Path, config: &SemtreeConfig) -> Result<()> {
    std::fs::create_dir_all(index_dir)?;

    let backends = make_backends(config)?;
    let store = backends.store;
    let mut registry = ChunkRegistry::default();

    let indexer = Indexer::new(backends.embedder, store.clone());

    println!("Indexing {}...", path.display());
    let n = indexer.index_dir(path, &mut registry).await?;

    store.save(index_dir)?;
    registry.save(index_dir)?;

    println!("Done. Indexed {n} chunks → {}", index_dir.display());
    Ok(())
}
