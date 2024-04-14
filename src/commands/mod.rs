use async_trait::async_trait;

pub mod indexer;

#[async_trait]
pub trait Executor {
    async fn execute(&self) -> anyhow::Result<()>;
}
