use crate::bucket_list::model::{AddToBucketList, BucketListItem};
use crate::bucket_list::repository::{BucketListRepository, BucketListRepositoryError};
use crate::dependency::Dep;
use crate::error::{ErrorOutput, ErrorReportResponse};
use crate::html_base::ContextHtmlBuilder;
use crate::icon::plus_icon;
use crate::user::dependency::UserDep;
use crate::validation::ValidationErrorResponse;
use error_stack::ResultExt;
use maud::{Markup, PreEscaped, html};
use rocket::fairing::AdHoc;
use rocket::serde::json::Json;
use rocket::serde::json::Value;
use rocket::serde::json::serde_json::json;

#[get("/")]
pub async fn main_bucket_list(context_html_builder: UserDep<ContextHtmlBuilder>) -> Markup {
    let title = "Bucket List";
    context_html_builder
        .0
        .attach_title(title.to_string())
        .set_current_tag("bucket-list".to_string())
        .attach_content(html! {
            h1 .mt-3 { (title) }
            div #bucket-list .mt-3 v-cloak {
                div .bucket-list-header {
                    span .bucket-list-col { "ID" }
                    span .bucket-list-col { "Name" }
                    span .bucket-list-col { "Description" }
                    span .bucket-list-col { "Timestamp" }
                }
                div .bucket-list-item "v-for"="item in bucket_list" {
                    span .bucket-list-col { "{{ item.id }}" }
                    span .bucket-list-col { "{{ item.name }}" }
                    span .bucket-list-col { "{{ item.description }}" }
                    span .bucket-list-col { "{{ item.timestamp }}" }
                }
                div .bucket-form .mt-5 {
                    input .bucket-list-col .bucket-form-input
                        type="text" placeholder="Name" "v-model"="input_name";
                    input .bucket-list-col .bucket-form-input
                        type="text" placeholder="Description" "v-model"="input_description";
                    button .bucket-list-col .btn .btn-sky-blue "v-on:click"="addToBucketList" {
                        "Add"
                        (plus_icon())
                    }
                }
                div .bucket-form-error "v-if"="error" {
                    span .bucket-list-col {
                        ul {
                            li "v-for"="message in error.name" { "{{ message }}" }
                        }
                    }
                    span .bucket-list-col {
                        ul {
                            li "v-for"="message in error.description" { "{{ message }}" }
                        }
                    }
                    span .bucket-list-col {}
                }
            }
        })
        .attach_footer(get_bucket_list_js())
        .build()
}

pub fn get_bucket_list_js() -> Markup {
    #[cfg(debug_assertions)]
    let js = include_str!("_asset/bucket_list.js");
    #[cfg(not(debug_assertions))]
    let js = include_str!("_asset/bucket_list.min.js");
    html! {
        script type="module" { (PreEscaped(js)) }
    }
}

#[get("/all")]
pub async fn all_bucket_list(
    repo: Dep<BucketListRepository>,
) -> Result<Json<Box<[BucketListItem]>>, ErrorReportResponse<BucketListRepositoryError>> {
    let items = repo
        .get_all_from_bucket_list()
        .attach(ErrorOutput::Json)
        .map_err(|e| ErrorReportResponse(e))?;

    Ok(Json(items))
}

#[derive(Responder)]
pub enum AddBucketListRouteError {
    Repo(ErrorReportResponse<BucketListRepositoryError>),
    Validate(ValidationErrorResponse),
}

#[post("/add", data = "<data>")]
pub async fn add_bucket_list(
    data: Json<AddToBucketList>,
    repo: Dep<BucketListRepository>,
) -> Result<Value, AddBucketListRouteError> {
    let data = data
        .to_validated()
        .map_err(|e| AddBucketListRouteError::Validate(e))?;

    repo.add_to_bucket_list(&data)
        .attach(ErrorOutput::Json)
        .map_err(|e| AddBucketListRouteError::Repo(ErrorReportResponse(e)))?;

    Ok(json!({"message": "success"}))
}

pub struct BucketListRoute;

impl BucketListRoute {
    pub fn adhoc() -> AdHoc {
        AdHoc::on_ignite("BucketListRoute", |rocket| async {
            rocket.mount(
                "/bucket-list",
                routes![main_bucket_list, all_bucket_list, add_bucket_list],
            )
        })
    }
}
