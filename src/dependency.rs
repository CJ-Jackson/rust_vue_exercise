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
    #[error("Dependency error: {0}")]
    Other(String),
}

pub struct GlobalContext {
    pub config: Arc<Config>,
    pub sqlite_client: SqliteClient,
}

impl GlobalContext {
    pub fn adhoc() -> AdHoc {
        AdHoc::on_ignite("DepContext", |rocket| async {
            let config = get_figment_for_other()
                .extract::<Arc<Config>>()
                .expect("Failed to extract config");

            let sqlite_client =
                SqliteClient::new(config.sqlite_path.clone()).expect("Failed to connect to sqlite");

            let dep_context = GlobalContext {
                config,
                sqlite_client,
            };

            rocket.manage(dep_context)
        })
    }

    /// Will not have a request
    pub async fn inject<T: FromGlobalContext>(
        &self,
        flag: &Arc<DependencyFlagData>,
    ) -> Result<T, DependencyError> {
        let dependency_global_context = Box::pin(DependencyGlobalContext {
            global_context: self,
            request: None,
        });
        T::from_global_context(&dependency_global_context, Arc::clone(flag)).await
    }
}

#[derive(Clone)]
pub struct DependencyFlagData {
    pub feature_flag: Box<[String]>,
    pub use_forward: bool,
    pub allow_user: bool,
    pub allow_visitor: bool,
}

impl DependencyFlagData {
    pub fn override_feature_flag(&self, feature_flag: &str) -> Arc<Self> {
        Arc::new(Self {
            feature_flag: feature_flag.split(' ').map(String::from).collect(),
            ..self.clone()
        })
    }
}

pub trait DependencyFlag {
    const FEATURE_FLAG: &'static str = "default";
    const USE_FORWARD: bool = false;
    const ALLOW_USER: bool = true;
    const ALLOW_VISITOR: bool = true;

    fn build_flag_data() -> Arc<DependencyFlagData> {
        Arc::new(DependencyFlagData {
            feature_flag: Self::FEATURE_FLAG.split(' ').map(String::from).collect(),
            use_forward: Self::USE_FORWARD,
            allow_user: Self::ALLOW_USER,
            allow_visitor: Self::ALLOW_VISITOR,
        })
    }
}

pub struct DefaultFlag;

impl DependencyFlag for DefaultFlag {}

pub struct DependencyGlobalContext<'r, 'life0> {
    pub global_context: &'r GlobalContext,
    pub request: Option<&'r Request<'life0>>,
}

impl DependencyGlobalContext<'_, '_> {
    pub async fn inject<T: FromGlobalContext>(
        &self,
        flag: &Arc<DependencyFlagData>,
    ) -> Result<T, DependencyError> {
        T::from_global_context(self, Arc::clone(flag)).await
    }
}

pub trait FromGlobalContext: Sized {
    fn from_global_context<'r>(
        dependency_global_context: &'r DependencyGlobalContext<'r, '_>,
        flag: Arc<DependencyFlagData>,
    ) -> impl Future<Output = Result<Self, DependencyError>> + Send;
}

pub struct DependencyGuard<T, F = DefaultFlag>(pub T, PhantomData<F>)
where
    T: FromGlobalContext,
    F: DependencyFlag;

pub type Dep<T, F = DefaultFlag> = DependencyGuard<T, F>;

#[rocket::async_trait]
impl<'r, T, F> FromRequest<'r> for DependencyGuard<T, F>
where
    T: FromGlobalContext,
    F: DependencyFlag,
{
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let flag = F::build_flag_data();
        match req.rocket().state::<GlobalContext>() {
            None => {
                if flag.use_forward {
                    Outcome::Forward(Status::InternalServerError)
                } else {
                    Outcome::Error((Status::InternalServerError, ()))
                }
            }
            Some(global_context) => {
                let dependency_global_context = Box::pin(DependencyGlobalContext {
                    global_context,
                    request: Some(req),
                });
                match T::from_global_context(&dependency_global_context, Arc::clone(&flag)).await {
                    Ok(dep) => Outcome::Success(Self(dep, PhantomData)),
                    Err(_) => {
                        if flag.use_forward {
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
    T: FromGlobalContext,
    F: DependencyFlag,
{
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
