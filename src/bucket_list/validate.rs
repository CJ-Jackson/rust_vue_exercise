use crate::bucket_list::model::AddToBucketList;
use crate::validation::{ValidationErrorResponse, ValidationErrorsBuilder};
use unicode_segmentation::UnicodeSegmentation;

pub fn validate_add_to_bucket_list(
    add_to_bucket_list: &AddToBucketList,
) -> Result<(), ValidationErrorResponse> {
    let mut builder = ValidationErrorsBuilder::new();

    {
        let mut message: Vec<String> = vec![];
        let field_name = "name";
        let subject = add_to_bucket_list.name.clone();
        let subject_count = subject.graphemes(true).count();

        if subject.is_empty() {
            message.push("Name is required".to_string());
        } else if subject_count < 5 {
            message.push("Name must be at least 5 characters".to_string());
        } else if subject_count > 20 {
            message.push("Name must be at most 20 characters".to_string());
        }

        builder.add(field_name.to_string(), message);
    }

    {
        let mut message: Vec<String> = vec![];
        let field_name = "description";
        let subject = add_to_bucket_list.description.clone();
        let subject_count = subject.graphemes(true).count();

        if subject.is_empty() {
            message.push("Description is required".to_string());
        } else if subject_count < 5 {
            message.push("Description must be at least 5 characters".to_string());
        } else if subject_count > 100 {
            message.push("Description must be at most 100 characters".to_string());
        }

        builder.add(field_name.to_string(), message);
    }

    builder.build_result()
}
