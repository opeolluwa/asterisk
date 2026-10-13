use askama::Template;

/// A message submitted through the public site, forwarded to the team inbox.
///
/// Every field is attacker-controlled: this renders input from an
/// unauthenticated endpoint, where the sender name, both addresses and the
/// subject are whatever the caller typed. Askama escapes all of them and this
/// template deliberately marks none of them `| safe`. Adding `| safe` here would
/// render caller-supplied script into the team's inbox.
#[derive(Template)]
#[template(path = "email_forward.html")]
pub struct EmailForwardTemplate<'a> {
    pub sender_name: &'a str,
    pub sender_email: &'a str,
    /// The address the message was also delivered to. Shown only when it
    /// differs from `sender_email`, so a mismatch between the two is visible
    /// rather than buried.
    pub recipient_email: &'a str,
    /// A callback number, or empty when the visitor gave none. The row is
    /// omitted rather than rendered blank, so the table does not carry an empty
    /// cell on every message that has no number.
    pub phone: &'a str,
    pub subject: &'a str,
    pub body: &'a str,
}
