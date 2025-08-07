use crate::dependency::DependencyFlag;
use crate::html_base::HtmlBuilder;
use crate::user::dependency::UserDep;
use crate::user::flag::{LoginFlag, LogoutFlag};
use crate::user::service::{NoopService, UserLoginService};
use maud::{Markup, html};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::Redirect;

#[get("/")]
pub async fn display_user(user: UserDep<NoopService>) -> Markup {
    let title = if user.1.is_user {
        format!("User: {}", user.1.username)
    } else {
        "Visitor".to_string()
    };

    HtmlBuilder::new(
        title.clone(),
        html! {
            div .container .main-content .mt-3 .px-7 .py-7 .mx-auto {
                h1 .mt-3 { (title) }
                p { "Welcome to the user page!" }
                @if user.1.is_user {
                    p { "You are logged in as a user '" (user.1.username) "'." }
                    p { "You can log out by clicking the button below." }
                    a .btn .btn-sky-blue .mt-3 href="/user/logout" { "Log out" }
                } @else {
                    p { "You are logged in as a visitor." }
                    p { "You can log in as a user by clicking the button below." }
                    a .btn .btn-sky-blue .mt-3 href="/user/login" { "Log in as a user" }
                }
            }
        },
    )
    .build()
}

#[get("/login")]
pub async fn login(_noop: UserDep<NoopService, LoginFlag>) -> Markup {
    let title = "Login".to_string();
    HtmlBuilder::new(
        title,
        html! {
            div .container .main-content .mt-3 .px-7 .py-7 .mx-auto {
                form method="post" {
                    input type="text" name="username" placeholder="Username";
                    button .btn .btn-sky-blue .mt-3 type="submit" { "Login" };
                }
            }
        },
    )
    .build()
}

#[derive(FromForm)]
pub struct UserLoginForm {
    pub username: String,
}

#[post("/login", data = "<data>")]
pub async fn login_post<'a>(
    data: Form<UserLoginForm>,
    user_login: UserDep<UserLoginService, LoginFlag>,
    jar: &'a CookieJar<'_>,
) -> Redirect {
    let token = user_login.0.validate_login(data.username.clone());
    jar.add(
        Cookie::build(("login-username", data.username.clone()))
            .path("/")
            .build(),
    );
    jar.add(Cookie::build(("login-token", token)).path("/").build());
    Redirect::to(uri!("/user"))
}

#[get("/logout")]
pub async fn logout<'a>(
    _noop: UserDep<NoopService, LogoutFlag>,
    jar: &'a CookieJar<'_>,
) -> Redirect {
    jar.remove(Cookie::from("login-username"));
    jar.remove(Cookie::from("login-token"));
    Redirect::to(uri!("/user"))
}

pub struct UserRoute;

impl UserRoute {
    pub fn adhoc() -> AdHoc {
        AdHoc::on_ignite("UserRoute", |r| async {
            r.mount("/user", routes![display_user, login, login_post, logout])
        })
    }
}
