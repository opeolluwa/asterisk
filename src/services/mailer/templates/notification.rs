use askama::Template;

/// A notification rendered into an inbox.
///
/// Both fields are escaped by askama rather than marked `| safe`, because they
/// originate from whichever service raised the event and are not markup.
#[derive(Template)]
#[template(path = "notification.html")]
pub struct NotificationTemplate<'a> {
    pub name: &'a str,
    pub title: &'a str,
    pub body: &'a str,
}
