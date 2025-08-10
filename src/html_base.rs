use crate::dependency::{
    DependencyError, DependencyFlagData, DependencyGlobalContext, FromGlobalContext,
};
use crate::user::dependency::{DependencyUserContext, FromUserContext};
use crate::user::model::UserContext;
use maud::{DOCTYPE, Markup, PreEscaped, html};
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

pub struct NavigationItem {
    name: String,
    url: String,
    tag: String,
}

impl NavigationItem {
    fn navigations() -> Box<[Self]> {
        [
            Self {
                name: "Home".to_string(),
                url: "/".to_string(),
                tag: "home".to_string(),
            },
            Self {
                name: "Bucket List".to_string(),
                url: "/bucket-list/".to_string(),
                tag: "bucket-list".to_string(),
            },
            Self {
                name: "User".to_string(),
                url: "/user/".to_string(),
                tag: "user".to_string(),
            },
        ]
        .into()
    }
}

pub struct HtmlCell {
    title: Option<String>,
    content: Option<Markup>,
    head: Option<Markup>,
    footer: Option<Markup>,
    current_tag: String,
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
                current_tag: "".to_string(),
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

    pub fn set_current_tag(&self, tag: String) -> &Self {
        self.data.borrow_mut().current_tag = tag;
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
            (parse_flash)
            (self.build_navigation())
            div .content-wrapper {
                div .container .main-content {
                    (content)
                }
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

    fn build_navigation(&self) -> Markup {
        let user_context = self.user_context.as_ref();
        html! {
            nav .nav-content {
                span .nav-home {
                    a href="/" { "Rust Vue Exercise, and more" }
                }
                (self.parse_navigation(self.data.borrow().current_tag.clone()))
                @if let Some(user_context) = user_context {
                    span .nav-user {
                        @if user_context.is_user {
                            a href="/user/" { "Hello, " (user_context.username) }
                        } @else {
                            a href="/user/login" { "You're a visitor, click here to login" }
                        }
                    }
                } @else {
                    span .nav-user {
                        a .nav-user href="/user/login" { "Login" }
                    }
                }
            }
        }
    }

    fn parse_navigation(&self, tag: String) -> Markup {
        let mut output = "".to_string();
        for item in NavigationItem::navigations() {
            let html = if item.tag == tag {
                html! {
                    span .nav-item .nav-item-active {
                        a href=(item.url) { (item.name) }
                    }
                }
            } else {
                html! {
                    span .nav-item {
                        a href=(item.url) { (item.name) }
                    }
                }
            };
            output.push_str(html.into_string().as_str());
        }
        PreEscaped(output)
    }
}

impl FromGlobalContext for ContextHtmlBuilder {
    async fn from_global_context(
        dependency_context: &DependencyGlobalContext<'_, '_>,
        _flag: Arc<DependencyFlagData>,
    ) -> Result<Self, DependencyError> {
        let request = dependency_context
            .request
            .ok_or(DependencyError::NeedsRequest)?;
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
    async fn from_user_context(
        dependency_user_context: &DependencyUserContext<'_, '_>,
        flag: Arc<DependencyFlagData>,
    ) -> Result<Self, DependencyError> {
        let flash_html_builder =
            Self::from_global_context(&dependency_user_context.dependency_global_context, flag)
                .await?;
        Ok(flash_html_builder.set_user_context(Arc::clone(&dependency_user_context.user_context)))
    }
}
