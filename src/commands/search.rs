use async_trait::async_trait;
use clap::Args;
use ansi_term::Style;

use crate::{context::Context, entity::types::file_embedding::FileEmbedding};

use super::Executor;

#[derive(Args, Debug)]
pub struct Search {
    #[arg(long)]
    query: String,
}

#[async_trait]
impl Executor for Search {
    async fn execute(&self) -> anyhow::Result<()> {
        let context = Context::default();
        let results = FileEmbedding::search(&context, &self.query).await?;

        for result in results {
            println!("{}", Style::new().bold().paint(result.file_path.0.display().to_string()));
            println!("{}", result.contents);
        }

        Ok(())
    }
}
