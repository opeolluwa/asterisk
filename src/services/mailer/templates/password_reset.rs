use askama::Template;

#[derive(Template)]
#[template(path = "password_reset.html")]
pub struct PasswordResetTemplate<'a> {
    pub reset_link: &'a str,
    pub name: &'a str,
}
