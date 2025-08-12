use crate::html_base::ContextHtmlBuilder;
use crate::user::dependency::UserDep;
use crate::user::flag::{LoginFlag, LogoutFlag};
use crate::user::model::UserRegisterFormValidated;
use crate::user::service::{UserLoginService, UserRegisterService};
use crate::user::validate::{Password, Username};
use crate::validation::{ValidationErrorResponse, ValidationErrorsBuilder};
use maud::{Markup, html};
use rocket::fairing::AdHoc;
use rocket::form::Form;
use rocket::http::{Cookie, CookieJar};
use rocket::response::{Flash, Redirect};
use rocket::time::Duration;

#[get("/")]
pub async fn display_user(context_html_builder: UserDep<ContextHtmlBuilder>) -> Markup {
    let title = if context_html_builder.1.is_user {
        format!("User: {}", context_html_builder.1.username)
    } else {
        "Visitor".to_string()
    };

    context_html_builder
        .0
        .attach_title(title.to_string())
        .set_current_tag("user".to_string())
        .attach_content(html! {
            h1 .mt-3 { (title) }
            p { "Welcome to the user page!" }
            @if context_html_builder.1.is_user {
                p { "You are logged in as a user '" (context_html_builder.1.username) "'." }
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
pub async fn login(context_html_builder: UserDep<ContextHtmlBuilder, LoginFlag>) -> Markup {
    let title = "Login".to_string();
    context_html_builder
        .0
        .attach_title(title.clone())
        .attach_content(html! {
            h1 .mt-3 { (title) }
            form method="post" .form {
                input .form-item type="text" name="username" placeholder="Username";
                input .form-item type="password" name="password" placeholder="Password";
                button .btn .btn-sky-blue .mt-3 type="submit" { "Login" };
            }
            p { "If you don't have an account, you can register by clicking the button below." }
            a .btn .btn-sky-blue .mt-3 href="/user/register/" { "Register" }
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
) -> Flash<Redirect> {
    user_login.0.logout();
    jar.remove(Cookie::from("login-token"));
    Flash::success(Redirect::to(uri!("/user")), "Logout succeeded.")
}

#[get("/register")]
pub async fn register(context_html_builder: UserDep<ContextHtmlBuilder, LoginFlag>) -> Markup {
    let title = "Register".to_string();
    context_html_builder
        .0
        .attach_title(title.clone())
        .attach_content(html! {
            h1 .mt-3 { (title) }
            form method="post" .form {
                input .form-item type="text" name="username" placeholder="Username";
                input .form-item type="password" name="password" placeholder="Password";
                input .form-item type="password" name="password_confirm" placeholder="Confirm password";
                button .btn .btn-sky-blue .mt-3 type="submit" { "Register" };
            }
        })
        .build()
}

#[derive(FromForm)]
pub struct UserRegisterForm {
    pub username: String,
    pub password: String,
    pub password_confirm: String,
}

impl UserRegisterForm {
    pub fn to_validated(&self) -> Result<UserRegisterFormValidated, ValidationErrorResponse> {
        let mut builder = ValidationErrorsBuilder::new();

        let username = builder
            .add_item_from_trait(Username::parse(self.username.clone(), None))
            .unwrap_or_default();
        let password = builder
            .add_item_from_trait(Password::parse(self.password.clone(), None, None))
            .unwrap_or_default();
        let password_confirm = builder
            .add_item_from_trait(Password::parse(
                self.password_confirm.clone(),
                Some("password-confirm".to_string()),
                Some(&password),
            ))
            .unwrap_or_default();

        builder.build_result()?;

        Ok(UserRegisterFormValidated {
            username,
            password,
            password_confirm,
        })
    }
}

#[post("/register", data = "<data>")]
async fn register_post(
    data: Form<UserRegisterForm>,
    user_register_service: UserDep<UserRegisterService, LoginFlag>,
) -> Flash<Redirect> {
    let data = data.to_validated();
    match data {
        Ok(data) => {
            if user_register_service.0.register_user(
                data.username.as_str().to_string(),
                data.password.as_str().to_string(),
            ) {
                Flash::success(Redirect::to(uri!("/user/login")), "Register succeeded.")
            } else {
                Flash::error(Redirect::to(uri!("/user/register")), "Register failed.")
            }
        }
        Err(err) => Flash::error(
            Redirect::to(uri!("/user/register")),
            format!("Registration failed: {}", err.to_string()),
        ),
    }
}

pub struct UserRoute;

impl UserRoute {
    pub fn adhoc() -> AdHoc {
        AdHoc::on_ignite("UserRoute", |r| async {
            r.mount(
                "/user",
                routes![
                    display_user,
                    login,
                    login_post,
                    logout,
                    register,
                    register_post
                ],
            )
        })
    }
}
