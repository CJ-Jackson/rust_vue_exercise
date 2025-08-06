use crate::dependency::{DepContext, DependencyError, FromDepContext};
use crate::user::dependency::FromUserContext;
use crate::user::model::UserContext;
use rocket::Request;
use std::sync::Arc;

const TOKEN: &str = "IamLoggedIn";

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
    username_cookie: Option<String>,
    token_cookie: Option<String>,
}

impl UserCheckService {
    fn new(username_cookie: Option<String>, token_cookie: Option<String>) -> Self {
        Self {
            username_cookie,
            token_cookie,
        }
    }

    pub fn get_user_context(&self) -> UserContext {
        if self.is_logged_in() {
            let username = self.username_cookie.clone().unwrap_or("".to_string());
            UserContext {
                is_user: true,
                username,
            }
        } else {
            UserContext {
                is_user: false,
                username: "Visitor".to_string(),
            }
        }
    }

    fn is_logged_in(&self) -> bool {
        if self.username_cookie.is_some() {
            if let Some(token) = &self.token_cookie
                && token == TOKEN
            {
                return true;
            }
        }
        false
    }
}

impl FromDepContext for UserCheckService {
    fn from_dep_context(
        _dep_context: &DepContext,
        _feature_flag: String,
        request: Option<&Request>,
    ) -> Result<Self, DependencyError> {
        let request = request.ok_or(DependencyError::NeedsRequest)?;
        let cookies = request.cookies();

        Ok(Self::new(
            cookies.get("login-username").map(|c| c.value().to_string()),
            cookies.get("login-token").map(|c| c.value().to_string()),
        ))
    }
}

pub struct UserLoginService;

impl UserLoginService {
    pub fn validate_login(&self, _username: String) -> String {
        TOKEN.to_string()
    }
}

impl FromUserContext for UserLoginService {
    fn from_user_context(
        _user_context: Arc<UserContext>,
        _dep_context: &DepContext,
        _feature_flag: String,
        _request: Option<&Request>,
    ) -> Result<Self, DependencyError> {
        Ok(Self)
    }
}
