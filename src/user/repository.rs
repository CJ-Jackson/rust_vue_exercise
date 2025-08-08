use crate::db::SqliteClient;
use crate::dependency::{DependencyError, DependencyFlagData, FromGlobalContext, GlobalContext};
use crate::user::model::{IdPassword, IdUsername};
use error_stack::{Report, ResultExt};
use rocket::Request;
use rusqlite::named_params;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum UserRepositoryError {
    #[error("Query error")]
    QueryError,
    #[error("Row Value error")]
    RowValueError,
    #[error("Lock error")]
    LockError,
    #[error("Not found error")]
    NotFoundError,
}

pub struct UserRepository {
    sqlite_client: SqliteClient,
}

impl UserRepository {
    pub fn new(sqlite_client: SqliteClient) -> Self {
        Self { sqlite_client }
    }

    pub fn add_token(
        &self,
        token: String,
        user_id: i64,
    ) -> Result<(), Report<UserRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| Report::new(UserRepositoryError::LockError))?;

        conn.execute(
            include_str!("_sql/add_token.sql"),
            named_params! {
                ":token": token,
                ":user_id": user_id,
            },
        )
        .change_context(UserRepositoryError::QueryError)?;

        Ok(())
    }

    pub fn delete_token(&self, token: String) -> Result<(), Report<UserRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| Report::new(UserRepositoryError::LockError))?;

        conn.execute(
            include_str!("_sql/delete_token.sql"),
            named_params! {
                ":token": token,
            },
        )
        .change_context(UserRepositoryError::QueryError)?;

        Ok(())
    }

    pub fn find_by_token(&self, token: String) -> Result<IdUsername, Report<UserRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| Report::new(UserRepositoryError::LockError))?;

        let mut stmt = conn
            .prepare(include_str!("_sql/find_by_token.sql"))
            .change_context(UserRepositoryError::QueryError)?;

        let mut item_iter = stmt
            .query_map(
                named_params! {
                    ":token": token,
                },
                |row| {
                    Ok(IdUsername {
                        id: row.get("id")?,
                        username: row.get("username")?,
                    })
                },
            )
            .change_context(UserRepositoryError::QueryError)?;

        let item = item_iter
            .next()
            .ok_or_else(|| Report::new(UserRepositoryError::NotFoundError))?;

        item.change_context(UserRepositoryError::RowValueError)
    }

    pub fn get_user_password(
        &self,
        username: String,
    ) -> Result<IdPassword, Report<UserRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| Report::new(UserRepositoryError::LockError))?;

        let mut stmt = conn
            .prepare(include_str!("_sql/get_user_password.sql"))
            .change_context(UserRepositoryError::QueryError)?;

        let mut item_iter = stmt
            .query_map(
                named_params! {
                    ":username": username,
                },
                |row| {
                    Ok(IdPassword {
                        id: row.get("id")?,
                        password: row.get("password")?,
                    })
                },
            )
            .change_context(UserRepositoryError::QueryError)?;

        let item = item_iter
            .next()
            .ok_or_else(|| Report::new(UserRepositoryError::NotFoundError))?;

        item.change_context(UserRepositoryError::RowValueError)
    }

    pub fn register_user(
        &self,
        username: String,
        password: String,
    ) -> Result<(), Report<UserRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| Report::new(UserRepositoryError::LockError))?;

        conn.execute(
            include_str!("_sql/register_user.sql"),
            named_params! {
                ":username": username,
                ":password": password,
            },
        )
        .change_context(UserRepositoryError::QueryError)?;

        Ok(())
    }
}

impl FromGlobalContext for UserRepository {
    fn from_global_context<'r>(
        dep_context: &GlobalContext,
        _flag: Arc<DependencyFlagData>,
        _request: Option<&'r Request<'_>>,
    ) -> Result<Self, DependencyError> {
        Ok(Self::new(dep_context.sqlite_client.clone()))
    }
}
