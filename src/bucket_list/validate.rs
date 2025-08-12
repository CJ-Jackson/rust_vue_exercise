use crate::utils::bools::BoolHelper;
use crate::validation::{OptionValidateErrorItemTrait, ValidateErrorItem, ValidateErrorItemTrait};
use error_stack::Report;
use thiserror::Error;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, Error)]
#[error("Name Error")]
pub struct NameError(ValidateErrorItem);

impl ValidateErrorItemTrait for NameError {
    fn get_validate_error_item(&self) -> Option<ValidateErrorItem> {
        Some(self.0.clone())
    }
}
#[derive(Default)]
pub struct Name(String);

impl Name {
    pub fn parse(name: String, field_name: Option<String>) -> Result<Self, Report<NameError>> {
        let mut message: Vec<String> = vec![];
        let field_name = field_name.unwrap_or("name".to_string());
        let name_count = name.graphemes(true).count();

        name.is_empty().do_call(|| {
            message.push("Name is required".to_string());
        });
        (name_count < 5).do_call(|| {
            message.push("Name must be at least 5 characters".to_string());
        });
        (name_count > 20).do_call(|| {
            message.push("Name must be at most 20 characters".to_string());
        });

        ValidateErrorItem::from_vec(field_name, message).then_err_report(|i| NameError(i))?;
        Ok(Name(name))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[derive(Debug, Error)]
#[error("Description Error")]
pub struct DescriptionError(ValidateErrorItem);

impl ValidateErrorItemTrait for DescriptionError {
    fn get_validate_error_item(&self) -> Option<ValidateErrorItem> {
        Some(self.0.clone())
    }
}

#[derive(Default)]
pub struct Description(String);

impl Description {
    pub fn parse(
        description: String,
        field_name: Option<String>,
    ) -> Result<Self, Report<DescriptionError>> {
        let mut message: Vec<String> = vec![];
        let field_name = field_name.unwrap_or("description".to_string());
        let description_count = description.graphemes(true).count();

        description.is_empty().do_call(|| {
            message.push("Description is required".to_string());
        });
        (description_count < 5).do_call(|| {
            message.push("Description must be at least 5 characters".to_string());
        });
        (description_count > 100).do_call(|| {
            message.push("Description must be at most 100 characters".to_string());
        });

        ValidateErrorItem::from_vec(field_name, message)
            .then_err_report(|i| DescriptionError(i))?;
        Ok(Description(description))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}
