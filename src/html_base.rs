use maud::{DOCTYPE, Markup, PreEscaped, html};

fn html_import_map() -> Markup {
    #[cfg(debug_assertions)]
    let map = include_str!("_asset/importmap.dev.min.json");
    #[cfg(not(debug_assertions))]
    let map = include_str!("_asset/importmap.prod.min.json");
    html! {
        script type="importmap" { (PreEscaped(map)) }
    }
}

fn html_doc(title: &str, content: Markup, head: Markup, footer: Markup) -> Markup {
    html! {
        (DOCTYPE)
        html {
            head {
                meta charset="utf-8";
                meta name="viewport" content="width=device-width, initial-scale=1";
                title { (title) }
                link rel="stylesheet" type="text/css" href="/main.css";
                (html_import_map())
                (head)
            }
            body {
                (content)
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
