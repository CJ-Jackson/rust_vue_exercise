use crate::html_base::ContextHtmlBuilder;
use crate::user::model::UserRegisterFormValidated;
use crate::user::validate::{Password, Username};
use crate::validation::{
    ValidateErrorItem, ValidationErrorResponse, ValidationErrorsBuilder, ValidationOptionMarkup,
};
use maud::{Markup, html};
use std::collections::HashMap;

#[derive(FromForm, Default, Clone)]
pub struct UserRegisterForm {
    pub username: String,
    pub password: String,
    pub password_confirm: String,
}

impl UserRegisterForm {
    pub fn as_validated(&self) -> Result<UserRegisterFormValidated, ValidationErrorResponse> {
        let mut builder = ValidationErrorsBuilder::new();

        let username = builder
            .add_item_from_trait(Username::parse(self.username.clone(), None))
            .unwrap_or_default();
        let password = builder
            .add_item_from_trait(Password::parse(self.password.clone(), None))
            .unwrap_or_default();
        let password_confirm = builder
            .add_item_from_trait(password.parse_confirm(self.password_confirm.clone(), None))
            .unwrap_or_default();

        builder.build_result()?;

        Ok(UserRegisterFormValidated {
            username,
            password,
            password_confirm,
        })
    }

    pub fn html_form(
        title: String,
        context_html_builder: &ContextHtmlBuilder,
        user_register_form: Option<UserRegisterForm>,
        errors: Option<HashMap<String, ValidateErrorItem>>,
    ) -> Markup {
        let user_register_form = user_register_form.unwrap_or_default();
        let errors = errors.unwrap_or_default();
        context_html_builder
            .attach_title(title.clone())
            .attach_content(html! {
                h1 .mt-3 { (title) }
                form method="post" .form {
                    input .form-item type="text" name="username" placeholder="Username" value=(user_register_form.username);
                    (errors.get("username").as_html())
                    input .form-item type="password" name="password" placeholder="Password";
                    (errors.get("password").as_html())
                    input .form-item type="password" name="password_confirm" placeholder="Confirm password";
                    (errors.get("password_confirm").as_html())
                    button .btn .btn-sky-blue .mt-3 type="submit" { "Register" };
                }
            })
            .build()
    }
}
