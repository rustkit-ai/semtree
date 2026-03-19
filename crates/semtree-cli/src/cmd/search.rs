use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Result};
use semtree_core::SemtreeConfig;
use semtree_rag::{ChunkRegistry, SearchEngine};

use super::make_backends;

pub async fn run(
    query: &str,
    top_k: usize,
    index_dir: &Path,
    config: &SemtreeConfig,
) -> Result<()> {
    if !index_dir.exists() {
        bail!(
            "No index found at {}. Run `semtree index <path>` first.",
            index_dir.display()
        );
    }

    let backends = make_backends(config)?;
    let mut store = backends.store;
    Arc::get_mut(&mut store)
        .expect("exclusive")
        .load(index_dir)?;

    let mut registry = ChunkRegistry::default();
    registry.load(index_dir)?;

    let engine = SearchEngine::new(backends.embedder, store);
    let hits = engine.search(query, top_k).await?;

    if hits.is_empty() {
        println!("No results.");
        return Ok(());
    }

    println!("Results for: \"{query}\"\n");
    for (i, hit) in hits.iter().enumerate() {
        let chunk = registry.get(&hit.id);
        let name = chunk.and_then(|c| c.name.as_deref()).unwrap_or("?");
        let path = chunk
            .map(|c| c.path.display().to_string())
            .unwrap_or_default();
        let line = chunk.map(|c| c.span.start_line + 1).unwrap_or(0);
        let kind = chunk.map(|c| format!("{:?}", c.kind)).unwrap_or_default();

        println!(
            "{}. [{kind}] {name}  (score: {:.3})\n   {}:{}",
            i + 1,
            hit.score,
            path,
            line
        );

        if let Some(c) = chunk {
            let preview: String = c.content.lines().take(3).collect::<Vec<_>>().join("\n");
            println!("   {}\n", preview);
        }
    }

    Ok(())
}
