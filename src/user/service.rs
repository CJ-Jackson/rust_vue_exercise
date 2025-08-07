use crate::dependency::{DepContext, DependencyError, FromDepContext};
use crate::user::dependency::FromUserContext;
use crate::user::model::{IdUsername, UserContext};
use crate::user::password::Password;
use crate::user::repository::UserRepository;
use rocket::Request;
use std::sync::Arc;
use uuid::Uuid;

pub struct NoopService;

impl FromDepContext for NoopService {
    fn from_dep_context(
        _dep_context: &DepContext,
        _feature_flag: String,
        _request: Option<&Request>,
    ) -> Result<Self, DependencyError> {
        Ok(Self)
    }
}

impl FromUserContext for NoopService {
    fn from_user_context(
        _user_context: Arc<UserContext>,
        _dep_context: &DepContext,
        _feature_flag: String,
        _request: Option<&Request>,
    ) -> Result<Self, DependencyError> {
        Ok(Self)
    }
}

pub struct UserCheckService {
    user_repository: UserRepository,
    token_cookie: Option<String>,
}

impl UserCheckService {
    fn new(user_repository: UserRepository, token_cookie: Option<String>) -> Self {
        Self {
            user_repository,
            token_cookie,
        }
    }

    pub fn get_user_context(&self) -> UserContext {
        if let Some(id_username) = self.is_logged_in() {
            UserContext {
                id: id_username.id,
                is_user: true,
                username: id_username.username,
            }
        } else {
            UserContext {
                id: 0,
                is_user: false,
                username: "Visitor".to_string(),
            }
        }
    }

    fn is_logged_in(&self) -> Option<IdUsername> {
        if let Some(token) = &self.token_cookie {
            if let Ok(id_username) = self.user_repository.find_by_token(token.clone()) {
                return Some(id_username);
            }
        }

        None
    }
}

impl FromDepContext for UserCheckService {
    fn from_dep_context(
        dep_context: &DepContext,
        feature_flag: String,
        request: Option<&Request>,
    ) -> Result<Self, DependencyError> {
        let request = request.ok_or(DependencyError::NeedsRequest)?;
        let cookies = request.cookies();

        Ok(Self::new(
            UserRepository::from_dep_context(dep_context, feature_flag, None)?,
            cookies.get("login-token").map(|c| c.value().to_string()),
        ))
    }
}

pub struct UserLoginService {
    user_repository: UserRepository,
}

impl UserLoginService {
    fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }
    pub fn validate_login(&self, username: String, password: String) -> Option<String> {
        if let Ok(id_password) = self.user_repository.get_user_password(username) {
            let password_status = Password::verify_password(id_password.password, password);
            if let Ok(password_status) = password_status {
                if password_status.is_valid() {
                    let uuid_token = Uuid::new_v4().to_string();

                    if self
                        .user_repository
                        .add_token(uuid_token.clone(), id_password.id)
                        .is_err()
                    {
                        return None;
                    }

                    return Some(uuid_token);
                }
            }
        }

        None
    }
}

impl FromUserContext for UserLoginService {
    fn from_user_context(
        _user_context: Arc<UserContext>,
        dep_context: &DepContext,
        feature_flag: String,
        _request: Option<&Request>,
    ) -> Result<Self, DependencyError> {
        Ok(Self::new(UserRepository::from_dep_context(
            dep_context,
            feature_flag,
            None,
        )?))
    }
}
