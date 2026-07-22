use async_trait::async_trait;

use crate::{EmbedError, Embedding};

#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, texts: &[&str]) -> Result<Vec<Embedding>, EmbedError>;

    async fn embed_one(&self, text: &str) -> Result<Embedding, EmbedError> {
        self.embed(&[text]).await.map(|mut v| v.remove(0))
    }

    /// Largest number of texts a single [`embed`](Embedder::embed) call should
    /// receive. Callers indexing many chunks split their work into groups of at
    /// most this size, bounding peak memory (local models) and request size
    /// (remote APIs). The default suits the on-device models; raise or lower it
    /// to match a backend's limits.
    fn max_batch_size(&self) -> usize {
        256
    }

    /// Number of dimensions every vector this embedder produces has.
    ///
    /// A store built for one dimension cannot hold vectors of another, so an
    /// index persists this and refuses to mix them.
    fn dimension(&self) -> usize;

    /// Stable identifier for the model behind this embedder, e.g.
    /// `"fastembed:AllMiniLML6V2"` or `"openai:text-embedding-3-small"`.
    ///
    /// It must stay the same across runs and versions for the *same* model, and
    /// differ whenever the produced vectors would be incompatible. It is the
    /// discriminating half of [`fingerprint`](Embedder::fingerprint).
    fn model_id(&self) -> &str;

    /// Fingerprint an index stores to detect that it was built with a different
    /// embedder. Re-indexing is required whenever this changes.
    fn fingerprint(&self) -> String {
        format!("{}/{}d", self.model_id(), self.dimension())
    }
}
