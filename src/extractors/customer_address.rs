use sanitizer::prelude::*;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateCustomerAddressRequest {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "address line one is required"))]
    pub address_line_one: String,
    #[sanitizer(trim)]
    pub address_line_two: Option<String>,
    #[sanitizer(trim)]
    pub local_govt_area: Option<String>,
    #[sanitizer(trim)]
    pub city_or_town: Option<String>,
    #[sanitizer(trim)]
    pub state: Option<String>,
    #[serde(default)]
    pub is_primary: bool,
}

#[derive(Deserialize, Serialize, Debug, Clone, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateCustomerAddressCommand {
    #[sanitizer(trim)]
    #[validate(length(min = 1, message = "address line one cannot be empty"))]
    pub address_line_one: Option<String>,
    #[sanitizer(trim)]
    pub address_line_two: Option<String>,
    #[sanitizer(trim)]
    pub local_govt_area: Option<String>,
    #[sanitizer(trim)]
    pub city_or_town: Option<String>,
    #[sanitizer(trim)]
    pub state: Option<String>,
    pub is_primary: Option<bool>,
}
