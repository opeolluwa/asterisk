use sanitizer::prelude::*;
use validator::Validate;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer)]
#[serde(rename_all = "camelCase")]
pub struct SubmitContactFormRequest {
    #[validate(email(message = "Invalid email format"))]
    pub email: String,
    #[sanitizer(trim)]
    #[validate(length(max = 255))]
    pub first_name: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 255))]
    pub last_name: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 64))]
    pub phone: Option<String>,
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub message: String,

    /// Honeypot — expected to be absent or empty. Never validated, never stored,
    /// and never rendered.
    ///
    /// Named differently from the newsletter form's `website` trap on purpose: if
    /// both forms can appear on one page, a single shared trap name is one name a
    /// bot needs to learn and then avoid.
    #[sanitizer(trim)]
    #[serde(default)]
    pub company: Option<String>,
}

#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContactFormReceipt {
    pub received: bool,
}
