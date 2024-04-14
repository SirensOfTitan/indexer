use clap::{Parser, Subcommand};
use commands::{indexer, Executor};

mod commands;
mod services;

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
    let args = Cli::parse();
    match args.command {
        Commands::Indexer(indexer) => indexer.execute().await,
    }
}
