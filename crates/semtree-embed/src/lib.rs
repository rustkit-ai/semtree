mod embedder;
mod error;

#[cfg(feature = "fastembed-backend")]
pub mod fastembed;

#[cfg(feature = "openai-backend")]
pub mod openai;

#[cfg(feature = "ollama-backend")]
pub mod ollama;

pub use embedder::Embedder;
pub use error::EmbedError;

pub type Embedding = Vec<f32>;
