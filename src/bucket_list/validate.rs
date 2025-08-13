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

        let mut check_count = true;
        name.is_empty().then(|| {
            message.push(format!("{} is required", &field_name));
            check_count = false;
        });
        check_count.then(|| {
            let name_count = name.graphemes(true).count();
            (name_count < 5)
                .then(|| message.push(format!("{} must be at least 5 characters", &field_name)));
            (name_count > 20)
                .then(|| message.push(format!("{} must be at most 20 characters", &field_name)));
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

        let mut check_count = true;
        description.is_empty().then(|| {
            message.push(format!("{} is required", &field_name));
            check_count = false;
        });
        check_count.then(|| {
            let description_count = description.graphemes(true).count();
            (description_count < 5)
                .then(|| message.push(format!("{} must be at least 5 characters", &field_name)));
            (description_count > 100)
                .then(|| message.push(format!("{} must be at most 100 characters", &field_name)));
        });

        ValidateErrorItem::from_vec(field_name, message)
            .then_err_report(|i| DescriptionError(i))?;
        Ok(Description(description))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    mod name {
        use super::*;

        #[test]
        fn test_parse_name() {
            let name = Name::parse("Hello".to_string(), None);
            assert!(name.is_ok());
        }

        #[test]
        fn test_parse_name_error_empty_name() {
            let name = Name::parse("".to_string(), None);
            assert!(name.is_err());
        }

        #[test]
        fn test_parse_name_error_name_length_too_short() {
            let name = Name::parse("a".to_string(), None);
            assert!(name.is_err());
        }

        #[test]
        fn test_parse_name_error_name_length_too_long() {
            let name = Name::parse("a".repeat(21), None);
            assert!(name.is_err());
        }
    }

    mod description {
        use super::*;

        #[test]
        fn test_parse_description() {
            let description = Description::parse("Hello".to_string(), None);
            assert!(description.is_ok());
        }

        #[test]
        fn test_parse_description_error_empty_description() {
            let description = Description::parse("".to_string(), None);
            assert!(description.is_err());
        }

        #[test]
        fn test_parse_description_error_description_length_too_short() {
            let description = Description::parse("a".to_string(), None);
            assert!(description.is_err());
        }

        #[test]
        fn test_parse_description_error_description_length_too_long() {
            let description = Description::parse("a".repeat(101), None);
            assert!(description.is_err());
        }
    }
}
