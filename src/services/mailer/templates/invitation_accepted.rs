use askama::Template;

#[derive(Template)]
#[template(path = "invitation_accepted.html")]
pub struct InvitationAcceptedTemplate<'a> {
    pub dashboard_link: &'a str,
}
