pub mod html_base;

#[macro_use]
extern crate rocket;

use crate::html_base::IcoFile;
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
                    button .sky-blue-button "@click"="count++" {
                        "Count is: {{ count }}"
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
async fn favicon() -> IcoFile<Box<[u8]>> {
    IcoFile((*include_bytes!("_asset/favicon.ico")).into())
}

#[get("/main.css")]
async fn main_css() -> RawCss<Box<[u8]>> {
    #[cfg(debug_assertions)]
    let css = *include_bytes!("_asset/main.css");
    #[cfg(not(debug_assertions))]
    let css = *include_bytes!("_asset/main.min.css");
    RawCss(css.into())
}

#[launch]
async fn rocket() -> _ {
    rocket::build().mount("/", routes![root, js_array, favicon, main_css])
}
