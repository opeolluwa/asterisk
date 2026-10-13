use sanitizer::prelude::*;
use sea_orm::entity::prelude::DateTimeWithTimeZone;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use validator::Validate;

use crate::models::{customer_address, customers};

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreateCustomerRequest {
    #[validate(email(message = "Invalid email format"))]
    #[sanitizer(trim, lower_case)]
    pub email: String,
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub password: String,
    #[validate(length(min = 1))]
    #[sanitizer(trim, lower_case)]
    pub first_name: String,
    #[validate(length(min = 1))]
    #[sanitizer(trim, lower_case)]
    pub last_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct LoginCustomerRequest {
    #[validate(email(message = "Invalid email format"))]
    #[sanitizer(trim, lower_case)]
    pub email: String,
    #[validate(length(min = 1, message = "password cannot be empty"))]
    pub password: String,
}

//TODO: allow customer to change their emails, send ask for confirmation before change
#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct UpdateCustomerRequest {
    #[validate(length(min = 1, message = "First name cannot be empty"))]
    #[sanitizer(trim, lower_case)]
    pub first_name: String,
    #[validate(length(min = 1, message = "Last name cannot be empty"))]
    #[sanitizer(trim, lower_case)]
    pub last_name: String,
    #[validate(length(max = 50, message = "Phone number is too long"))]
    #[sanitizer(trim)]
    pub phone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ConfirmCustomerAccountRequest {
    #[validate(length(min = 1))]
    #[sanitizer(trim)]
    pub otp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ResetCustomerAccountRequest {
    #[validate(length(min = 1, message = "Password cannot be empty"))]
    pub new_password: String,
    #[validate(must_match(other = "new_password", message = "passwords do not match"))]
    pub confirm_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct VerifyCustomerOtpRequest {
    #[validate(length(min = 1))]
    #[sanitizer(trim)]
    pub otp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RequestPasswordResetRequest {
    #[validate(email(message = "Invalid email format"))]
    #[sanitizer(trim, lower_case)]
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct RequestPasswordResetResult {
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CustomerLoginResult {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expiry: i64,
    pub refresh_token_expiry: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CustomerSignUpResult {
    pub token: String,
}

/// A customer as the admin console needs to render it.
///
/// This is projected from `customers` rather than the `SeaORM` model on purpose:
/// the entity carries the bcrypt `password` hash, and serializing it to a client
/// is never what an admin listing wants. Anything sensitive has to be added to
/// the entity deliberately to leak, rather than inherited by accident.
///
/// The console talks in terms of blocking, so `deleted_at` is projected as
/// `blocked` rather than exposing the soft-delete column itself.
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CustomerProfile {
    pub identifier: String,
    pub email: String,
    pub first_name: String,
    pub last_name: String,
    pub phone: Option<String>,
    pub picture: Option<String>,
    pub addresses: Vec<customer_address::Model>,
    pub created_at: DateTimeWithTimeZone,
    pub updated_at: Option<DateTimeWithTimeZone>,
    pub blocked: bool,
    pub completed_onboarding: bool,
}

impl From<(customers::Model, Vec<customer_address::Model>)> for CustomerProfile {
    fn from(
        (customer, customer_addresses): (customers::Model, Vec<customer_address::Model>),
    ) -> Self {
        Self {
            identifier: customer.identifier,
            email: customer.email,
            first_name: customer.first_name.unwrap_or_default(),
            last_name: customer.last_name.clone().unwrap_or_default(),
            phone: customer.phone,
            picture: customer.picture,
            addresses: customer_addresses,
            created_at: customer.created_at,
            updated_at: customer.updated_at,
            blocked: customer.deleted_at.is_some(),
            completed_onboarding: customer.completed_onboarding,
        }
    }
}

impl From<&customers::Model> for CustomerProfile {
    fn from(customer: &customers::Model) -> Self {
        Self {
            identifier: customer.identifier.clone(),
            email: customer.email.clone(),
            first_name: customer.first_name.clone().unwrap_or_default(),
            last_name: customer.last_name.clone().unwrap_or_default(),
            phone: customer.phone.clone(),
            picture: customer.picture.clone(),
            // A list has no room for per-customer address lookups, so the
            // caller decides whether to fill this in.
            addresses: Vec::new(),
            created_at: customer.created_at,
            updated_at: customer.updated_at,
            blocked: customer.deleted_at.is_some(),
            completed_onboarding: customer.completed_onboarding,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CreatePrimaryDeliveryAddress {
    #[validate(length(min = 1, message = "Address line 1 cannot be empty"))]
    #[sanitizer(trim)]
    pub address_line_one: String,
    #[sanitizer(trim)]
    pub address_line_two: Option<String>,
    #[sanitizer(trim)]
    pub local_govt_area: Option<String>,
    #[sanitizer(trim)]
    pub city_or_town: Option<String>,
    #[sanitizer(trim)]
    pub state: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Validate, Sanitizer, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CompleteCustomerOnboardingRequest {
    #[validate(length(min = 1, message = "Phone number cannot be empty"))]
    #[sanitizer(trim)]
    pub phone: String,
    pub address: CreatePrimaryDeliveryAddress,
}
