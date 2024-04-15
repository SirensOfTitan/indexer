use async_trait::async_trait;

use crate::context;

pub mod types;
pub mod columns;

#[async_trait]
pub trait Entity {
    type ID: Sized;
    /// A globally unique identifier for this entity.  Global meaning: unique
    /// throughout this whole application.
    fn get_id(&self) -> Self::ID;

    /// A globally unique name for the entity, should essentially match the
    /// lower snake_case equivalent to the UpperCamelCase name of the entity.
    ///
    /// TODO: This should likely be generated via macro.
    fn name() -> &'static str;

    /// Given a list of IDs, returns a vec of matches.
    async fn find_many(
        vc: &context::Context,
        ids: &[Self::ID],
    ) -> Result<Vec<Self>, sqlx::Error>
    where
        Self: Sized;

    async fn find_one(vc: &context::Context, id: Self::ID) -> Result<Option<Self>, sqlx::Error>
    where
        Self: Sized,
        <Self as Entity>::ID: Send + Sync,
    {
        let mut results = Self::find_many(vc, &[id]).await?;
        if results.is_empty() {
            Ok(None)
        } else {
            Ok(Some(results.remove(0)))
        }
    }
}
