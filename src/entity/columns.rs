use std::{
    borrow::Cow,
    os::unix::ffi::{OsStrExt, OsStringExt},
    path::PathBuf,
};


use sqlx::{
    database::{HasValueRef},
    sqlite::{SqliteArgumentValue, SqliteTypeInfo},
    Decode, Encode, Sqlite, Type,
};

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct FilePath(PathBuf);

impl FilePath {
    pub fn new(buf: PathBuf) -> Self {
        FilePath(buf)
    }
}

impl Type<Sqlite> for FilePath {
    fn type_info() -> SqliteTypeInfo {
        <Vec<u8> as Type<Sqlite>>::type_info()
    }
}
impl Encode<'_, Sqlite> for FilePath {
    fn encode_by_ref(&self, args: &mut Vec<SqliteArgumentValue<'_>>) -> sqlx::encode::IsNull {
        let as_bytes = self.0.as_os_str().as_bytes().to_vec();
        args.push(SqliteArgumentValue::Blob(Cow::Owned(as_bytes)));
        sqlx::encode::IsNull::No
    }
}
impl<'r> Decode<'r, Sqlite> for FilePath {
    fn decode(
        value: <Sqlite as HasValueRef<'r>>::ValueRef,
    ) -> Result<Self, sqlx::error::BoxDynError> {
        let bytes = <Vec<u8> as Decode<Sqlite>>::decode(value)?;
        Ok(FilePath(PathBuf::from(std::ffi::OsString::from_vec(bytes))))
    }
}

impl From<&FilePath> for sea_query::Value {
    fn from(path: &FilePath) -> Self {
        let as_bytes = path.0.as_os_str().as_bytes();
        Self::Bytes(Some(Box::<Vec<u8>>::new(as_bytes.into())))
    }
}

impl From<FilePath> for sea_query::Value {
    fn from(path: FilePath) -> Self {
        let as_bytes = path.0.as_os_str().as_bytes();
        Self::Bytes(Some(Box::<Vec<u8>>::new(as_bytes.into())))
    }
}
