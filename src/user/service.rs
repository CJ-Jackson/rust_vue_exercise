use crate::dependency::{DependencyError, DependencyGlobalContext, FromGlobalContext};
use crate::user::dependency::{DependencyUserContext, FromUserContext};
use crate::user::model::{IdUsername, UserContext};
use crate::user::password::Password;
use crate::user::repository::UserRepository;
use crate::user::validate::username::IsUsernameTaken;
use error_stack::Report;
use uuid::Uuid;

pub struct NoopService;

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

pub struct UserLoginService {
    user_repository: UserRepository,
    token_cookie: Option<String>,
}

impl UserLoginService {
    fn new(user_repository: UserRepository, token_cookie: Option<String>) -> Self {
        Self {
            user_repository,
            token_cookie,
        }
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

    pub fn logout(&self) -> bool {
        if let Some(token) = &self.token_cookie {
            self.user_repository.delete_token(token.clone()).is_ok()
        } else {
            false
        }
    }
}

pub struct UserRegisterService {
    user_repository: UserRepository,
}

impl UserRegisterService {
    pub fn new(user_repository: UserRepository) -> Self {
        Self { user_repository }
    }

    pub fn register_user(&self, username: String, password: String) -> bool {
        let password = match Password::hash_password(password) {
            Ok(password) => password,
            Err(_) => return false,
        };
        let password = match password.encode_to_msg_pack() {
            Ok(password) => password,
            Err(_) => return false,
        };

        self.user_repository
            .register_user(username, password)
            .is_ok()
    }
}

impl IsUsernameTaken for UserRegisterService {
    async fn is_username_taken(&self, username: &str) -> bool {
        self.user_repository
            .username_taken(username.to_string())
            .is_ok()
    }
}

impl FromGlobalContext for NoopService {
    async fn from_global_context(
        _dependency_global_context: &DependencyGlobalContext<'_, '_>,
    ) -> Result<Self, Report<DependencyError>> {
        Ok(Self)
    }
}

impl FromUserContext for NoopService {
    async fn from_user_context(
        _dependency_user_context: &DependencyUserContext<'_, '_>,
    ) -> Result<Self, Report<DependencyError>> {
        Ok(Self)
    }
}

impl FromGlobalContext for UserCheckService {
    async fn from_global_context(
        dependency_global_context: &DependencyGlobalContext<'_, '_>,
    ) -> Result<Self, Report<DependencyError>> {
        let request = dependency_global_context
            .request
            .ok_or(DependencyError::NeedsRequest)?;
        let cookies = request.cookies();

        Ok(Self::new(
            dependency_global_context.inject().await?,
            cookies.get("login-token").map(|c| c.value().to_string()),
        ))
    }
}

impl FromUserContext for UserLoginService {
    async fn from_user_context(
        dependency_user_context: &DependencyUserContext<'_, '_>,
    ) -> Result<Self, Report<DependencyError>> {
        let request = dependency_user_context
            .request
            .ok_or(DependencyError::NeedsRequest)?;
        let cookies = request.cookies();

        Ok(Self::new(
            dependency_user_context.inject_global().await?,
            cookies.get("login-token").map(|c| c.value().to_string()),
        ))
    }
}

impl FromUserContext for UserRegisterService {
    async fn from_user_context(
        dependency_user_context: &DependencyUserContext<'_, '_>,
    ) -> Result<Self, Report<DependencyError>> {
        Ok(Self::new(dependency_user_context.inject_global().await?))
    }
}
