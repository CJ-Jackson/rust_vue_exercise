use crate::config::{Config, get_figment_for_other};
use crate::db::SqliteClient;
use rocket::Request;
use rocket::fairing::AdHoc;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;

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

pub trait DepFeatureFlag {
    const FEATURE_FLAG: &'static str;
}

pub struct DefaultFeatureFlag;

impl DepFeatureFlag for DefaultFeatureFlag {
    const FEATURE_FLAG: &'static str = "default";
}

pub trait FromDepContext {
    fn from_dep_context(dep_context: &DepContext, feature_flag: String) -> Self;
}

pub struct DepContextGuard<T, E = DefaultFeatureFlag>(pub T, PhantomData<E>)
where
    T: FromDepContext,
    E: DepFeatureFlag;

pub type Dep<T, E = DefaultFeatureFlag> = DepContextGuard<T, E>;

#[rocket::async_trait]
impl<'r, T, E> FromRequest<'r> for DepContextGuard<T, E>
where
    T: FromDepContext,
    E: DepFeatureFlag,
{
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        match req.rocket().state::<DepContext>() {
            None => Outcome::Error((Status::InternalServerError, ())),
            Some(dep_context) => Outcome::Success(Self(
                T::from_dep_context(dep_context, E::FEATURE_FLAG.to_string()),
                PhantomData,
            )),
        }
    }
}

impl<T> Deref for DepContextGuard<T>
where
    T: FromDepContext,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
