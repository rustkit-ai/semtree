use anyhow::Result;
use semtree_core::SemtreeConfig;
use std::path::Path;

pub fn run(dir: &Path) -> Result<()> {
    let config_path = dir.join(".semtree.toml");

    if config_path.exists() {
        println!(".semtree.toml already exists at {}", config_path.display());
        return Ok(());
    }

    SemtreeConfig::default().save(dir)?;
    println!("Created {}", config_path.display());
    println!();
    println!("Edit .semtree.toml to configure your embed backend and vector store.");
    println!("Credentials go in environment variables:");
    println!("  OPENAI_API_KEY   — for openai embed backend");
    println!("  QDRANT_URL       — for qdrant store backend");

    Ok(())
}
