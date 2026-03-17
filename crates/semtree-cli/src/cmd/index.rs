use std::path::Path;

use anyhow::Result;
use semtree_rag::{ChunkRegistry, Indexer};
use semtree_store::VectorStore;

use super::{make_embedder, make_store};

pub async fn run(path: &Path, index_dir: &Path) -> Result<()> {
    std::fs::create_dir_all(index_dir)?;

    let embedder = make_embedder()?;
    let store = make_store()?;
    let mut registry = ChunkRegistry::default();

    let indexer = Indexer::new(embedder, store.clone());

    println!("Indexing {}...", path.display());
    let n = indexer.index_dir(path, &mut registry).await?;

    store.save(index_dir)?;
    registry.save(index_dir)?;

    println!("Done. Indexed {n} chunks → {}", index_dir.display());
    Ok(())
}
