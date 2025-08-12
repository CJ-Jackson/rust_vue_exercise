use crate::bucket_list::validate::{Description, Name};
use crate::validation::{ValidationErrorResponse, ValidationErrorsBuilder};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Debug)]
pub struct BucketListItem {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct AddToBucketList {
    pub name: String,
    pub description: String,
}

impl AddToBucketList {
    pub fn to_validated(&self) -> Result<AddToBucketListValidated, ValidationErrorResponse> {
        let mut builder = ValidationErrorsBuilder::new();

        let name = builder
            .add_item_from_trait(Name::parse(self.name.clone(), None))
            .unwrap_or_default();
        let description = builder
            .add_item_from_trait(Description::parse(self.description.clone(), None))
            .unwrap_or_default();

        builder.build_result()?;

        Ok(AddToBucketListValidated { name, description })
    }
}

pub struct AddToBucketListValidated {
    pub name: Name,
    pub description: Description,
}
