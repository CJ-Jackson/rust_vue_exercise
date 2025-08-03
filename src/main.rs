pub mod bucket_list;
pub mod config;
pub mod content_type;
pub mod db;
pub mod dep_context;
pub mod error;
pub mod html_base;
pub mod icon;
pub mod utils;

#[macro_use]
extern crate rocket;

use crate::bucket_list::route::BucketListRoute;
use crate::config::get_figment_for_rocket;
use crate::dep_context::DepContext;
use crate::icon::plus_icon;
use crate::utils::{EmbedEtag, EtagCheck};
use content_type::IcoFile;
use html_base::HtmlBuilder;
use maud::{Markup, PreEscaped, html};
use rocket::response::content::RawCss;
use rocket::serde::json::Value;
use rocket::serde::json::serde_json::json;

#[get("/")]
async fn root() -> Markup {
    let title = "Rust Vue Exercise";
    HtmlBuilder::new(
        title.to_string(),
        html! {
            div .container .main-content .mt-3 .px-7 .py-7 .mx-auto {
                h1 .mt-3 { (title) }
                p .mt-3 { "This is Rust Vue Exercise." }
                h2 .mt-3 { "Exercise 1" }
                div #app .mt-3 { "{{ message }}" }
                h2 .mt-3 { "Exercise 2" }
                div #counter .mt-3 {
                    button .btn .btn-sky-blue "@click"="count++" {
                        "Count is: {{ count }}  "
                        (plus_icon())
                    }
                }
                h2 .mt-3 { "Exercise 3" }
                div #array .mt-3 {
                    ul .ul-bullet {
                        li "v-for"="(item) in items" { "{{ item }}" }
                    }
                }
            }
        },
    )
    .attach_footer(root_js())
    .build()
}

fn root_js() -> Markup {
    #[cfg(debug_assertions)]
    let js = include_str!("_asset/root.js");
    #[cfg(not(debug_assertions))]
    let js = include_str!("_asset/root.min.js");
    html! {
        script type="module" { (PreEscaped(js)) }
    }
}

#[get("/array")]
async fn js_array() -> Value {
    json!(["Apple", "Orange", "Banana", "Strawberry", "Mango"])
}

#[get("/favicon.ico")]
async fn favicon(_etag: EtagCheck) -> EmbedEtag<IcoFile<Box<[u8]>>> {
    EmbedEtag::new(IcoFile((*include_bytes!("_asset/favicon.ico")).into()))
}

#[get("/main.css")]
async fn main_css(_etag: EtagCheck) -> EmbedEtag<RawCss<Box<[u8]>>> {
    #[cfg(debug_assertions)]
    let css = *include_bytes!("_asset/main.css");
    #[cfg(not(debug_assertions))]
    let css = *include_bytes!("_asset/main.min.css");
    EmbedEtag::new(RawCss(css.into()))
}

#[launch]
async fn rocket() -> _ {
    rocket::custom(get_figment_for_rocket())
        .attach(DepContext::adhoc())
        .attach(BucketListRoute::adhoc())
        .mount("/", routes![root, js_array, favicon, main_css])
}
