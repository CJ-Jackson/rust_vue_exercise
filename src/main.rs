pub mod html_base;

#[macro_use]
extern crate rocket;

use html_base::HtmlBuilder;
use maud::{Markup, PreEscaped, html};
use rocket::serde::json::Value;
use rocket::serde::json::serde_json::json;

#[get("/")]
async fn root() -> Markup {
    let title = "Rust Vue Exercise";
    HtmlBuilder::new(
        title.to_string(),
        html! {
            div .container {
                h1 .mt-3 { (title) }
                p .mt-3 { "This is Rust Vue Exercise." }
                div #app .mt-3 { "{{ message }}" }
                div #counter .mt-3 {
                    button .btn .btn-primary "@click"="count++" {
                        "Count is: {{ count }}"
                    }
                }
                div #array .mt-3 {
                    ul {
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
        script type="text/javascript" { (PreEscaped(js)) }
    }
}

#[get("/array")]
async fn js_array() -> Value {
    json!(["Apple", "Orange", "Banana", "Strawberry", "Mango"])
}

#[get("/favicon.ico")]
async fn favicon() -> Box<[u8]> {
    (*include_bytes!("_asset/favicon.ico")).into()
}

#[launch]
async fn rocket() -> _ {
    rocket::build().mount("/", routes![root, js_array, favicon])
}
