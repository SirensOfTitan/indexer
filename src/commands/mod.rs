use async_trait::async_trait;

pub mod embeddings;
pub mod indexer;
pub mod search;

#[async_trait]
pub trait Executor {
    async fn execute(&self) -> anyhow::Result<()>;
}
