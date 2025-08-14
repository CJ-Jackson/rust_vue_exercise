use error_stack::Report;
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
    pub fn as_map(&self) -> HashMap<String, Box<[&ValidateErrorItem]>> {
        let mut map = HashMap::new();
        for item in &self.0.0 {
            match map.get(&item.field_name) {
                None => {
                    map.insert(item.field_name.clone(), vec![item].into_boxed_slice());
                }
                Some(value) => {
                    let mut value_map = value.to_vec();
                    value_map.push(item);
                    map.insert(item.field_name.clone(), value_map.into_boxed_slice());
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
    pub fn as_map(&self) -> HashMap<String, Box<[&ValidateErrorItem]>> {
        let mut map = HashMap::new();
        for (_, items) in &self.0.0 {
            for item in items {
                match map.get(&item.field_name) {
                    None => {
                        map.insert(item.field_name.clone(), vec![item].into_boxed_slice());
                    }
                    Some(value) => {
                        let mut value_map = value.to_vec();
                        value_map.push(item);
                        map.insert(item.field_name.clone(), value_map.into_boxed_slice());
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
