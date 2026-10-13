use askama::Template;

/// An admin's reply to a complaint, rendered into the customer's inbox.
///
/// `body_html` must already have been through
/// [`crate::services::mailer::sanitize::clean_reply_html`] — the template
/// interpolates it with `| safe`, which is the only place in the crate that
/// lets untrusted markup through unescaped.
#[derive(Template)]
#[template(path = "complaint_reply.html")]
pub struct ComplaintReplyTemplate<'a> {
    pub customer_name: &'a str,
    pub complaint_subject: &'a str,
    pub body_html: &'a str,
}
