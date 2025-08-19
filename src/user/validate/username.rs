use crate::validation::{
    OptionValidateErrorItemTrait, StrValidationExtension, ValidateErrorItem, ValidateErrorItemTrait,
};
use error_stack::Report;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("Username is invalid")]
pub struct UsernameError(ValidateErrorItem);

impl ValidateErrorItemTrait for UsernameError {
    fn get_validate_error_item(&self) -> Option<ValidateErrorItem> {
        Some(self.0.clone())
    }
}

#[derive(Default)]
pub struct Username(String);

impl Username {
    pub fn parse(
        username: String,
        field_name: Option<String>,
    ) -> Result<Self, Report<UsernameError>> {
        let mut message: Vec<String> = vec![];
        let field_name = field_name.unwrap_or("username".to_string());
        let field_name_no_underscore = field_name.replace("_", " ");
        let username_validator = username.as_string_validator();

        let mut check_count = true;
        username_validator.is_empty().then(|| {
            message.push(format!("{} cannot be empty", &field_name_no_underscore));
            check_count = false;
        });
        check_count.then(|| {
            (username_validator.count_graphemes() < 5).then(|| {
                message.push(format!(
                    "{} must be at least 5 characters",
                    &field_name_no_underscore
                ))
            });
            (username_validator.count_graphemes() > 30).then(|| {
                message.push(format!(
                    "{} must be at most 30 characters",
                    &field_name_no_underscore
                ))
            });
        });

        ValidateErrorItem::from_vec(field_name, message).then_err_report(|s| UsernameError(s))?;
        Ok(Self(username))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

pub trait IsUsernameTaken {
    fn is_username_taken(&self, username: &str) -> impl Future<Output = bool>;
}

trait Sealed {}

#[allow(private_bounds)]
pub trait UsernameCheckResult: Sealed {
    fn check_username_result<T: IsUsernameTaken>(
        self,
        service: &T,
        field_name: Option<String>,
    ) -> impl Future<Output = Self>;
}

impl Sealed for Result<Username, Report<UsernameError>> {}

impl UsernameCheckResult for Result<Username, Report<UsernameError>> {
    async fn check_username_result<T: IsUsernameTaken>(
        self,
        service: &T,
        field_name: Option<String>,
    ) -> Self {
        match self {
            Ok(v) => {
                let mut message: Vec<String> = vec![];
                let field_name = field_name.unwrap_or("username".to_string());
                let field_name_no_underscore = field_name.replace("_", " ");

                service.is_username_taken(v.as_str()).await.then(|| {
                    message.push(format!("{} is already taken", &field_name_no_underscore));
                });

                ValidateErrorItem::from_vec(field_name, message)
                    .then_err_report(|s| UsernameError(s))?;

                Ok(v)
            }
            Err(_) => self,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_username_parse() {
        let username = Username::parse("Hello".to_string(), None);
        assert!(username.is_ok());
    }

    #[test]
    fn test_username_parse_error_empty_string() {
        let username = Username::parse("".to_string(), None);
        assert!(username.is_err());
    }

    #[test]
    fn test_username_parse_error_too_short() {
        let username = Username::parse("a".to_string(), None);
        assert!(username.is_err());
    }

    #[test]
    fn test_username_parse_error_too_long() {
        let username_str = "a".repeat(31);
        let username = Username::parse(username_str, None);
        assert!(username.is_err());
    }

    struct FakeUsernameCheckService(String);

    impl IsUsernameTaken for FakeUsernameCheckService {
        async fn is_username_taken(&self, username: &str) -> bool {
            username == self.0.as_str()
        }
    }

    #[tokio::test]
    async fn username_is_taken() {
        let username_result: Result<Username, Report<UsernameError>> =
            Ok(Username("taken".to_string()));

        assert!(
            username_result
                .check_username_result(&FakeUsernameCheckService("taken".to_string()), None)
                .await
                .is_err()
        )
    }

    #[tokio::test]
    async fn username_is_not_taken() {
        let username_result: Result<Username, Report<UsernameError>> =
            Ok(Username("not_taken".to_string()));

        assert!(
            username_result
                .check_username_result(&FakeUsernameCheckService("taken".to_string()), None)
                .await
                .is_ok()
        )
    }
}
