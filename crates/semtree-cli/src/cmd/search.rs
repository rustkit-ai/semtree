use std::path::Path;
use std::sync::Arc;

use anyhow::{bail, Result};
use semtree_core::{ChunkKind, Language, SemtreeConfig};
use semtree_rag::{ChunkRegistry, HybridSearcher, LexicalIndex, SearchEngine, SearchMode};
use serde::Serialize;

use super::make_backends;

/// Optional filters applied to search results after the vector lookup.
#[derive(Default)]
pub struct SearchFilters {
    pub langs: Vec<String>,
    pub kinds: Vec<String>,
    pub path: Option<String>,
}

impl SearchFilters {
    fn is_empty(&self) -> bool {
        self.langs.is_empty() && self.kinds.is_empty() && self.path.is_none()
    }
}

#[derive(Serialize)]
struct SearchResultJson {
    rank: usize,
    score: f32,
    name: Option<String>,
    kind: Option<String>,
    language: Option<String>,
    path: String,
    line: usize,
    content: String,
}

pub async fn run(
    query: &str,
    top_k: usize,
    index_dir: &Path,
    config: &SemtreeConfig,
    filters: &SearchFilters,
    mode: &str,
    json: bool,
) -> Result<()> {
    if !index_dir.exists() {
        bail!(
            "No index found at {}. Run `semtree index <path>` first.",
            index_dir.display()
        );
    }

    let mode = SearchMode::parse(mode).ok_or_else(|| {
        anyhow::anyhow!("unknown search mode '{mode}' (expected: hybrid, semantic, lexical)")
    })?;

    // Parse filter values up front so a typo fails fast.
    let langs = filters
        .langs
        .iter()
        .map(|s| parse_language(s))
        .collect::<Result<Vec<_>>>()?;
    let kinds = filters
        .kinds
        .iter()
        .map(|s| parse_kind(s))
        .collect::<Result<Vec<_>>>()?;
    let path_filter = filters.path.as_deref();

    let backends = make_backends(config)?;
    let mut store = backends.store;
    Arc::get_mut(&mut store)
        .expect("exclusive")
        .load(index_dir)?;

    let mut registry = ChunkRegistry::default();
    registry.load(index_dir)?;

    // When filtering, over-fetch candidates from the vector store so that
    // post-filter we still have enough to fill top_k.
    let fetch_k = if filters.is_empty() {
        top_k
    } else {
        (top_k * 10).max(50)
    };

    let engine = SearchEngine::new(backends.embedder, store);
    let lexical = LexicalIndex::from_chunks(registry.iter());
    let searcher = HybridSearcher::new(engine, lexical);
    let hits = searcher.search(query, fetch_k, mode).await?;

    let matches: Vec<_> = hits
        .iter()
        .filter(|hit| {
            let Some(chunk) = registry.get(&hit.id) else {
                // Keep unresolved hits only when no metadata filter is active.
                return filters.is_empty();
            };
            if !langs.is_empty() && !langs.contains(&chunk.language) {
                return false;
            }
            if !kinds.is_empty() && !kinds.contains(&chunk.kind) {
                return false;
            }
            if let Some(p) = path_filter
                && !chunk.path.display().to_string().contains(p)
            {
                return false;
            }
            true
        })
        .take(top_k)
        .collect();

    if json {
        let results: Vec<SearchResultJson> = matches
            .iter()
            .enumerate()
            .map(|(i, hit)| {
                let chunk = registry.get(&hit.id);
                SearchResultJson {
                    rank: i + 1,
                    score: hit.score,
                    name: chunk.and_then(|c| c.name.clone()),
                    kind: chunk.map(|c| format!("{:?}", c.kind).to_lowercase()),
                    language: chunk.map(|c| c.language.to_string()),
                    path: chunk
                        .map(|c| c.path.display().to_string())
                        .unwrap_or_default(),
                    line: chunk.map(|c| c.span.start_line + 1).unwrap_or(0),
                    content: chunk.map(|c| c.content.clone()).unwrap_or_default(),
                }
            })
            .collect();
        println!("{}", serde_json::to_string_pretty(&results)?);
        return Ok(());
    }

    if matches.is_empty() {
        println!("No results.");
        return Ok(());
    }

    println!("Results for: \"{query}\"\n");
    for (i, hit) in matches.iter().enumerate() {
        let chunk = registry.get(&hit.id);
        let name = chunk.and_then(|c| c.name.as_deref()).unwrap_or("?");
        let path = chunk
            .map(|c| c.path.display().to_string())
            .unwrap_or_default();
        let line = chunk.map(|c| c.span.start_line + 1).unwrap_or(0);
        let kind = chunk.map(|c| format!("{:?}", c.kind)).unwrap_or_default();

        println!(
            "{}. [{kind}] {name}  (score: {:.3})\n   {}:{}",
            i + 1,
            hit.score,
            path,
            line
        );

        if let Some(c) = chunk {
            let preview: String = c.content.lines().take(3).collect::<Vec<_>>().join("\n");
            println!("   {}\n", preview);
        }
    }

    Ok(())
}

fn parse_language(s: &str) -> Result<Language> {
    Ok(match s.to_lowercase().as_str() {
        "rust" | "rs" => Language::Rust,
        "python" | "py" => Language::Python,
        "javascript" | "js" => Language::JavaScript,
        "typescript" | "ts" => Language::TypeScript,
        "go" => Language::Go,
        other => bail!(
            "unknown language filter '{other}' (expected: rust, python, javascript, typescript, go)"
        ),
    })
}

fn parse_kind(s: &str) -> Result<ChunkKind> {
    Ok(match s.to_lowercase().as_str() {
        "function" | "fn" => ChunkKind::Function,
        "method" => ChunkKind::Method,
        "struct" => ChunkKind::Struct,
        "enum" => ChunkKind::Enum,
        "trait" => ChunkKind::Trait,
        "impl" => ChunkKind::Impl,
        "module" | "mod" => ChunkKind::Module,
        "class" => ChunkKind::Class,
        "file" => ChunkKind::File,
        other => bail!(
            "unknown kind filter '{other}' (expected: function, method, struct, enum, trait, impl, module, class, file)"
        ),
    })
}
