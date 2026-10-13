use serde::{Deserialize, Serialize};
use serde_json::Value;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PaymentMetadata {
    pub order_identifier: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[derive(TypedBuilder)]
pub struct InitializePaymentCommand {
    pub email: String,
    pub amount: u64, // Amount in Units (e.g. kobo for NGN).
    pub currency: String,
    pub callback_url: String,
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializePaymentResponse {
    pub status: bool,
    pub message: String,
    pub checkout_url: Option<String>,
    pub access_code: Option<String>,
    pub reference: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifyPaymentResponse {
    pub succeeded: bool,
    pub reference: String,
    /// In the smallest currency unit, matching [`InitializePaymentCommand::amount`].
    pub amount: Option<u64>,
    /// Uppercase ISO 4217, as Paystack reports it.
    pub currency: Option<String>,
}
