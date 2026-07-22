//! **Text-embedding abstraction for semtree.**
//!
//! One trait - [`Embedder`] - with swappable backends behind feature flags, so
//! the rest of the pipeline never hard-codes an embedding provider:
//!
//! | Backend | Feature | Type |
//! |---------|---------|------|
//! | fastembed (local ONNX, default) | `fastembed-backend` | [`fastembed::FastEmbedder`] |
//! | OpenAI | `openai-backend` | `openai::OpenAIEmbedder` |
//! | Ollama | `ollama-backend` | `ollama::OllamaEmbedder` |
//!
//! Implement [`Embedder`] to plug in any model:
//!
//! ```
//! use async_trait::async_trait;
//! use semtree_embed::{Embedder, Embedding, EmbedError};
//!
//! struct Zeros;
//!
//! #[async_trait]
//! impl Embedder for Zeros {
//!     async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError> {
//!         Ok(texts.iter().map(|_| vec![0.0; 384]).collect())
//!     }
//!     fn dimension(&self) -> usize { 384 }
//!     fn model_id(&self) -> &str { "zeros" }
//! }
//! ```

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
