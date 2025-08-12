use crate::bucket_list::model::{AddToBucketListValidated, BucketListItem};
use crate::db::SqliteClient;
use crate::dependency::{DependencyError, DependencyGlobalContext, FromGlobalContext};
use crate::error::ErrorStatus;
use error_stack::{Report, ResultExt};
use rocket::http::Status;
use rusqlite::named_params;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum BucketListRepositoryError {
    #[error("Query error")]
    QueryError,
    #[error("Row Value error")]
    RowValueError,
    #[error("Lock error")]
    LockError,
}

impl ErrorStatus for BucketListRepositoryError {
    fn error_status(&self) -> Status {
        Status::InternalServerError
    }
}

pub struct BucketListRepository {
    sqlite_client: SqliteClient,
}

impl BucketListRepository {
    pub fn new(sqlite_client: SqliteClient) -> Self {
        Self { sqlite_client }
    }

    pub fn get_all_from_bucket_list(
        &self,
    ) -> Result<Box<[BucketListItem]>, Report<BucketListRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| BucketListRepositoryError::LockError)?;

        let mut stmt = conn
            .prepare(include_str!("_sql/get_all_from_bucket_list.sql"))
            .change_context(BucketListRepositoryError::QueryError)?;

        let item_iter = stmt
            .query_map([], |row| {
                Ok(BucketListItem {
                    id: row.get("id")?,
                    name: row.get("name")?,
                    description: row.get("description")?,
                    timestamp: row.get("timestamp")?,
                })
            })
            .change_context(BucketListRepositoryError::RowValueError)?;

        let mut items: Vec<BucketListItem> = Vec::new();
        for item in item_iter {
            items.push(item.change_context(BucketListRepositoryError::RowValueError)?);
        }

        Ok(items.into())
    }

    pub fn add_to_bucket_list(
        &self,
        add_to_bucket_list: &AddToBucketListValidated,
    ) -> Result<(), Report<BucketListRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| BucketListRepositoryError::LockError)?;

        conn.execute(
            include_str!("_sql/add_to_bucket_list.sql"),
            named_params! {
                ":name": add_to_bucket_list.name.as_str(),
                ":description": add_to_bucket_list.description.as_str(),
            },
        )
        .change_context(BucketListRepositoryError::QueryError)?;

        Ok(())
    }
}

impl FromGlobalContext for BucketListRepository {
    async fn from_global_context(
        dependency_global_context: &DependencyGlobalContext<'_, '_>,
    ) -> Result<Self, DependencyError> {
        Ok(Self::new(dependency_global_context.inject().await?))
    }
}
