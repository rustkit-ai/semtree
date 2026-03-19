use std::sync::Arc;

use semtree_embed::fastembed::FastEmbedder;
use semtree_rag::{ChunkRegistry, Indexer, SearchEngine};
use semtree_store::usearch::UsearchStore;

const EMBED_DIM: usize = 384;

/// Write a temporary Rust source file and return its path.
fn write_tmp_rs(name: &str, content: &str) -> std::path::PathBuf {
    let mut path = std::env::temp_dir();
    path.push(format!("semtree_rag_test_{name}.rs"));
    std::fs::write(&path, content).expect("write temp file");
    path
}

#[tokio::test]
#[ignore = "requires fastembed model download (~23 MB) — run with --ignored locally"]
async fn test_index_search_and_registry() {
    // ── setup backends ────────────────────────────────────────────────────────
    let embedder = Arc::new(FastEmbedder::new().expect("FastEmbedder"));
    let store = Arc::new(UsearchStore::new(EMBED_DIM).expect("UsearchStore"));

    // ── write three small Rust snippets ───────────────────────────────────────
    let file_a = write_tmp_rs(
        "alpha",
        r#"
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
"#,
    );

    let file_b = write_tmp_rs(
        "beta",
        r#"
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl Point {
    pub fn distance(&self, other: &Point) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }
}
"#,
    );

    let file_c = write_tmp_rs(
        "gamma",
        r#"
pub fn multiply(a: i32, b: i32) -> i32 {
    a * b
}

pub fn subtract(a: i32, b: i32) -> i32 {
    a - b
}
"#,
    );

    // ── index ─────────────────────────────────────────────────────────────────
    let indexer = Indexer::new(embedder.clone(), store.clone());
    let mut registry = ChunkRegistry::default();

    let n_a = indexer
        .index_file(&file_a, &mut registry)
        .await
        .expect("index file_a");
    let n_b = indexer
        .index_file(&file_b, &mut registry)
        .await
        .expect("index file_b");
    let n_c = indexer
        .index_file(&file_c, &mut registry)
        .await
        .expect("index file_c");

    let total = n_a + n_b + n_c;
    assert!(total > 0, "should have indexed at least one chunk");

    // ── registry ──────────────────────────────────────────────────────────────
    assert_eq!(
        registry.len(),
        total,
        "registry should hold all indexed chunks"
    );
    assert!(!registry.is_empty(), "registry should not be empty");

    // ── search ────────────────────────────────────────────────────────────────
    let engine = SearchEngine::new(embedder, store);
    let hits = engine
        .search("arithmetic operations", 3)
        .await
        .expect("search");

    assert!(!hits.is_empty(), "should return at least one hit");

    // Every hit id should be findable in the registry
    for hit in &hits {
        let chunk = registry.get(&hit.id);
        assert!(chunk.is_some(), "hit id {} should be in registry", hit.id);
    }

    // ── cleanup ───────────────────────────────────────────────────────────────
    let _ = std::fs::remove_file(file_a);
    let _ = std::fs::remove_file(file_b);
    let _ = std::fs::remove_file(file_c);
}
