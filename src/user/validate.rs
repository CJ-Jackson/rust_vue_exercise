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

#[derive(Debug, Error)]
#[error("Password is invalid")]
pub struct PasswordError(ValidateErrorItem);

impl ValidateErrorItemTrait for PasswordError {
    fn get_validate_error_item(&self) -> Option<ValidateErrorItem> {
        Some(self.0.clone())
    }
}

#[derive(Default)]
pub struct Password(String);

impl Password {
    pub fn parse(
        password: String,
        field_name: Option<String>,
    ) -> Result<Self, Report<PasswordError>> {
        let mut message: Vec<String> = vec![];
        let field_name = field_name.unwrap_or("password".to_string());
        let field_name_no_underscore = field_name.replace("_", " ");
        let password_validator = password.as_string_validator();

        let mut check_count_and_chars = true;
        password_validator.is_empty().then(|| {
            message.push(format!("{} cannot be empty", &field_name_no_underscore));
            check_count_and_chars = false;
        });
        check_count_and_chars.then(|| {
            (password_validator.count_graphemes() < 8).then(|| {
                message.push(format!(
                    "{} must be at least 8 characters",
                    &field_name_no_underscore
                ));
            });
            (password_validator.count_graphemes() > 64).then(|| {
                message.push(format!(
                    "{} must be at most 64 characters",
                    &field_name_no_underscore
                ));
            });
            (!password_validator.has_ascii_uppercase_and_lowercase()).then(|| {
                message.push(format!(
                    "{} must contain at least one uppercase and lowercase letter",
                    &field_name_no_underscore
                ));
            });
            (!password_validator.has_special_chars()).then(|| {
                message.push(format!(
                    "{} must contain at least one special character",
                    &field_name_no_underscore
                ));
            });
            (!password_validator.has_ascii_digit()).then(|| {
                message.push(format!(
                    "{} must contain at least one digit",
                    &field_name_no_underscore
                ));
            })
        });

        ValidateErrorItem::from_vec(field_name, message).then_err_report(|s| PasswordError(s))?;
        Ok(Self(password))
    }

    pub fn parse_confirm(
        &self,
        password_confirm: String,
        field_name: Option<String>,
    ) -> Result<Self, Report<PasswordError>> {
        let mut message: Vec<String> = vec![];
        let field_name = field_name.unwrap_or("password_confirm".to_string());
        let field_name_no_underscore = field_name.replace("_", " ");

        (password_confirm != self.as_str())
            .then(|| message.push(format!("{} does not match", &field_name_no_underscore)));

        ValidateErrorItem::from_vec(field_name, message).then_err_report(|s| PasswordError(s))?;
        Ok(Self(password_confirm))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod username {
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
    }

    mod password {
        use super::*;
        #[test]
        fn test_password_parse() {
            let password = Password::parse("Hello@Wor1d".to_string(), None);
            assert!(password.is_ok());
        }

        #[test]
        fn test_password_parse_error_empty_string() {
            let password = Password::parse("".to_string(), None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_too_short() {
            let password = Password::parse("a".to_string(), None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_too_long() {
            let password_str = "a".repeat(65);
            let password = Password::parse(password_str, None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_lower_case_only() {
            let password = Password::parse("hello@wor1d".to_string(), None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_upper_case_only() {
            let password = Password::parse("HELLO@WOR1D".to_string(), None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_special_char_only() {
            let password = Password::parse("!@#$%^&*()".to_string(), None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_digit_only() {
            let password = Password::parse("1234567890".to_string(), None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_password_confirmation_mismatch() {
            let password = Password("match".to_string());
            let password = password.parse_confirm("mismatch".to_string(), None);
            assert!(password.is_err());
        }

        #[test]
        fn test_password_parse_error_password_confirmation_match() {
            let password = Password("match".to_string());
            let password = password.parse_confirm("match".to_string(), None);
            assert!(password.is_ok());
        }
    }
}
