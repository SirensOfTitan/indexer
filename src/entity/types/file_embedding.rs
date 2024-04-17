use std::path::PathBuf;

use anyhow::Context;
use async_trait::async_trait;
use sea_query::{Asterisk, Expr, Iden, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use serde_json::json;
use typed_builder::TypedBuilder;

use crate::{
    context,
    entity::{columns::FilePath, Entity},
    services::embeddings::EmbeddingsService,
};

#[derive(Iden)]
pub enum FileEmbeddingTable {
    #[iden = "file_embeddings"]
    Table,
    FilePath,
    Embedding,
}

#[derive(sqlx::FromRow, Debug)]
pub struct FileEmbedding {
    pub file_path: FilePath,
    pub embedding: sqlx::types::Json<Vec<f32>>,
}

#[async_trait]
impl Entity for FileEmbedding {
    type ID = FilePath;

    fn get_id(&self) -> Self::ID {
        self.file_path.clone()
    }

    fn name() -> &'static str {
        "file"
    }

    async fn find_many(
        context: &context::Context,
        ids: &[Self::ID],
    ) -> Result<Vec<Self>, sqlx::Error> {
        let (sql, values) = Query::select()
            .column(Asterisk)
            .from(FileEmbeddingTable::Table)
            .and_where(Expr::col(FileEmbeddingTable::FilePath).is_in(ids))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with(&sql, values)
            .fetch_all(&context.db)
            .await
    }
}

#[derive(TypedBuilder)]
pub struct CreateFileEmbeddingProps {
    pub file_path: PathBuf,
    pub embedding: Vec<f32>,
}

impl FileEmbedding {
    pub async fn create_many(
        context: &context::Context,
        files: Vec<CreateFileEmbeddingProps>,
    ) -> Result<(), sqlx::Error> {
        let mut builder = Query::insert();

        builder
            .into_table(FileEmbeddingTable::Table)
            .columns([FileEmbeddingTable::FilePath, FileEmbeddingTable::Embedding]);

        for file in files {
            builder.values_panic([
                FilePath::new(file.file_path).into(),
                json!(file.embedding).into(),
            ]);
        }

        let (query, values) = builder.build_sqlx(SqliteQueryBuilder);

        let _ = sqlx::query_with(&query, values)
            .execute(&context.db)
            .await?;
        Ok(())
    }

    pub async fn search(
        context: &context::Context,
        query: &str,
    ) -> anyhow::Result<Vec<FileEmbedding>> {
        let embedding_svc = EmbeddingsService::try_new()?;
        let embedded_query = json!(embedding_svc.embedding(query).await?);

        sqlx::query_as(&format!(
            r#"SELECT f.file_path, f.embedding
                FROM file_embeddings f
                INNER JOIN vss_file_embeddings v ON (v.rowid = f.rowid)
                WHERE vss_search(
                    v.embedding,
                    vss_search_params('{embedded_query}', 3)
                )
                LIMIT 3"#
        ))
        .fetch_all(&context.db)
        .await
        .context("Query failed")
    }
}
