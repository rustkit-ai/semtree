use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Result};
use semtree_core::SemtreeConfig;
use semtree_rag::{ChunkRegistry, ContextBuilder, SearchEngine};

use super::make_backends;

pub async fn run(query: &str, top_k: usize, index_dir: &Path, config: &SemtreeConfig) -> Result<()> {
    if !index_dir.exists() {
        bail!("No index found at {}. Run `semtree index <path>` first.", index_dir.display());
    }

    let backends = make_backends(config)?;
    let mut store = backends.store;
    Arc::get_mut(&mut store)
        .expect("exclusive")
        .load(index_dir)?;

    let mut registry = ChunkRegistry::default();
    registry.load(index_dir)?;

    let engine = Arc::new(SearchEngine::new(backends.embedder, store));
    let builder = ContextBuilder::new(engine).with_max_chunks(top_k);
    let window = builder.build(query).await?;

    println!("=== Context for: \"{}\" ===\n", query);
    for (i, snippet) in window.snippets.iter().enumerate() {
        let chunk = registry.get(&snippet.chunk_id);
        let name = chunk.and_then(|c| c.name.as_deref()).unwrap_or("?");
        let path = chunk.map(|c| c.path.display().to_string()).unwrap_or_default();
        let line = chunk.map(|c| c.span.start_line + 1).unwrap_or(0);

        println!("[{}] {} — {}:{} (score: {:.3})", i + 1, name, path, line, snippet.score);

        if let Some(c) = chunk {
            println!("```\n{}\n```\n", c.content);
        }
    }

    println!("=== Prompt ===\n{}", window.prompt);
    Ok(())
}
