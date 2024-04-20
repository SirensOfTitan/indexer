use sqlx::{sqlite::SqlitePoolOptions, SqlitePool};

use crate::platform::{get_connect_opts, init_sqlite_extensions};

#[derive(Clone)]
pub struct Context {
    pub db: SqlitePool,
}

impl Default for Context {
    fn default() -> Self {
        let db = SqlitePoolOptions::new()
            .after_connect(|conn, _| {
                Box::pin(async move {
                    init_sqlite_extensions(conn)
                        .await
                        .expect("should be able to init sqlite extensions");

                    Ok(())
                })
            })
            .before_acquire(|conn, _| {
                Box::pin(async move {
                    init_sqlite_extensions(conn)
                        .await
                        .expect("should be able to init sqlite extensions");

                    Ok(true)
                })
            })
            .connect_lazy_with(get_connect_opts());

        Context { db }
    }
}
