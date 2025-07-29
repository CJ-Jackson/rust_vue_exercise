#[macro_use] extern crate rocket;

use maud::{html, Markup, PreEscaped, DOCTYPE};
use rocket::serde::json::serde_json::json;
use rocket::serde::json::Value;

fn html_bootstrap() -> (Markup, Markup) {
    let url = "https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/css/bootstrap.min.css";
    let integrity = "sha384-LN+7fdVzj6u52u30Kp6M/trliBMCMKTyK833zpbD+pXdCLuTusPj697FH4R/5mcr";
    let head = html! {
        link rel="stylesheet" href=(url) integrity=(integrity) crossorigin="anonymous";
    };

    let popper_url = "https://cdn.jsdelivr.net/npm/@popperjs/core@2.11.8/dist/umd/popper.min.js";
    let popper_integrity = "sha384-I7E8VVD/ismYTF4hNIPjVp/Zjvgyol6VFvRkX/vR+Vc4jQkC+hVqc2pM8ODewa9r";
    let url = "https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/js/bootstrap.min.js";
    let integrity = "sha384-7qAoOXltbVP82dhxHAUje59V5r2YsVfBafyUDxEdApLPmcdhBPg1DKg1ERo0BZlK";
    let footer = html! {
        script src=(popper_url) integrity=(popper_integrity) crossorigin="anonymous" {}
        script src=(url) integrity=(integrity) crossorigin="anonymous" {}
    };

    (head, footer)
}

fn html_vue() -> Markup {
    let url = if cfg!(debug_assertions) {
        ("https://cdnjs.cloudflare.com/ajax/libs/vue/3.5.18/vue.global.min.js",
         "sha512-ubvSkfu/RhMm6R8R/oPQnirvwkGzdxP5meB8jnmo6HXxSueW6E6Yt2dsPLmcrQKImFXz/vHaZEGkgv3usohk4A==")
    } else {
        ("https://cdnjs.cloudflare.com/ajax/libs/vue/3.5.18/vue.global.prod.min.js",
         "sha512-UyWj9VA5pQrLGoqDUNeT9ciMYGkWexsNHnMQksL9eeiRQyHzD9nIHAj9JzUrbaJ8XRukAkZkQS9EKCnx/Dc8Lw==")
    };
    html! {
        script src=(url.0) integrity=(url.1) crossorigin="anonymous" {}
    }
}

fn html_doc(title: &str, content: Markup, head: Markup, footer: Markup) -> Markup {
    let (bootstrap_head, bootstrap_footer) = html_bootstrap();
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                title { (title) }
                (head)
                (bootstrap_head)
            }
            body {
                (content)
                (bootstrap_footer)
                (html_vue())
                (footer)
            }
        }
    }
}

struct HtmlBuilder {
    title: String,
    content: Markup,
    head: Option<Markup>,
    footer: Option<Markup>,
}

impl HtmlBuilder {
    fn new(title: String, content: Markup) -> Self {
        Self {
            title,
            content,
            head: None,
            footer: None,
        }
    }

    #[allow(dead_code)]
    fn attach_head(mut self, head: Markup) -> Self {
        self.head = Some(head);
        self
    }

    fn attach_footer(mut self, footer: Markup) -> Self {
        self.footer = Some(footer);
        self
    }

    fn build(self) -> Markup {
        html_doc(&self.title, self.content, self.head.unwrap_or(html!{}), self.footer.unwrap_or(html!{}))
    }
}

#[get("/")]
async fn root() -> Markup {
    let title= "Rust Vue Exercise";
    HtmlBuilder::new(title.to_string(), html! {
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
    }).attach_footer(root_js()).build()
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
