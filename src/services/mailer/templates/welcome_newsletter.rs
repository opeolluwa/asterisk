use askama::Template;

#[derive(Template)]
#[template(path = "welcome_newsletter.html")]
pub struct WelcomeNewsletterTemplate<'a> {
    pub name: &'a str,
    pub dashboard_link: &'a str,
}
