mod error;
mod indexer;
mod registry;
mod search;
mod context;

pub use error::RagError;
pub use indexer::Indexer;
pub use registry::ChunkRegistry;
pub use search::SearchEngine;
pub use context::{ContextBuilder, ContextWindow, ContextSnippet};
