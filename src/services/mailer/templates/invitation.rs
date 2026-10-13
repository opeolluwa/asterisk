use askama::Template;

/// Everything the invitation email needs beyond the recipient's address.
///
/// Grouped into a struct because the call sites (create and resend) assemble
/// the same six values, and positional arguments that long are easy to
/// transpose. Every field is optional: the template degrades to a generic
/// greeting or drops the corresponding line entirely.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct InvitationEmail {
    pub signup_link: String,
    pub invitee_name: String,
    pub inviter_name: String,
    pub role_name: String,
    pub expires_on: String,
}

#[derive(Template)]
#[template(path = "invitation.html")]
pub struct InvitationTemplate {
    pub signup_link: String,
    pub invitee_name: String,
    pub inviter_name: String,
    pub role_name: String,
    pub expires_on: String,
}

impl<'a> From<InvitationEmail> for InvitationTemplate {
    fn from(email: InvitationEmail) -> Self {
        Self {
            signup_link: email.signup_link,
            invitee_name: email.invitee_name,
            inviter_name: email.inviter_name,
            role_name: email.role_name,
            expires_on: email.expires_on,
        }
    }
}
