use askama::Template;

#[derive(Template)]
#[template(path = "auto_respond.html")]
pub struct AutoRespondTemplate<'a> {
    pub name: &'a str,
}
