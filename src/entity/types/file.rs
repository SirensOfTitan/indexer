use std::path::PathBuf;

use async_trait::async_trait;
use sea_query::{Asterisk, Expr, Iden, OnConflict, Query, SqliteQueryBuilder};
use sea_query_binder::SqlxBinder;
use typed_builder::TypedBuilder;

use crate::{
    context,
    entity::{columns::FilePath, Entity},
};

#[derive(Iden)]
pub enum Excluded {
    Table,
}

#[derive(Iden)]
pub enum FileTable {
    #[iden = "file"]
    Table,
    Path,
    Hash,
}

#[derive(sqlx::FromRow, Debug)]
pub struct File {
    /// The path where the file is at.  This is currently defined as a string,
    /// and it's unclear if this is a viable path because OsStr can differ
    /// depending on platform.
    pub path: FilePath,

    /// The recorded hash of the file
    pub hash: Vec<u8>,
}

#[async_trait]
impl Entity for File {
    type ID = FilePath;

    fn get_id(&self) -> Self::ID {
        self.path.clone()
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
            .from(FileTable::Table)
            .and_where(Expr::col(FileTable::Path).is_in(ids))
            .build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with(&sql, values)
            .fetch_all(&context.db)
            .await
    }
}

#[derive(TypedBuilder)]
pub struct CreateFileProps {
    pub path: PathBuf,
    pub hash: Vec<u8>,
}

impl File {
    pub async fn create_many(
        context: &context::Context,
        files: Vec<CreateFileProps>,
    ) -> Result<Vec<Self>, sqlx::Error> {
        if files.is_empty() {
            return Ok(vec![]);
        }

        let mut builder = Query::insert();

        builder
            .into_table(FileTable::Table)
            .on_conflict(
                OnConflict::column(FileTable::Path)
                    .update_columns([FileTable::Hash])
                    .action_and_where(
                        Expr::col((Excluded::Table, FileTable::Hash))
                            .ne(Expr::col((FileTable::Table, FileTable::Hash))),
                    )
                    .to_owned(),
            )
            .columns([FileTable::Path, FileTable::Hash])
            .returning_all();

        for file in files {
            builder.values_panic([FilePath::new(file.path).into(), file.hash.into()]);
        }

        let (query, values) = builder.build_sqlx(SqliteQueryBuilder);

        sqlx::query_as_with(&query, values)
            .fetch_all(&context.db)
            .await
    }
}
