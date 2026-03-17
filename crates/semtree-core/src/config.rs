use serde::{Deserialize, Serialize};
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmbedBackend {
    Fastembed,
    OpenAI,
    Ollama,
}

impl Default for EmbedBackend {
    fn default() -> Self { Self::Fastembed }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StoreBackend {
    Usearch,
    Qdrant,
}

impl Default for StoreBackend {
    fn default() -> Self { Self::Usearch }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EmbedConfig {
    #[serde(default)]
    pub backend: EmbedBackend,
    /// Model name — defaults depend on backend
    pub model: Option<String>,
    /// Base URL — for Ollama
    pub url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct StoreConfig {
    #[serde(default)]
    pub backend: StoreBackend,
    /// Qdrant URL
    pub url: Option<String>,
    /// Qdrant collection name
    pub collection: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SemtreeConfig {
    #[serde(default)]
    pub embed: EmbedConfig,
    #[serde(default)]
    pub store: StoreConfig,
    /// Where to store the local index
    #[serde(default = "default_index_dir")]
    pub index_dir: String,
}

fn default_index_dir() -> String {
    ".semtree".to_string()
}

impl SemtreeConfig {
    /// Load from `.semtree.toml` if present, otherwise use defaults.
    pub fn load(dir: &Path) -> Self {
        let path = dir.join(".semtree.toml");
        if let Ok(raw) = std::fs::read_to_string(&path) {
            toml::from_str(&raw).unwrap_or_else(|e| {
                eprintln!("Warning: invalid .semtree.toml: {e}");
                Self::default()
            })
        } else {
            Self::default()
        }
    }

    pub fn save(&self, dir: &Path) -> std::io::Result<()> {
        let path = dir.join(".semtree.toml");
        let raw = toml::to_string_pretty(self).expect("serializable");
        std::fs::write(path, raw)
    }
}
