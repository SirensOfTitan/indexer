use std::{path::PathBuf, str::FromStr};

use directories::ProjectDirs;
use libsqlite3_sys::{sqlite3, sqlite3_api_routines};
use sqlx::{sqlite::SqliteConnectOptions, Connection, SqliteConnection};

use std::ffi::{c_char, c_int};

pub const PROJECT_NAME: &str = "indexer";
pub const ORG_NAME: &str = "potrocky";
pub const QUALIFIER: &str = "";
pub const DB_NAME: &str = "db.sqlite3";

#[link(name = "sqlite_vector0")]
extern "C" {
    pub fn sqlite3_vector_init(
        db: *mut sqlite3,
        pzErrMsg: *mut *mut c_char,
        pApi: *const sqlite3_api_routines,
    ) -> c_int;
}

#[link(name = "sqlite_vss0")]
extern "C" {
    pub fn sqlite3_vss_init(
        db: *mut sqlite3,
        pzErrMsg: *mut *mut c_char,
        pApi: *const sqlite3_api_routines,
    ) -> c_int;
}

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
}

pub async fn init_sqlite_extensions(conn: &mut SqliteConnection) -> anyhow::Result<()> {
    let mut handle = conn.lock_handle().await?;
    unsafe {
        let ptr = handle.as_raw_handle().as_mut();
        sqlite3_vector_init(ptr, std::ptr::null_mut(), std::ptr::null());
        sqlite3_vss_init(ptr, std::ptr::null_mut(), std::ptr::null());
    }

    Ok(())
}

pub async fn init_db() -> anyhow::Result<()> {
    let mut temp_conn = SqliteConnection::connect_with(&get_connect_opts()).await?;

    init_sqlite_extensions(&mut temp_conn).await?;

    println!("Running migrations...");
    sqlx::migrate!("./migrations").run(&mut temp_conn).await?;
    Ok(())
}
