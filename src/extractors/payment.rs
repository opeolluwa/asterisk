use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct InitializePaymentRequest {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "order identifier is required"))]
    pub order_identifier: String,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConfirmPaymentRequest {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "order identifier is required"))]
    pub order_identifier: String,
}
