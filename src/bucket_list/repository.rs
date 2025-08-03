use crate::bucket_list::model::{AddToBucketList, BucketListItem};
use crate::db::SqliteClient;
use crate::dep_context::DepContext;
use crate::error::ErrorStatus;
use error_stack::{Report, ResultExt};
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
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
    pub fn new(dep_context: &DepContext) -> Self {
        Self {
            sqlite_client: dep_context.sqlite_client.clone(),
        }
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

        Ok(item_iter
            .filter(|item| item.is_ok())
            .map(|item| item.unwrap())
            .collect())
    }

    pub fn add_to_bucket_list(
        &self,
        add_to_bucket_list: &AddToBucketList,
    ) -> Result<(), Report<BucketListRepositoryError>> {
        let conn = self
            .sqlite_client
            .get_conn()
            .lock()
            .map_err(|_| BucketListRepositoryError::LockError)?;

        conn.execute(
            include_str!("_sql/add_to_bucket_list.sql"),
            (&add_to_bucket_list.name, &add_to_bucket_list.description),
        )
        .change_context(BucketListRepositoryError::QueryError)?;

        Ok(())
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for BucketListRepository {
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.rocket().state::<DepContext>() {
            None => Outcome::Error((Status::InternalServerError, ())),
            Some(dep_context) => Outcome::Success(BucketListRepository::new(dep_context)),
        }
    }
}
