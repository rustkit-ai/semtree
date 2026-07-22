//! **RAG pipeline for code: index, search, and inject codebase context into
//! LLM prompts - fully on-device, no daemon, no API key.**
//!
//! `semtree-rag` is the composable core behind the [`semtree`] CLI. It wires a
//! pluggable [`Embedder`](semtree_embed::Embedder) and
//! [`VectorStore`](semtree_store::VectorStore) into a full pipeline -
//! parse → chunk → embed → index → search - that you can drop into your own
//! tool, an MCP server, or an LLM context provider.
//!
//! # Pipeline at a glance
//!
//! | Stage | Building blocks |
//! |-------|-----------------|
//! | Index a directory | [`Indexer`] + [`ChunkRegistry`] |
//! | Search (vector / BM25 / fused) | [`HybridSearcher`] over [`SearchEngine`] + [`LexicalIndex`] |
//! | Build an LLM prompt | [`ContextBuilder`] → [`ContextWindow`] |
//! | Incremental re-index | [`FileManifest`] |
//!
//! # Example
//!
//! ```no_run
//! use std::sync::Arc;
//! use semtree_embed::fastembed::FastEmbedder;
//! use semtree_store::usearch::UsearchStore;
//! use semtree_rag::{ChunkRegistry, HybridSearcher, Indexer, LexicalIndex, SearchEngine, SearchMode};
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! // Backends are traits - swap FastEmbedder for OpenAI, UsearchStore for Qdrant.
//! let embedder = Arc::new(FastEmbedder::new()?);
//! let store = Arc::new(UsearchStore::new(384)?);
//!
//! // Index: parse -> chunk -> embed -> store. The registry keeps chunk metadata.
//! let mut registry = ChunkRegistry::default();
//! Indexer::new(embedder.clone(), store.clone())
//!     .index_dir(std::path::Path::new("./src"), &mut registry, None, |_, _| {})
//!     .await?;
//!
//! // Search: fuse vector similarity with BM25 keyword matching.
//! let engine = SearchEngine::new(embedder, store);
//! let lexical = LexicalIndex::from_chunks(registry.iter());
//! let hits = HybridSearcher::new(engine, lexical)
//!     .search("how is authentication handled", 5, SearchMode::Hybrid)
//!     .await?;
//! # Ok(())
//! # }
//! ```
//!
//! A complete, runnable version lives in [`examples/build_your_own.rs`].
//!
//! [`semtree`]: https://crates.io/crates/semtree
//! [`examples/build_your_own.rs`]: https://github.com/rustkit-ai/semtree/blob/main/crates/semtree-rag/examples/build_your_own.rs

mod context;
mod error;
mod hybrid;
mod indexer;
mod lexical;
mod manifest;
mod registry;
mod search;

pub use context::{ContextBuilder, ContextSnippet, ContextWindow};
pub use error::RagError;
pub use hybrid::{HybridSearcher, SearchMode};
pub use indexer::{Indexer, collect_indexable_files};
pub use lexical::LexicalIndex;
pub use manifest::FileManifest;
pub use registry::ChunkRegistry;
pub use search::SearchEngine;
