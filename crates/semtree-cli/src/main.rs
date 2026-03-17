mod cmd;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "semtree", about = "Semantic code intelligence")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Index a directory and persist the vector store
    Index {
        /// Path to the codebase to index
        path: PathBuf,
        /// Where to store the index (default: .semtree)
        #[arg(long, default_value = ".semtree")]
        index_dir: PathBuf,
    },
    /// Search the index semantically
    Search {
        /// Natural language query
        query: String,
        /// Number of results to return
        #[arg(short, long, default_value_t = 5)]
        top_k: usize,
        /// Index directory
        #[arg(long, default_value = ".semtree")]
        index_dir: PathBuf,
    },
    /// Build a RAG context prompt for a query
    Context {
        /// Natural language query
        query: String,
        /// Number of chunks to include
        #[arg(short, long, default_value_t = 5)]
        top_k: usize,
        /// Index directory
        #[arg(long, default_value = ".semtree")]
        index_dir: PathBuf,
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
    match cli.command {
        Command::Index { path, index_dir } => cmd::index::run(&path, &index_dir).await,
        Command::Search { query, top_k, index_dir } => cmd::search::run(&query, top_k, &index_dir).await,
        Command::Context { query, top_k, index_dir } => cmd::context::run(&query, top_k, &index_dir).await,
    }
}
