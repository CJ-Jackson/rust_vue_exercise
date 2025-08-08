use crate::dependency::Dep;
use crate::html_base::FlashHtmlBuilder;
use crate::user::dependency::UserDep;
use crate::user::flag::{LoginFlag, LogoutFlag};
use crate::user::service::{NoopService, UserLoginService};
use maud::{Markup, html};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::{Flash, Redirect};
use rocket::time::Duration;

#[get("/")]
pub async fn display_user(
    user: UserDep<NoopService>,
    flash_html_builder: Dep<FlashHtmlBuilder>,
) -> Markup {
    let title = if user.1.is_user {
        format!("User: {}", user.1.username)
    } else {
        "Visitor".to_string()
    };

    flash_html_builder
        .attach_title(title.to_string())
        .attach_content(html! {
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
        })
        .build()
}

#[get("/login")]
pub async fn login(
    _noop: UserDep<NoopService, LoginFlag>,
    flash_html_builder: Dep<FlashHtmlBuilder>,
) -> Markup {
    let title = "Login".to_string();
    flash_html_builder
        .attach_title(title.clone())
        .attach_content(html! {
            form method="post" {
                input type="text" name="username" placeholder="Username";
                input type="password" name="password" placeholder="Password";
                button .btn .btn-sky-blue .mt-3 type="submit" { "Login" };
            }
        })
        .build()
}

#[derive(FromForm)]
pub struct UserLoginForm {
    pub username: String,
    pub password: String,
}

#[post("/login", data = "<data>")]
pub async fn login_post<'a>(
    data: Form<UserLoginForm>,
    user_login: UserDep<UserLoginService, LoginFlag>,
    jar: &'a CookieJar<'_>,
) -> Flash<Redirect> {
    let token = user_login
        .0
        .validate_login(data.username.clone(), data.password.clone());
    if let Some(token) = token {
        jar.add(
            Cookie::build(("login-token", token))
                .path("/")
                .max_age(Duration::days(30))
                .build(),
        );
        return Flash::success(Redirect::to(uri!("/user/")), "Login succeeded.");
    }

    Flash::error(Redirect::to(uri!("/user/login/")), "Login failed.")
}

#[get("/logout")]
pub async fn logout<'a>(
    user_login: UserDep<UserLoginService, LogoutFlag>,
    jar: &'a CookieJar<'_>,
) -> Redirect {
    user_login.0.logout();
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
