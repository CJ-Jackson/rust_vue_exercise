use maud::{DOCTYPE, Markup, html};

fn html_bootstrap() -> (Markup, Markup) {
    let head = {
        let header_data = (
            "https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/css/bootstrap.min.css",
            "sha384-LN+7fdVzj6u52u30Kp6M/trliBMCMKTyK833zpbD+pXdCLuTusPj697FH4R/5mcr",
        );
        html! {
            link rel="stylesheet" href=(header_data.0) integrity=(header_data.1) crossorigin="anonymous";
        }
    };

    let footer = {
        let footer_data = [
            (
                "https://cdn.jsdelivr.net/npm/@popperjs/core@2.11.8/dist/umd/popper.min.js",
                "sha384-I7E8VVD/ismYTF4hNIPjVp/Zjvgyol6VFvRkX/vR+Vc4jQkC+hVqc2pM8ODewa9r",
            ),
            (
                "https://cdn.jsdelivr.net/npm/bootstrap@5.3.7/dist/js/bootstrap.min.js",
                "sha384-7qAoOXltbVP82dhxHAUje59V5r2YsVfBafyUDxEdApLPmcdhBPg1DKg1ERo0BZlK",
            ),
        ];
        html! {
            @for data in &footer_data {
                script src=(data.0) integrity=(data.1) crossorigin="anonymous" {}
            }
        }
    };

    (head, footer)
}

fn html_vue() -> Markup {
    let data = if cfg!(debug_assertions) {
        (
            "https://cdnjs.cloudflare.com/ajax/libs/vue/3.5.18/vue.global.min.js",
            "sha512-ubvSkfu/RhMm6R8R/oPQnirvwkGzdxP5meB8jnmo6HXxSueW6E6Yt2dsPLmcrQKImFXz/vHaZEGkgv3usohk4A==",
        )
    } else {
        (
            "https://cdnjs.cloudflare.com/ajax/libs/vue/3.5.18/vue.global.prod.min.js",
            "sha512-UyWj9VA5pQrLGoqDUNeT9ciMYGkWexsNHnMQksL9eeiRQyHzD9nIHAj9JzUrbaJ8XRukAkZkQS9EKCnx/Dc8Lw==",
        )
    };
    html! {
        script src=(data.0) integrity=(data.1) crossorigin="anonymous" {}
    }
}

fn html_doc(title: &str, content: Markup, head: Markup, footer: Markup) -> Markup {
    let (bootstrap_head, bootstrap_footer) = html_bootstrap();
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                (bootstrap_head)
                link rel="stylesheet" type="text/css" href="/main.css";
                (head)
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

pub struct HtmlBuilder {
    title: String,
    content: Markup,
    head: Option<Markup>,
    footer: Option<Markup>,
}

impl HtmlBuilder {
    pub fn new(title: String, content: Markup) -> Self {
        Self {
            title,
            content,
            head: None,
            footer: None,
        }
    }

    #[allow(dead_code)]
    pub fn attach_head(mut self, head: Markup) -> Self {
        self.head = Some(head);
        self
    }

    pub fn attach_footer(mut self, footer: Markup) -> Self {
        self.footer = Some(footer);
        self
    }

    pub fn build(self) -> Markup {
        html_doc(
            &self.title,
            self.content,
            self.head.unwrap_or(html! {}),
            self.footer.unwrap_or(html! {}),
        )
    }
}

#[derive(Responder)]
#[response(content_type = "image/x-icon")]
pub struct IcoFile<T>(pub T);
