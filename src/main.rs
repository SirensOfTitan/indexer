use clap::{Parser, Subcommand};
use commands::{ai, embeddings, indexer, search, Executor};
use platform::{init_db, init_project_dirs};

mod commands;
mod context;
mod entity;
mod platform;
mod services;
mod config;

#[derive(Subcommand, Debug)]
enum Commands {
    Indexer(indexer::Indexer),
    Embeddings(embeddings::Embeddings),
    Search(search::Search),
    AI(ai::AI),
}

#[derive(Parser, Debug)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_project_dirs().await?;
    init_db().await?;

    let args = Cli::parse();
    match args.command {
        Commands::Indexer(indexer) => indexer.execute().await,
        Commands::Embeddings(embeddings) => embeddings.execute().await,
        Commands::Search(search) => search.execute().await,
        Commands::AI(ai) => ai.execute().await,
    }
}
