use crate::validation::{OptionValidateErrorItemTrait, ValidateErrorItem, ValidateErrorItemTrait};
use error_stack::Report;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

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
        let field_name = field_name.unwrap_or("name".to_string());
        let username_count = username.graphemes(true).count();

        username
            .is_empty()
            .then(|| message.push("Username cannot be empty".to_string()));
        (username_count < 5)
            .then(|| message.push("Username must be at least 5 characters".to_string()));
        (username_count > 30)
            .then(|| message.push("Username must be at most 30 characters".to_string()));

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
        password_confirmation: Option<&Password>,
    ) -> Result<Self, Report<PasswordError>> {
        let mut message: Vec<String> = vec![];
        let field_name = field_name.unwrap_or("password".to_string());
        let password_count = password.graphemes(true).count();

        match password_confirmation {
            Some(password_confirmation) => {
                (password != password_confirmation.as_str())
                    .then(|| message.push("Password confirmation does not match".to_string()));
            }
            None => {
                password
                    .is_empty()
                    .then(|| message.push("Password cannot be empty".to_string()));
                (password_count < 8)
                    .then(|| message.push("Password must be at least 8 characters".to_string()));
                (password_count > 64)
                    .then(|| message.push("Password must be at most 64 characters".to_string()));
            }
        }

        ValidateErrorItem::from_vec(field_name, message).then_err_report(|s| PasswordError(s))?;
        Ok(Self(password))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}
