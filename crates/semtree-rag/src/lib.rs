mod context;
mod error;
mod indexer;
mod manifest;
mod registry;
mod search;

pub use context::{ContextBuilder, ContextSnippet, ContextWindow};
pub use error::RagError;
pub use indexer::{collect_indexable_files, Indexer};
pub use manifest::FileManifest;
pub use registry::ChunkRegistry;
pub use search::SearchEngine;
