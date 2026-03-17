mod error;
mod embedder;

#[cfg(feature = "fastembed-backend")]
pub mod fastembed;

pub use embedder::Embedder;
pub use error::EmbedError;

pub type Embedding = Vec<f32>;
