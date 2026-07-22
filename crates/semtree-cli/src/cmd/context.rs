use std::path::Path;
use std::sync::Arc;

use anyhow::{Result, bail};
use semtree_core::SemtreeConfig;
use semtree_rag::{ChunkRegistry, ContextBuilder, SearchEngine};

use super::make_backends;

pub async fn run(
    query: &str,
    top_k: usize,
    index_dir: &Path,
    config: &SemtreeConfig,
    json: bool,
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

    let engine = Arc::new(SearchEngine::new(backends.embedder, store));
    let builder = ContextBuilder::new(engine).with_max_chunks(top_k);
    let window = builder.build(query, &registry).await?;

    if json {
        println!("{}", serde_json::to_string_pretty(&window)?);
        return Ok(());
    }

    println!("=== Context for: \"{}\" ===\n", query);
    for (i, snippet) in window.snippets.iter().enumerate() {
        let name = snippet.name.as_deref().unwrap_or("?");
        println!(
            "[{}] {} - {}:{} (score: {:.3})",
            i + 1,
            name,
            snippet.path,
            snippet.start_line,
            snippet.score
        );
        println!("```\n{}\n```\n", snippet.content);
    }

    println!("=== Prompt ===\n{}", window.prompt);
    Ok(())
}
