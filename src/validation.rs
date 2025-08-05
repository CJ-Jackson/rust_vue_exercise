use rocket::serde::json::Json;
use serde::Serialize;
use std::collections::HashMap;

#[derive(Serialize)]
struct ValidateErrorItem {
    field_name: String,
    messages: Box<[String]>,
}

impl ValidateErrorItem {
    fn from_vec(field_name: String, messages: Vec<String>) -> Option<Self> {
        if messages.is_empty() {
            return None;
        }

        Some(Self {
            field_name,
            messages: messages.into(),
        })
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
