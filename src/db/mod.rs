use crate::error::{ExtraResultExt, FromIntoStackError};
use error_stack::{Report, ResultExt};
use rusqlite::Connection;
use std::sync::{Arc, Mutex};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SqliteClientError {
    #[error("Sqlite file empty")]
    SqliteFileEmpty,
    #[error("Connection error")]
    Connection,
    #[error("Init failed")]
    InitFailed,
}

impl FromIntoStackError for SqliteClientError {}

pub struct SqliteClient(Arc<Mutex<Connection>>);

impl SqliteClient {
    pub fn new(sqlite_path: String) -> Result<Self, Report<SqliteClientError>> {
        if sqlite_path.is_empty() {
            return Err(SqliteClientError::SqliteFileEmpty
                .into_stack_error_critical("Sqlite file path is empty".to_string()));
        }
        let file_exist = std::fs::metadata(&sqlite_path).is_ok();

        let conn = Connection::open(sqlite_path)
            .change_context(SqliteClientError::Connection)
            .attach_critical("Sqlite Connection failed".to_string())?;
        if !file_exist {
            conn.execute_batch(include_str!("_sql/init.sql"))
                .change_context(SqliteClientError::InitFailed)
                .attach_critical("Init failed".to_string())?;
        }

        Ok(SqliteClient(Arc::new(Mutex::new(conn))))
    }

    pub fn get_conn(&self) -> &Mutex<Connection> {
        self.0.as_ref()
    }
}

impl Clone for SqliteClient {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}
