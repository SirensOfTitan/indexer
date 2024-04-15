use clap::{Parser, Subcommand};
use commands::{indexer, Executor};
use platform::{init_db, init_project_dirs};

mod commands;
mod services;
mod entity;
mod context;
mod platform;

#[derive(Subcommand, Debug)]
enum Commands {
    Indexer(indexer::Indexer),
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
    }
}
