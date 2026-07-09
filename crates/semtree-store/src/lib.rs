//! **Vector-store abstraction for semtree.**
//!
//! One trait - [`VectorStore`] - with swappable backends behind feature flags:
//!
//! | Backend | Feature | Type |
//! |---------|---------|------|
//! | usearch (local HNSW, default) | `usearch-backend` | [`usearch::UsearchStore`] |
//! | Qdrant | `qdrant-backend` | `qdrant::QdrantStore` |
//!
//! [`search`](VectorStore::search) returns ranked [`Hit`]s (id + score); the
//! caller resolves ids back to chunks via the registry in `semtree-rag`.
//! Implement [`VectorStore`] to target any other index or database.

mod error;
mod store;

#[cfg(feature = "usearch-backend")]
pub mod usearch;

#[cfg(feature = "qdrant-backend")]
pub mod qdrant;

pub use error::StoreError;
pub use store::{Hit, VectorStore};
