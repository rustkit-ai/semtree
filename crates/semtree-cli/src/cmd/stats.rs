use std::collections::HashMap;
use std::path::Path;

use anyhow::{Result, bail};
use semtree_core::Language;
use semtree_rag::ChunkRegistry;

pub fn run(index_dir: &Path) -> Result<()> {
    if !index_dir.exists() {
        bail!(
            "No index found at {}. Run `semtree index <path>` first.",
            index_dir.display()
        );
    }

    let mut registry = ChunkRegistry::default();
    registry.load(index_dir)?;

    let total = registry.len();
    if total == 0 {
        println!("Index at {} is empty.", index_dir.display());
        return Ok(());
    }

    let mut by_language: HashMap<Language, usize> = HashMap::new();
    let mut by_kind: HashMap<String, usize> = HashMap::new();
    let mut file_set = std::collections::HashSet::new();

    for chunk in registry.iter() {
        *by_language.entry(chunk.language).or_insert(0) += 1;
        *by_kind
            .entry(format!("{:?}", chunk.kind).to_lowercase())
            .or_insert(0) += 1;
        file_set.insert(chunk.path.clone());
    }

    let index_size = dir_size(index_dir);

    println!("=== Index: {} ===\n", index_dir.display());
    println!("  Chunks : {total}");
    println!("  Files  : {}", file_set.len());
    if let Some(size) = index_size {
        println!("  Size   : {}", human_bytes(size));
    }

    println!("\nBy language:");
    let mut langs: Vec<_> = by_language.iter().collect();
    langs.sort_by(|a, b| b.1.cmp(a.1));
    for (lang, count) in langs {
        let pct = *count * 100 / total;
        println!("  {:12} {:>5}  ({pct}%)", format!("{lang}:"), count);
    }

    println!("\nBy kind:");
    let mut kinds: Vec<_> = by_kind.iter().collect();
    kinds.sort_by(|a, b| b.1.cmp(a.1));
    for (kind, count) in kinds {
        let pct = *count * 100 / total;
        println!("  {:12} {:>5}  ({pct}%)", format!("{kind}:"), count);
    }

    Ok(())
}

fn dir_size(dir: &Path) -> Option<u64> {
    std::fs::read_dir(dir).ok().map(|entries| {
        entries
            .flatten()
            .filter_map(|e| e.metadata().ok().map(|m| m.len()))
            .sum()
    })
}

fn human_bytes(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{bytes} B")
    } else if bytes < 1024 * 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
