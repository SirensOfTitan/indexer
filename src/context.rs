use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::platform::get_db_path;

#[derive(Clone)]
pub struct Context {
    pub db: SqlitePool,
}

impl Default for Context {
    fn default() -> Self {
        let db_path = get_db_path();

        let db = SqlitePoolOptions::new()
            .connect_lazy(&db_path)
            .expect("Should be able to open sqlite db.");

        Context { db }
    }
}
