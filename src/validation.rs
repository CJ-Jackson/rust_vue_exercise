use error_stack::Report;
use maud::{Markup, html};
use rocket::serde::json::Json;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fmt::Display;
use unicode_segmentation::UnicodeSegmentation;

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
    pub fn as_html(&self) -> Markup {
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
    fn as_html(&self) -> Markup;
}

impl MarkupSealed for Option<&ValidateErrorItem> {}

impl ValidationOptionMarkup for Option<&ValidateErrorItem> {
    fn as_html(&self) -> Markup {
        match self {
            Some(item) => item.as_html(),
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

pub struct StringValidator<'a>(&'a str, usize);

impl<'a> StringValidator<'a> {
    const SPECIAL_CHARS: [char; 30] = [
        '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+', '[', ']', '{', '}',
        '\\', '|', ';', ':', '\'', '"', ',', '.', '<', '>', '/', '?',
    ];

    fn new(s: &'a str) -> Self {
        Self(s, s.graphemes(true).count())
    }

    pub fn count_graphemes(&self) -> usize {
        self.1
    }

    pub fn is_empty(&self) -> bool {
        self.1 == 0
    }

    pub fn has_special_chars(&self) -> bool {
        self.0.chars().any(|c| Self::SPECIAL_CHARS.contains(&c))
    }

    pub fn count_special_chars(&self) -> usize {
        self.0
            .chars()
            .filter(|c| Self::SPECIAL_CHARS.contains(c))
            .count() as usize
    }

    pub fn has_ascii_uppercase(&self) -> bool {
        self.0.chars().any(|c| c.is_ascii_uppercase())
    }

    pub fn count_ascii_uppercase(&self) -> usize {
        self.0.chars().filter(|c| c.is_ascii_uppercase()).count()
    }

    pub fn has_ascii_lowercase(&self) -> bool {
        self.0.chars().any(|c| c.is_ascii_lowercase())
    }

    pub fn count_ascii_lowercase(&self) -> usize {
        self.0.chars().filter(|c| c.is_ascii_lowercase()).count()
    }

    pub fn has_ascii_uppercase_and_lowercase(&self) -> bool {
        self.has_ascii_uppercase() && self.has_ascii_lowercase()
    }
    pub fn count_ascii_uppercase_and_lowercase(&self) -> usize {
        self.count_ascii_uppercase() + self.count_ascii_lowercase()
    }

    pub fn has_ascii_digit(&self) -> bool {
        self.0.chars().any(|c| c.is_ascii_digit())
    }

    pub fn count_ascii_digit(&self) -> usize {
        self.0.chars().filter(|c| c.is_ascii_digit()).count()
    }

    pub fn has_ascii_alphanumeric(&self) -> bool {
        self.0.chars().any(|c| c.is_ascii_alphanumeric())
    }

    pub fn count_ascii_alphanumeric(&self) -> usize {
        self.0.chars().filter(|c| c.is_ascii_alphanumeric()).count()
    }
}

trait StrSealed {}

#[allow(private_bounds)]
pub trait StrValidationExtension: StrSealed {
    fn as_string_validator(&'_ self) -> StringValidator<'_>;
}

impl StrSealed for &str {}

impl StrValidationExtension for &str {
    fn as_string_validator(&'_ self) -> StringValidator<'_> {
        StringValidator::new(self)
    }
}

impl StrSealed for String {}

impl StrValidationExtension for String {
    fn as_string_validator(&'_ self) -> StringValidator<'_> {
        StringValidator::new(&self)
    }
}
