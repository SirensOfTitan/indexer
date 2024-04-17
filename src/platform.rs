use std::{path::PathBuf, str::FromStr};

use directories::ProjectDirs;
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection};

pub const PROJECT_NAME: &str = "indexer";
pub const ORG_NAME: &str = "potrocky";
pub const QUALIFIER: &str = "";
pub const DB_NAME: &str = "db.sqlite3";

pub fn project_dirs() -> ProjectDirs {
    ProjectDirs::from(QUALIFIER, ORG_NAME, PROJECT_NAME).unwrap()
}

pub fn data_dir() -> PathBuf {
    let dirs = project_dirs();
    dirs.data_dir().to_path_buf()
}

pub fn get_db_path() -> String {
    let data = data_dir();
    data.join(DB_NAME)
        .to_str()
        .expect("Database path must be defined")
        .to_string()
}

pub async fn init_project_dirs() -> anyhow::Result<()> {
    let data_dir = data_dir();

    println!("Data dir at: {}", data_dir.display());

    if !data_dir.exists() {
        tokio::fs::create_dir(data_dir).await?;
    }

    Ok(())
}

pub fn get_connect_opts() -> SqliteConnectOptions {
    SqliteConnectOptions::from_str(&get_db_path())
        .expect("Should be able to get options")
        .create_if_missing(true)
        .extension("vector0")
        .extension("vss0")
}

pub async fn init_db() -> anyhow::Result<()> {
    let mut temp_conn = SqliteConnection::connect_with(&get_connect_opts()).await?;

    println!("Running migrations...");
    sqlx::migrate!("./migrations").run(&mut temp_conn).await?;
    Ok(())
}
