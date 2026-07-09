//! **Core data types shared across the semtree crates.**
//!
//! No logic, no I/O - just the vocabulary every other crate speaks:
//! [`Chunk`] and [`ChunkKind`] (a unit of indexed code), [`Language`],
//! [`Span`] (byte + line range), and the [`SemtreeConfig`] tree.

pub mod chunk;
pub mod config;
pub mod language;
pub mod span;

pub use chunk::{Chunk, ChunkKind};
pub use config::{EmbedBackend, EmbedConfig, SemtreeConfig, StoreBackend, StoreConfig};
pub use language::Language;
pub use span::Span;
