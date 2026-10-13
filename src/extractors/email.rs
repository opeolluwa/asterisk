use sanitizer::prelude::*;
use serde::Deserialize;
use serde_json::Value;
use ts_rs::TS;
use validator::Validate;

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateEmailRequest {
    #[validate(length(min = 1))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub body: String,
    #[validate(email)]
    pub sender_email: String,
    #[sanitizer(trim)]
    #[validate(length(max = 255))]
    pub first_name: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 255))]
    pub last_name: Option<String>,
    #[sanitizer(trim)]
    #[validate(length(max = 64))]
    pub phone: Option<String>,
    #[validate(email)]
    pub recipient_email: String,
    pub tag: Option<String>,
    pub has_attachments: Option<bool>,
    pub data: Option<Value>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Clone, Validate, TS, Sanitizer)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateEmailRequest {
    pub tag: Option<String>,
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListEmailsQuery {
    pub search: Option<String>,
    pub tag: Option<String>,
    pub is_read: Option<bool>,
    pub is_starred: Option<bool>,
    pub has_attachments: Option<bool>,
    pub page: Option<u64>,
    pub page_size: Option<u64>,
}
