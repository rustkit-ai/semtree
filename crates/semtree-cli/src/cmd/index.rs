use std::path::Path;

use anyhow::Result;
use indicatif::{ProgressBar, ProgressStyle};
use semtree_core::SemtreeConfig;
use semtree_rag::{ChunkRegistry, FileManifest, Indexer};

use super::make_backends;

pub async fn run(path: &Path, index_dir: &Path, config: &SemtreeConfig, full: bool) -> Result<()> {
    std::fs::create_dir_all(index_dir)?;

    let backends = make_backends(config)?;
    let store = backends.store;
    let fingerprint = backends.embedder.fingerprint();
    let store_fingerprint = store.metric().to_string();
    let mut registry = ChunkRegistry::default();

    // Fall back to a full rebuild when the existing index was built with a
    // different embedder, store, or an older chunker: reusing it would mix
    // incompatible vectors, re-rank results, or leave stale chunk IDs behind.
    let existing = (!full).then(|| FileManifest::load(index_dir));
    let full = match &existing {
        Some(m) if !m.is_compatible_with(&fingerprint, &store_fingerprint) => {
            eprintln!(
                "Index was built with a different embedder, store, or chunker ({}/{} → {}/{}); rebuilding from scratch.",
                m.embedder(),
                m.store(),
                fingerprint,
                store_fingerprint,
            );
            true
        }
        None => true,
        _ => false,
    };

    let mut manifest = if full {
        FileManifest::new(&fingerprint, &store_fingerprint)
    } else {
        existing.unwrap_or_else(|| FileManifest::new(&fingerprint, &store_fingerprint))
    };

    // Pre-load existing index if doing incremental update
    if !full && index_dir.join("index.usearch").exists() {
        std::sync::Arc::get_mut(&mut store.clone()).map(|s| s.load(index_dir).ok());
        registry.load(index_dir).ok();
    }

    let file_count = semtree_rag::collect_indexable_files(path).len();

    let bar = ProgressBar::new(file_count as u64);
    bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.cyan} [{bar:40.cyan/blue}] {pos}/{len} files  ({elapsed})",
        )
        .unwrap()
        .progress_chars("=>-"),
    );

    let indexer = Indexer::new(backends.embedder, store.clone());

    let n = indexer
        .index_dir(path, &mut registry, Some(&mut manifest), |done, _total| {
            bar.set_position(done as u64);
        })
        .await?;

    bar.finish_and_clear();

    store.save(index_dir)?;
    registry.save(index_dir)?;
    manifest.save(index_dir)?;

    let mode = if full { "full" } else { "incremental" };
    println!(
        "Done ({mode}). Indexed {n} chunks → {}",
        index_dir.display()
    );
    Ok(())
}
