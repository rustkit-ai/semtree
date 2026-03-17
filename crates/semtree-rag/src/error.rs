use thiserror::Error;

#[derive(Debug, Error)]
pub enum RagError {
    #[error("parse error: {0}")]
    Parse(#[from] semtree_parse::ParseError),
    #[error("embed error: {0}")]
    Embed(#[from] semtree_embed::EmbedError),
    #[error("store error: {0}")]
    Store(#[from] semtree_store::StoreError),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}
