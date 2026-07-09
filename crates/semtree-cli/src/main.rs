mod cmd;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "semtree", about = "Semantic code intelligence")]
struct Cli {
    /// Path to a .semtree.toml config file (default: .semtree.toml in current dir)
    #[arg(long, global = true)]
    config: Option<PathBuf>,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Create a .semtree.toml config file in the current directory
    Init {
        /// Directory to initialize (default: current dir)
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Index a directory and persist the vector store
    Index {
        /// Path to the codebase to index
        path: PathBuf,
        /// Where to store the index (default: from config or .semtree)
        #[arg(long)]
        index_dir: Option<PathBuf>,
        /// Force full re-index, ignoring the incremental manifest
        #[arg(long)]
        full: bool,
    },
    /// Search the index semantically
    Search {
        /// Natural language query
        query: String,
        /// Number of results to return
        #[arg(short, long, default_value_t = 5)]
        top_k: usize,
        /// Index directory
        #[arg(long)]
        index_dir: Option<PathBuf>,
        /// Filter by language (comma-separated): rust, python, javascript, typescript, go
        #[arg(long = "lang", value_delimiter = ',')]
        lang: Vec<String>,
        /// Filter by chunk kind (comma-separated): function, method, struct, enum, trait, impl, module, class, file
        #[arg(long = "kind", value_delimiter = ',')]
        kind: Vec<String>,
        /// Filter by path substring
        #[arg(long)]
        path: Option<String>,
        /// Matching strategy: hybrid (default), semantic, or lexical
        #[arg(long, default_value = "hybrid")]
        mode: String,
        /// Output results as JSON
        #[arg(long)]
        json: bool,
    },
    /// Build a RAG context prompt for a query
    Context {
        /// Natural language query
        query: String,
        /// Number of chunks to include
        #[arg(short, long, default_value_t = 5)]
        top_k: usize,
        /// Index directory
        #[arg(long)]
        index_dir: Option<PathBuf>,
        /// Output the full context window as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show statistics about the current index
    Stats {
        /// Index directory
        #[arg(long)]
        index_dir: Option<PathBuf>,
    },
    /// Analyze indexed code for complexity metrics
    Analyze {
        /// Index directory
        #[arg(long)]
        index_dir: Option<PathBuf>,
        /// Number of top results to show per section
        #[arg(short, long, default_value_t = 15)]
        top: usize,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("semtree=info".parse()?),
        )
        .init();

    let cli = Cli::parse();

    let cwd = std::env::current_dir()?;
    let config = if let Some(config_path) = cli.config {
        let raw = std::fs::read_to_string(&config_path)?;
        toml::from_str(&raw)?
    } else {
        semtree_core::SemtreeConfig::load(&cwd)
    };

    match cli.command {
        Command::Init { dir } => cmd::init::run(&dir),
        Command::Index {
            path,
            index_dir,
            full,
        } => {
            let index_dir = index_dir.unwrap_or_else(|| PathBuf::from(&config.index_dir));
            cmd::index::run(&path, &index_dir, &config, full).await
        }
        Command::Search {
            query,
            top_k,
            index_dir,
            lang,
            kind,
            path,
            mode,
            json,
        } => {
            let index_dir = index_dir.unwrap_or_else(|| PathBuf::from(&config.index_dir));
            let filters = cmd::search::SearchFilters {
                langs: lang,
                kinds: kind,
                path,
            };
            cmd::search::run(&query, top_k, &index_dir, &config, &filters, &mode, json).await
        }
        Command::Context {
            query,
            top_k,
            index_dir,
            json,
        } => {
            let index_dir = index_dir.unwrap_or_else(|| PathBuf::from(&config.index_dir));
            cmd::context::run(&query, top_k, &index_dir, &config, json).await
        }
        Command::Stats { index_dir } => {
            let index_dir = index_dir.unwrap_or_else(|| PathBuf::from(&config.index_dir));
            cmd::stats::run(&index_dir)
        }
        Command::Analyze { index_dir, top } => {
            let index_dir = index_dir.unwrap_or_else(|| PathBuf::from(&config.index_dir));
            cmd::analyze::run(&index_dir, top)
        }
    }
}
