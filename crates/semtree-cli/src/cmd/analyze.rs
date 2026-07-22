use std::path::Path;

use anyhow::{Result, bail};
use semtree_analyze::analyze_chunks;
use semtree_rag::ChunkRegistry;

pub fn run(index_dir: &Path, top: usize) -> Result<()> {
    if !index_dir.exists() {
        bail!(
            "No index found at {}. Run `semtree index <path>` first.",
            index_dir.display()
        );
    }

    let mut registry = ChunkRegistry::default();
    registry.load(index_dir)?;

    if registry.is_empty() {
        println!("Index at {} is empty.", index_dir.display());
        return Ok(());
    }

    let all_chunks: Vec<_> = registry.iter().cloned().collect();
    let reports = analyze_chunks(&all_chunks);

    println!("=== Complexity analysis: {} ===\n", index_dir.display());

    println!("Top {top} by cyclomatic complexity (approximate):");
    println!("  {:<40} {:>6}  {:>5}  location", "name", "cc", "lines");
    println!("  {}", "-".repeat(80));
    for r in reports.iter().take(top) {
        let loc = format!("{}:{}", r.path.display(), r.start_line);
        println!(
            "  {:<40} {:>6}  {:>5}  {}",
            truncate(&r.name, 40),
            r.cyclomatic,
            r.line_count,
            loc
        );
    }

    println!("\nTop {top} by line count:");
    println!("  {:<40} {:>5}  location", "name", "lines");
    println!("  {}", "-".repeat(70));
    let mut by_lines = reports.clone();
    by_lines.sort_by_key(|r| std::cmp::Reverse(r.line_count));
    for r in by_lines.iter().take(top) {
        let loc = format!("{}:{}", r.path.display(), r.start_line);
        println!(
            "  {:<40} {:>5}  {}",
            truncate(&r.name, 40),
            r.line_count,
            loc
        );
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
