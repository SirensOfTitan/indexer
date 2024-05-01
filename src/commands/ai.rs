use std::io::Write;

use async_trait::async_trait;
use clap::{Parser, Subcommand};
use futures::StreamExt;

use crate::services::ai::AIService;

use super::Executor;

#[derive(Subcommand, Debug)]
enum Commands {
    Start,
}

#[derive(Parser, Debug)]
pub struct AI {
    #[command(subcommand)]
    command: Commands,
}

#[async_trait]
impl Executor for AI {
    async fn execute(&self) -> anyhow::Result<()> {
        match self.command {
            Commands::Start => {
                let mut ai_svc = AIService::try_new().await?;

                let mut stream = ai_svc.infer("Who was president of the US in 1978?")?;

                while let Some(Ok(item)) = stream.next().await {
                    print!("{item}");
                    std::io::stdout().flush()?;
                }

                Ok(())
            }
        }
    }
}
