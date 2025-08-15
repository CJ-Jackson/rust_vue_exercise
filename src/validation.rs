use error_stack::Report;
use maud::{Markup, html};
use rocket::serde::json::Json;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;

pub trait ValidateErrorItemTrait: Sized + Send + Sync + 'static {
    fn get_validate_error_item(&self) -> Option<ValidateErrorItem>;
}

impl<E> ValidateErrorItemTrait for Report<E>
where
    E: ValidateErrorItemTrait,
{
    fn get_validate_error_item(&self) -> Option<ValidateErrorItem> {
        self.current_context().get_validate_error_item()
    }
}

impl<T, E> ValidateErrorItemTrait for Result<T, Report<E>>
where
    E: ValidateErrorItemTrait,
    T: Send + Sync + 'static,
{
    fn get_validate_error_item(&self) -> Option<ValidateErrorItem> {
        if let Err(e) = self {
            return e.get_validate_error_item();
        }
        None
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct ValidateErrorItem {
    field_name: String,
    messages: Box<[String]>,
}

impl ValidateErrorItem {
    pub fn html(&self) -> Markup {
        html! {
            ul .validation-error-list {
                @for message in &self.messages {
                    li .validation-error-message { (message) }
                }
            }
        }
    }
}

trait MarkupSealed {}

#[allow(private_bounds)]
pub trait ValidationOptionMarkup: MarkupSealed {
    fn html(&self) -> Markup;
}

impl MarkupSealed for Option<&ValidateErrorItem> {}

impl ValidationOptionMarkup for Option<&ValidateErrorItem> {
    fn html(&self) -> Markup {
        match self {
            Some(item) => item.html(),
            None => html! {},
        }
    }
}

impl ValidateErrorItem {
    pub fn from_vec(field_name: String, messages: Vec<String>) -> Option<Self> {
        if messages.is_empty() {
            return None;
        }

        Some(Self {
            field_name,
            messages: messages.into(),
        })
    }
}

trait OptionSealed {}

#[allow(private_bounds)]
pub trait OptionValidateErrorItemTrait: OptionSealed {
    fn then_err_report<F, E>(self, f: F) -> Result<(), Report<E>>
    where
        F: FnOnce(ValidateErrorItem) -> E,
        E: Error + Send + Sync + 'static;
}

impl OptionSealed for Option<ValidateErrorItem> {}

impl OptionValidateErrorItemTrait for Option<ValidateErrorItem> {
    fn then_err_report<F, E>(self, f: F) -> Result<(), Report<E>>
    where
        F: FnOnce(ValidateErrorItem) -> E,
        E: Error + Send + Sync + 'static,
    {
        if let Some(item) = self {
            Err(Report::new(f(item)))
        } else {
            Ok(())
        }
    }
}

#[derive(Responder)]
#[response(status = 422)]
pub struct ValidationErrorResponse(Json<Box<[ValidateErrorItem]>>);

impl ValidationErrorResponse {
    pub fn as_map(&self) -> HashMap<String, ValidateErrorItem> {
        let mut map = HashMap::new();
        for item in &self.0.0 {
            match map.get(&item.field_name) {
                None => {
                    map.insert(item.field_name.clone(), item.clone());
                }
                Some(value) => {
                    let current_messages = value.messages.clone();
                    let new_messages = item.messages.clone();
                    let merge_message = current_messages
                        .into_iter()
                        .chain(new_messages.into_iter())
                        .collect::<Vec<_>>()
                        .into_boxed_slice();
                    let new_item = ValidateErrorItem {
                        field_name: item.field_name.clone(),
                        messages: merge_message,
                    };
                    map.insert(item.field_name.clone(), new_item);
                }
            }
        }
        map
    }
}

impl Display for ValidationErrorResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(for item in &self.0.0 {
            write!(f, "{};\n", item.messages.join(", "))?
        })
    }
}

#[derive(Responder)]
#[response(status = 422)]
pub struct ValidationErrorMergedResponse(Json<HashMap<String, Box<[ValidateErrorItem]>>>);

impl ValidationErrorMergedResponse {
    pub fn as_map(&self) -> HashMap<String, ValidateErrorItem> {
        let mut map = HashMap::new();
        for (_, items) in &self.0.0 {
            for item in items {
                match map.get(&item.field_name) {
                    None => {
                        map.insert(item.field_name.clone(), item.clone());
                    }
                    Some(value) => {
                        let current_messages = value.messages.clone();
                        let new_messages = item.messages.clone();
                        let merge_message = current_messages
                            .into_iter()
                            .chain(new_messages.into_iter())
                            .collect::<Vec<_>>()
                            .into_boxed_slice();
                        let new_item = ValidateErrorItem {
                            field_name: item.field_name.clone(),
                            messages: merge_message,
                        };
                        map.insert(item.field_name.clone(), new_item);
                    }
                }
            }
        }
        map
    }
}

impl Display for ValidationErrorMergedResponse {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Ok(for (_, items) in &self.0.0 {
            for item in items {
                write!(f, "{};\n", item.messages.join(", "))?
            }
        })
    }
}

pub struct ValidationErrorsBuilder(Vec<ValidateErrorItem>);

impl ValidationErrorsBuilder {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn add(&mut self, field_name: String, messages: Vec<String>) {
        if let Some(item) = ValidateErrorItem::from_vec(field_name, messages) {
            self.0.push(item);
        }
    }

    pub fn add_item_from_trait<T: ValidateErrorItemTrait>(&mut self, item: T) -> T {
        if let Some(item) = item.get_validate_error_item() {
            self.0.push(item);
        }
        item
    }

    fn build(self) -> ValidationErrorResponse {
        ValidationErrorResponse(Json(self.0.into()))
    }

    pub fn build_result(self) -> Result<(), ValidationErrorResponse> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.build())
        }
    }
}

#[derive(Clone, Debug)]
pub struct ValidationErrorsMergeBuilder(HashMap<String, Box<[ValidateErrorItem]>>);

impl ValidationErrorsMergeBuilder {
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    pub fn merge(mut self, name: String, errors: Result<(), ValidationErrorResponse>) -> Self {
        if let Err(e) = errors {
            self.0.insert(name, e.0.into_inner());
        }
        self
    }

    fn build(self) -> ValidationErrorMergedResponse {
        ValidationErrorMergedResponse(Json(self.0))
    }

    pub fn build_result(self) -> Result<(), ValidationErrorMergedResponse> {
        if self.0.is_empty() {
            Ok(())
        } else {
            Err(self.build())
        }
    }
}

trait StrSealed {
    const SPECIAL_CHARS: [char; 30] = [
        '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', ']', '{', '}',
        '\\', '|', ';', ':', '\'', '"', ',', '.', '<', '>', '/', '?',
    ];
}

#[allow(private_bounds)]
pub trait StrValidationExtension: StrSealed {
    fn has_special_chars(&self) -> bool;
    fn has_ascii_uppercase(&self) -> bool;
    fn has_ascii_lowercase(&self) -> bool;
    fn has_ascii_uppercase_and_lowercase(&self) -> bool {
        self.has_ascii_uppercase() && self.has_ascii_lowercase()
    }
    fn has_ascii_digit(&self) -> bool;
    fn has_ascii_alphanumeric(&self) -> bool;
}

impl StrSealed for &str {}

impl StrValidationExtension for &str {
    fn has_special_chars(&self) -> bool {
        self.chars().any(|c| Self::SPECIAL_CHARS.contains(&c))
    }

    fn has_ascii_uppercase(&self) -> bool {
        self.chars().any(|c| c.is_ascii_uppercase())
    }

    fn has_ascii_lowercase(&self) -> bool {
        self.chars().any(|c| c.is_ascii_lowercase())
    }

    fn has_ascii_digit(&self) -> bool {
        self.chars().any(|c| c.is_ascii_digit())
    }

    fn has_ascii_alphanumeric(&self) -> bool {
        self.chars().any(|c| c.is_ascii_alphanumeric())
    }
}

impl StrSealed for String {}

impl StrValidationExtension for String {
    fn has_special_chars(&self) -> bool {
        self.as_str().has_special_chars()
    }

    fn has_ascii_uppercase(&self) -> bool {
        self.as_str().has_ascii_uppercase()
    }

    fn has_ascii_lowercase(&self) -> bool {
        self.as_str().has_ascii_lowercase()
    }

    fn has_ascii_digit(&self) -> bool {
        self.as_str().has_ascii_digit()
    }

    fn has_ascii_alphanumeric(&self) -> bool {
        self.as_str().has_ascii_alphanumeric()
    }
}
