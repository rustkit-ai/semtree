//! A local semantic code-search MCP server, built from semtree's building blocks.
//!
//! It exposes one tool - `search_code` - that AI agents (Claude Code, Cursor,
//! Windsurf, Zed, ...) can call to find code by *meaning*, entirely on-device.
//! This is ~100 lines because semtree does the work; the MCP layer is just glue.
//!
//! # Usage
//!
//! First index a codebase with the `semtree` CLI (or the `build_your_own`
//! example), then point this server at the resulting index directory:
//!
//! ```sh
//! semtree index ./my-project           # writes ./.semtree
//! cargo run -p semtree-mcp-example -- ./.semtree
//! ```
//!
//! Register it with an MCP client (e.g. Claude Code's `mcp.json`):
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "semtree": {
//!       "command": "semtree-mcp-example",
//!       "args": ["/abs/path/to/.semtree"]
//!     }
//!   }
//! }
//! ```

use std::path::Path;
use std::sync::Arc;

use rmcp::{
    handler::server::wrapper::Parameters,
    model::{CallToolResult, ContentBlock, Implementation, ServerCapabilities, ServerInfo},
    tool, tool_handler, tool_router,
    transport::stdio,
    ServerHandler, ServiceExt,
};
use schemars::JsonSchema;
use semtree_embed::fastembed::FastEmbedder;
use semtree_rag::{ChunkRegistry, HybridSearcher, LexicalIndex, SearchEngine, SearchMode};
use semtree_store::{usearch::UsearchStore, VectorStore};
use serde::Deserialize;

// fastembed's default AllMiniLML6V2 model produces 384-dimensional vectors.
const EMBED_DIM: usize = 384;

#[derive(Deserialize, JsonSchema)]
struct SearchArgs {
    /// Natural-language description of the code you're looking for.
    query: String,
    /// Maximum number of results (default 5).
    #[serde(default)]
    top_k: Option<usize>,
}

/// The MCP server: holds a ready-to-query semtree index.
#[derive(Clone)]
struct SemtreeMcp {
    searcher: Arc<HybridSearcher>,
    registry: Arc<ChunkRegistry>,
}

#[tool_router]
impl SemtreeMcp {
    /// Load a persisted semtree index from `index_dir` into a live server.
    fn load(index_dir: &Path) -> anyhow::Result<Self> {
        let embedder = Arc::new(FastEmbedder::new()?);

        let mut store = UsearchStore::new(EMBED_DIM)?;
        store.load(index_dir)?;
        let store = Arc::new(store);

        let mut registry = ChunkRegistry::default();
        registry.load(index_dir)?;

        let engine = SearchEngine::new(embedder, store);
        let lexical = LexicalIndex::from_chunks(registry.iter());
        let searcher = Arc::new(HybridSearcher::new(engine, lexical));

        Ok(Self {
            searcher,
            registry: Arc::new(registry),
        })
    }

    #[tool(
        name = "search_code",
        description = "Search the indexed codebase semantically and return the most relevant \
                       functions, structs, and methods with their file locations."
    )]
    async fn search_code(
        &self,
        Parameters(SearchArgs { query, top_k }): Parameters<SearchArgs>,
    ) -> Result<CallToolResult, rmcp::ErrorData> {
        let top_k = top_k.unwrap_or(5);

        let hits = self
            .searcher
            .search(&query, top_k, SearchMode::Hybrid)
            .await
            .map_err(|e| rmcp::ErrorData::internal_error(e.to_string(), None))?;

        let mut out = String::new();
        for (i, hit) in hits.iter().enumerate() {
            if let Some(chunk) = self.registry.get(&hit.id) {
                let name = chunk.name.as_deref().unwrap_or("?");
                out.push_str(&format!(
                    "{}. [{:?}] {name}  (score {:.3})\n   {}:{}\n",
                    i + 1,
                    chunk.kind,
                    hit.score,
                    chunk.path.display(),
                    chunk.span.start_line + 1,
                ));
            }
        }
        if out.is_empty() {
            out.push_str("No results.");
        }

        Ok(CallToolResult::success(vec![ContentBlock::text(out)]))
    }
}

// `#[tool_handler]` generates `call_tool`/`list_tools` from the `#[tool]`
// methods above; we only override `get_info` to give the server a real identity
// and usage hint (the default reports rmcp's own crate name/version).
#[tool_handler]
impl ServerHandler for SemtreeMcp {
    fn get_info(&self) -> ServerInfo {
        // Both `Implementation` and `ServerInfo` are #[non_exhaustive], so we
        // start from a default and override the fields we care about.
        let mut server_info = Implementation::from_build_env();
        server_info.name = "semtree".into();
        server_info.version = env!("CARGO_PKG_VERSION").to_string();
        server_info.title = Some("semtree code search".into());
        server_info.description =
            Some("Local, on-device semantic search over an indexed codebase.".into());
        server_info.website_url = Some("https://github.com/rustkit-ai/semtree".into());

        let mut info = ServerInfo::default();
        info.capabilities = ServerCapabilities::builder().enable_tools().build();
        info.server_info = server_info;
        info.instructions = Some(
            "Call `search_code` with a natural-language query to find the most \
             relevant functions, structs, and methods in the indexed codebase."
                .into(),
        );
        info
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Logs go to stderr - stdout is reserved for the MCP JSON-RPC stream.
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .init();

    let index_dir = std::env::args()
        .nth(1)
        .unwrap_or_else(|| ".semtree".to_string());

    let server = SemtreeMcp::load(Path::new(&index_dir))?;

    // Serve the Model Context Protocol over stdio and block until the client
    // disconnects.
    let service = server.serve(stdio()).await?;
    service.waiting().await?;
    Ok(())
}
