use crate::dependency::{DependencyError, DependencyGlobalContext, FromGlobalContext};
use crate::error::{ExtraResultExt, FromIntoStackError};
use crate::user::password::Password;
use error_stack::{Report, ResultExt};
use rusqlite::{Connection, named_params};
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};
use thiserror::Error;
use tokio::sync::OnceCell;

pub trait ConnectionMarker: Send + Sync + 'static {}

pub struct DefaultConnection;

impl ConnectionMarker for DefaultConnection {}

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

pub struct SqliteClient<T = DefaultConnection>(Arc<Mutex<Connection>>, PhantomData<T>)
where
    T: ConnectionMarker;

impl<T: ConnectionMarker> SqliteClient<T> {
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

            let password = Password::hash_password("banana".to_string())
                .change_context(SqliteClientError::InitFailed)
                .attach_critical("Failed to hash password".to_string())?
                .encode_to_msg_pack()
                .change_context(SqliteClientError::InitFailed)
                .attach_critical("Failed to encode password".to_string())?;

            conn.execute(
                include_str!("_sql/add_user.sql",),
                named_params! {
                    ":username": "default",
                    ":password": password.to_vec(),
                },
            )
            .change_context(SqliteClientError::InitFailed)
            .attach_critical("Failed to create default user".to_string())?;
        }

        Ok(SqliteClient(Arc::new(Mutex::new(conn)), PhantomData))
    }

    pub fn get_conn(&self) -> &Mutex<Connection> {
        self.0.as_ref()
    }
}

impl<T: ConnectionMarker> Clone for SqliteClient<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0), PhantomData)
    }
}

static SQLITE_CLIENT: OnceCell<SqliteClient> = OnceCell::const_new();

impl FromGlobalContext for SqliteClient {
    async fn from_global_context(
        dependency_global_context: &DependencyGlobalContext<'_, '_>,
    ) -> Result<Self, Report<DependencyError>> {
        let sqlite_client: Result<&Self, Report<DependencyError>> = SQLITE_CLIENT
            .get_or_try_init(|| async {
                Self::new(
                    dependency_global_context
                        .global_context
                        .config
                        .sqlite_path
                        .clone(),
                )
                .change_context(DependencyError::Other("Could not start SQLITE".to_string()))
            })
            .await;

        Ok(sqlite_client?.clone())
    }
}
