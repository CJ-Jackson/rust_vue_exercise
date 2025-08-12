use error_stack::Report;
use rocket::serde::json::Json;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;

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

#[derive(Responder)]
#[response(status = 422)]
pub struct ValidationErrorMergedResponse(Json<HashMap<String, Box<[ValidateErrorItem]>>>);

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
