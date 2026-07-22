//! Assemble your own semantic-search pipeline from semtree's building blocks -
//! no CLI, no config files. This is the core that the `semtree` CLI (and any
//! MCP server you build on top) wraps.
//!
//! Run it against any directory:
//!
//! ```sh
//! cargo run --example build_your_own -- ./src "how are errors handled"
//! ```

use std::sync::Arc;

use semtree_embed::Embedder;
use semtree_embed::fastembed::FastEmbedder;
use semtree_rag::{ChunkRegistry, HybridSearcher, Indexer, LexicalIndex, SearchEngine, SearchMode};
use semtree_store::usearch::UsearchStore;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "./src".to_string());
    let query = std::env::args()
        .nth(2)
        .unwrap_or_else(|| "error handling".to_string());

    // 1. Pick your backends. Both are traits - swap FastEmbedder for OpenAI or
    //    Ollama, or UsearchStore for Qdrant, without touching the pipeline below.
    let embedder = Arc::new(FastEmbedder::new()?);
    let store = Arc::new(UsearchStore::new(embedder.dimension())?);

    // 2. Index the directory: parse -> chunk -> embed -> store. The registry
    //    holds chunk metadata (path, span, kind) keyed by id.
    let mut registry = ChunkRegistry::default();
    let n = Indexer::new(embedder.clone(), store.clone())
        .index_dir(
            std::path::Path::new(&dir),
            &mut registry,
            None,
            |done, total| eprint!("\rindexing {done}/{total} files"),
        )
        .await?;
    eprintln!("\nindexed {n} chunks from {dir}\n");

    // 3. Search. HybridSearcher fuses vector similarity (SearchEngine) with
    //    BM25 keyword matching (LexicalIndex) - meaning *and* exact identifiers.
    let engine = SearchEngine::new(embedder, store);
    let lexical = LexicalIndex::from_chunks(registry.iter());
    let searcher = HybridSearcher::new(engine, lexical);

    let hits = searcher.search(&query, 5, SearchMode::Hybrid).await?;

    println!("Top results for {query:?}:");
    for (i, hit) in hits.iter().enumerate() {
        if let Some(chunk) = registry.get(&hit.id) {
            let name = chunk.name.as_deref().unwrap_or("?");
            println!(
                "{}. [{:?}] {name}  (score {:.3})\n   {}:{}",
                i + 1,
                chunk.kind,
                hit.score,
                chunk.path.display(),
                chunk.span.start_line + 1,
            );
        }
    }

    Ok(())
}
