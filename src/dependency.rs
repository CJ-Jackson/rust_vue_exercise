use crate::config::{Config, get_figment_for_other};
use crate::db::SqliteClient;
use rocket::Request;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DependencyError {
    #[error("Needs request")]
    NeedsRequest,
}

pub struct DepContext {
    pub config: Arc<Config>,
    pub sqlite_client: SqliteClient,
}

impl DepContext {
    pub fn adhoc() -> AdHoc {
        AdHoc::on_ignite("DepContext", |rocket| async {
            let config = get_figment_for_other()
                .extract::<Arc<Config>>()
                .expect("Failed to extract config");

            let sqlite_client =
                SqliteClient::new(config.sqlite_path.clone()).expect("Failed to connect to sqlite");

            let dep_context = DepContext {
                config,
                sqlite_client,
            };

            rocket.manage(dep_context)
        })
    }
}

pub trait DependencyFlag {
    const FEATURE_FLAG: &'static str = "default";
    const USE_FORWARD: bool = false;

    fn feature_flag() -> String {
        Self::FEATURE_FLAG.to_string()
    }

    fn use_forward() -> bool {
        Self::USE_FORWARD
    }
}

pub struct DefaultFlag;

impl DependencyFlag for DefaultFlag {}

pub trait FromDepContext: Sized {
    fn from_dep_context<'r>(
        dep_context: &DepContext,
        feature_flag: String,
        request: Option<&'r Request<'_>>,
    ) -> Result<Self, DependencyError>;
}

pub struct DependencyGuard<T, F = DefaultFlag>(pub T, PhantomData<F>)
where
    T: FromDepContext,
    F: DependencyFlag;

pub type Dep<T, F = DefaultFlag> = DependencyGuard<T, F>;

#[rocket::async_trait]
impl<'r, T, F> FromRequest<'r> for DependencyGuard<T, F>
where
    T: FromDepContext,
    F: DependencyFlag,
{
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.rocket().state::<DepContext>() {
            None => {
                if F::use_forward() {
                    Outcome::Forward(Status::InternalServerError)
                } else {
                    Outcome::Error((Status::InternalServerError, ()))
                }
            }
            Some(dep_context) => {
                match T::from_dep_context(dep_context, F::feature_flag(), Some(req)) {
                    Ok(dep) => Outcome::Success(Self(dep, PhantomData)),
                    Err(_) => {
                        if F::use_forward() {
                            Outcome::Forward(Status::InternalServerError)
                        } else {
                            Outcome::Error((Status::InternalServerError, ()))
                        }
                    }
                }
            }
        }
    }
}

impl<T, F> Deref for DependencyGuard<T, F>
where
    T: FromDepContext,
    F: DependencyFlag,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DifferentFlag;

    impl DependencyFlag for DifferentFlag {
        const FEATURE_FLAG: &'static str = "different";
        const USE_FORWARD: bool = true;
    }

    #[test]
    fn flags() {
        assert_eq!(DefaultFlag::feature_flag(), "default");
        assert_eq!(DefaultFlag::use_forward(), false);

        assert_eq!(DifferentFlag::feature_flag(), "different");
        assert_eq!(DifferentFlag::use_forward(), true);
    }
}
