mod error;
mod store;

#[cfg(feature = "usearch-backend")]
pub mod usearch;

pub use error::StoreError;
pub use store::{Hit, VectorStore};
