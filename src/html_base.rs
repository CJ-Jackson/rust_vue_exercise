use crate::dependency::{DependencyError, DependencyFlagData, FromGlobalContext, GlobalContext};
use crate::user::dependency::FromUserContext;
use crate::user::model::UserContext;
use maud::{DOCTYPE, Markup, PreEscaped, html};
use rocket::Request;
use rocket::request::FlashMessage;
use std::cell::RefCell;
use std::sync::Arc;

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

pub struct HtmlCell {
    title: Option<String>,
    content: Option<Markup>,
    head: Option<Markup>,
    footer: Option<Markup>,
}

pub struct ContextHtmlBuilder {
    flash_message: Option<(String, String)>,
    user_context: Option<Arc<UserContext>>,
    data: RefCell<HtmlCell>,
}

impl ContextHtmlBuilder {
    pub fn new(flash_message: Option<(String, String)>) -> Self {
        Self {
            flash_message,
            user_context: None,
            data: RefCell::new(HtmlCell {
                title: None,
                content: None,
                head: None,
                footer: None,
            }),
        }
    }

    pub fn attach_title(&self, title: String) -> &Self {
        self.data.borrow_mut().title = Some(title);
        self
    }

    pub fn attach_content(&self, content: Markup) -> &Self {
        self.data.borrow_mut().content = Some(content);
        self
    }

    pub fn attach_head(&self, head: Markup) -> &Self {
        self.data.borrow_mut().head = Some(head);
        self
    }

    pub fn attach_footer(&self, footer: Markup) -> &Self {
        self.data.borrow_mut().footer = Some(footer);
        self
    }

    pub fn build(&self) -> Markup {
        let parse_flash = self.parse_flash();
        let data = self.data.borrow();
        let title = data.title.clone().unwrap_or_else(|| "Untitled".to_string());
        let content = data.content.clone().unwrap_or_else(|| html! {});
        let head = data.head.clone().unwrap_or_else(|| html! {});
        let footer = data.footer.clone().unwrap_or_else(|| html! {});

        let new_content = html! {
            div .container .main-content .mt-3 .px-7 .py-7 .mx-auto {
                (parse_flash)
                (content)
            }
        };

        HtmlBuilder::new(title, new_content)
            .attach_head(head)
            .attach_footer(footer)
            .build()
    }

    fn parse_flash(&self) -> Markup {
        match &self.flash_message {
            Some((kind, message)) => {
                let kind = kind.as_str();
                match kind {
                    "success" => html! {
                        div .flash-message .flash-message-success {
                            (message)
                        }
                    },
                    "error" => html! {
                        div .flash-message .flash-message-error {
                            (message)
                        }
                    },
                    "warning" => html! {
                        div .flash-message .flash-message-warning {
                            (message)
                        }
                    },
                    _ => html! {},
                }
            }
            _ => html! {},
        }
    }

    fn set_user_context(mut self, user_context: Arc<UserContext>) -> Self {
        self.user_context = Some(user_context);
        self
    }
}

impl FromGlobalContext for ContextHtmlBuilder {
    async fn from_global_context(
        _global_context: &GlobalContext,
        _flag: Arc<DependencyFlagData>,
        request: Option<&Request<'_>>,
    ) -> Result<Self, DependencyError> {
        let request = request.ok_or(DependencyError::NeedsRequest)?;
        let flash_message = request
            .guard::<Option<FlashMessage<'_>>>()
            .await
            .succeeded();

        let flash_message: Option<(String, String)> =
            if let Some(Some(flash_message)) = flash_message {
                Some((
                    flash_message.kind().to_string(),
                    flash_message.message().to_string(),
                ))
            } else {
                None
            };

        Ok(Self::new(flash_message))
    }
}

impl FromUserContext for ContextHtmlBuilder {
    async fn from_user_context<'r>(
        user_context: Arc<UserContext>,
        global_context: &GlobalContext,
        flag: Arc<DependencyFlagData>,
        request: Option<&'r Request<'_>>,
    ) -> Result<Self, DependencyError> {
        let flash_html_builder = Self::from_global_context(global_context, flag, request).await?;
        Ok(flash_html_builder.set_user_context(user_context))
    }
}
