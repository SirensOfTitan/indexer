use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::platform::get_connect_opts;

#[derive(Clone)]
pub struct Context {
    pub db: SqlitePool,
}

impl Default for Context {
    fn default() -> Self {
        let db = SqlitePoolOptions::new().connect_lazy_with(get_connect_opts());

        Context { db }
    }
}
