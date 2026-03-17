use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("store init failed: {0}")]
    Init(String),
    #[error("insert failed: {0}")]
    Insert(String),
    #[error("search failed: {0}")]
    Search(String),
}
