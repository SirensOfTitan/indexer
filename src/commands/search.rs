use async_trait::async_trait;
use clap::Args;

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
            println!("{}", result.file_path.0.display());
        }

        Ok(())
    }
}
