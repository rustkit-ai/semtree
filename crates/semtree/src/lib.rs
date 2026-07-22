//! **semtree - local semantic code search, batteries included.**
//!
//! semtree is a toolbox of small crates ([`parse`], [`embed`], [`store`],
//! [`rag`], ...) you can assemble however you like. This crate is the assembled
//! default: it pulls in the on-device stack - tree-sitter chunking, fastembed
//! (local ONNX) embeddings, usearch (HNSW) storage, hybrid BM25 + vector search -
//! so the common case is a handful of lines instead of wiring four crates by hand.
//!
//! Reach for the individual crates when you want to swap a backend or trim the
//! dependency tree; reach for this one when you just want search to work.
//!
//! ```no_run
//! use semtree::prelude::*;
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! // The default on-device stack, sized and wired so it agrees with itself.
//! let (embedder, store) = semtree::default_backends()?;
//!
//! let mut registry = ChunkRegistry::default();
//! Indexer::new(embedder.clone(), store.clone())
//!     .index_dir(std::path::Path::new("./src"), &mut registry, None, |_, _| {})
//!     .await?;
//!
//! let engine = SearchEngine::new(embedder, store);
//! let lexical = LexicalIndex::from_chunks(registry.iter());
//! let hits = HybridSearcher::new(engine, lexical)
//!     .search("where do we skip unchanged files", 5, SearchMode::Hybrid)
//!     .await?;
//! # Ok(())
//! # }
//! ```

// Re-export every layer under a short path, so `semtree::rag::Indexer` and
// friends are reachable without adding each crate to your own Cargo.toml.
pub use semtree_analyze as analyze;
pub use semtree_core as core;
pub use semtree_embed as embed;
pub use semtree_parse as parse;
pub use semtree_rag as rag;
pub use semtree_store as store;

/// Everything you need to index and search, in one glob import.
pub mod prelude {
    pub use semtree_core::{Chunk, ChunkKind, Language, Span};
    pub use semtree_embed::{Embedder, Embedding};
    pub use semtree_rag::{
        ChunkRegistry, ContextBuilder, FileManifest, HybridSearcher, Indexer, LexicalIndex,
        SearchEngine, SearchMode,
    };
    pub use semtree_store::{Hit, Metric, VectorStore};
}

/// A shared, backend-agnostic embedder handle.
pub type SharedEmbedder = std::sync::Arc<dyn semtree_embed::Embedder>;
/// A shared, backend-agnostic vector store handle.
pub type SharedStore = std::sync::Arc<dyn semtree_store::VectorStore>;

/// Errors from assembling or driving the default stack.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Embed(#[from] semtree_embed::EmbedError),
    #[error(transparent)]
    Store(#[from] semtree_store::StoreError),
}

/// Build the default on-device backends: [`FastEmbedder`](semtree_embed::fastembed::FastEmbedder)
/// paired with a [`UsearchStore`](semtree_store::usearch::UsearchStore) sized to
/// the embedder's own dimension, so the two can never disagree on vector width.
///
/// Returns trait objects, so you can hand them straight to [`rag::Indexer`] and
/// [`rag::SearchEngine`] and later swap in any other [`Embedder`]/[`VectorStore`].
#[cfg(all(feature = "fastembed", feature = "usearch"))]
pub fn default_backends() -> Result<(SharedEmbedder, SharedStore), Error> {
    use semtree_embed::Embedder;

    let embedder = semtree_embed::fastembed::FastEmbedder::new()?;
    let store = semtree_store::usearch::UsearchStore::new(embedder.dimension())?;
    Ok((std::sync::Arc::new(embedder), std::sync::Arc::new(store)))
}
