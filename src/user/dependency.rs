use crate::dependency::{
    DefaultFlag, Dep, DependencyError, DependencyFlag, DependencyFlagData, GlobalContext,
};
use crate::user::model::UserContext;
use crate::user::service::UserCheckService;
use rocket::Request;
use rocket::http::Status;
use rocket::request::{FromRequest, Outcome};
use std::marker::PhantomData;
use std::sync::Arc;

pub trait FromUserContext: Sized {
    fn from_user_context<'r>(
        user_context: Arc<UserContext>,
        global_context: &GlobalContext,
        flag: Arc<DependencyFlagData>,
        request: Option<&'r Request<'_>>,
    ) -> impl Future<Output = Result<Self, DependencyError>> + Send;
}

pub struct UserDependencyGuard<T, F = DefaultFlag>(pub T, pub Arc<UserContext>, PhantomData<F>)
where
    T: FromUserContext,
    F: DependencyFlag;

pub type UserDep<T, F = DefaultFlag> = UserDependencyGuard<T, F>;

#[rocket::async_trait]
impl<'r, T, F> FromRequest<'r> for UserDependencyGuard<T, F>
where
    T: FromUserContext,
    F: DependencyFlag,
{
    type Error = ();

    async fn from_request(req: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let flag = F::build_flag_data();
        let user_context = req
            .local_cache_async(async {
                let user_service = req.guard::<Dep<UserCheckService>>().await.succeeded()?;

                Some(Arc::new(user_service.get_user_context()))
            })
            .await;

        let user_context = match user_context {
            None => {
                return if flag.use_forward {
                    Outcome::Forward(Status::InternalServerError)
                } else {
                    Outcome::Error((Status::InternalServerError, ()))
                };
            }
            Some(user_context) => Arc::clone(user_context),
        };

        if user_context.is_user && !flag.allow_user {
            return if flag.use_forward {
                Outcome::Forward(Status::Unauthorized)
            } else {
                Outcome::Error((Status::Unauthorized, ()))
            };
        } else if !user_context.is_user && !flag.allow_visitor {
            return if flag.use_forward {
                Outcome::Forward(Status::Forbidden)
            } else {
                Outcome::Error((Status::Forbidden, ()))
            };
        }

        match req.rocket().state::<GlobalContext>() {
            None => {
                if flag.use_forward {
                    Outcome::Forward(Status::InternalServerError)
                } else {
                    Outcome::Error((Status::InternalServerError, ()))
                }
            }
            Some(global_context) => {
                match T::from_user_context(
                    Arc::clone(&user_context),
                    global_context,
                    Arc::clone(&flag),
                    Some(req),
                )
                .await
                {
                    Ok(dep) => Outcome::Success(Self(dep, user_context, PhantomData)),
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
