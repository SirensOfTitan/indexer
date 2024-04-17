use std::path::PathBuf;

use async_trait::async_trait;
use clap::{Args, Parser, Subcommand};

use crate::services::embeddings::EmbeddingsService;

use super::Executor;

#[derive(Args, Debug)]
struct ParseFileArgs {
    #[arg(long)]
    /// The path to index.
    file: PathBuf,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Parses a file into tokens for embedding.
    Parse(ParseFileArgs),
}

#[derive(Parser, Debug)]
pub struct Embeddings {
    #[command(subcommand)]
    command: Commands,
}

#[async_trait]
impl Executor for Embeddings {
    async fn execute(&self) -> anyhow::Result<()> {
        let svc = EmbeddingsService::try_new()?;
        match self.command {
            Commands::Parse(ref args) => {
                let tokens = svc.parse_file(&args.file).await?;

                for token in tokens {
                    println!("{:?}", token);
                }

                Ok(())
            }
        }
    }
}
