use sanitizer::prelude::*;
use ts_rs::TS;
use validator::Validate;

/// Body of a newsletter subscription.
///
/// `website` is a honeypot: it is hidden from real users in the UI, so any
/// value here means a bot filled it in. The handler answers those as a success
/// without writing to the database, so bots get no signal that they were caught.
#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct SubscribeNewsletterRequest {
    #[sanitizer(trim)]
    #[validate(email(message = "Invalid email format"))]
    pub email: String,

    /// Honeypot — expected to be absent or empty. Never validated.
    #[sanitizer(trim)]
    #[serde(default)]
    pub website: Option<String>,
}

/// Public response shape. Deliberately omits `identifier` — nothing outside the
/// admin needs it, and the endpoint is unauthenticated.
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NewsletterSubscription {
    pub email: String,
    pub subscribed: bool,
}
