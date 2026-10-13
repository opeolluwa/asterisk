use hmac::{Hmac, Mac};
use reqwest::{
    Method,
    header::{HeaderMap, HeaderValue},
};
use secrecy::{ExposeSecret, SecretString};
use serde::{Deserialize, Serialize};
use sha2::Sha512;

use super::adapter::{InitializePaymentCommand, InitializePaymentResponse, VerifyPaymentResponse};
use super::error::PaymentError;
use super::port::PaymentProviderTrait;
use crate::utils::url::encode_path_segment;

#[derive(Clone)]
pub struct PaystackClient {
    private_key: SecretString,
    http_client: reqwest::Client,
    base_url: String,
    verify_base_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaystackInitializeTransactionResponse {
    pub status: bool,
    pub message: String,
    pub data: PaystackInitializeTransactionData,
}

impl From<PaystackInitializeTransactionResponse> for InitializePaymentResponse {
    fn from(val: PaystackInitializeTransactionResponse) -> Self {
        InitializePaymentResponse {
            status: val.status,
            message: val.message,
            checkout_url: Some(val.data.authorization_url),
            access_code: Some(val.data.access_code),
            reference: val.data.reference,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaystackInitializeTransactionData {
    pub authorization_url: String,
    pub access_code: String,
    pub reference: String,
}

/// The shape of `GET /transaction/verify/{reference}`.
///
/// Every field is optional on the wire. Paystack answers a well-formed JSON body
/// for references it does not recognise as well as for ones it does, using
/// `status: false` rather than an error status, so a missing `data` is a normal
/// outcome here and not a parsing failure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaystackVerifyTransactionResponse {
    pub status: bool,
    pub message: String,
    pub data: Option<PaystackVerifyTransactionData>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaystackVerifyTransactionData {
    pub reference: String,
    /// In the smallest currency unit (kobo for NGN), the same unit the initialize
    /// call sends and the same unit an order total is held in.
    pub amount: Option<u64>,
    pub currency: Option<String>,
    pub status: Option<String>,
}

impl From<PaystackVerifyTransactionResponse> for VerifyPaymentResponse {
    fn from(val: PaystackVerifyTransactionResponse) -> Self {
        match val.data {
            Some(data) => VerifyPaymentResponse {
                // Both gates matter: the envelope says the lookup worked, and the
                // transaction inside it says the money arrived. A reference that
                // resolves to an abandoned or failed transaction is not a payment.
                succeeded: val.status && data.status.as_deref() == Some("success"),
                reference: data.reference,
                amount: data.amount,
                currency: data.currency,
            },
            None => VerifyPaymentResponse {
                succeeded: false,
                reference: String::new(),
                amount: None,
                currency: None,
            },
        }
    }
}

impl PaystackClient {
    pub fn new(private_key: &SecretString, base_url: &str, verify_base_url: &str) -> Self {
        Self {
            private_key: private_key.clone(),
            http_client: reqwest::Client::new(),
            base_url: base_url.to_string(),
            verify_base_url: verify_base_url.to_string(),
        }
    }

    /// The bearer header every Paystack call carries.
    fn auth_headers(&self) -> Result<HeaderMap, PaymentError> {
        let mut headers = HeaderMap::new();

        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", self.private_key.expose_secret()))
                .map_err(|err| PaymentError::UnexpectedError(err.to_string()))?,
        );

        headers.insert(
            "Content-Type",
            HeaderValue::from_str("Application/json")
                .map_err(|err| PaymentError::UnexpectedError(err.to_string()))?,
        );

        Ok(headers)
    }

    /// Checks a Paystack webhook's `x-paystack-signature`.
    ///
    /// Paystack signs the raw request body (hex-encoded HMAC-SHA512 with the
    /// secret key), so the body must be hashed exactly as it arrived — already
    /// decoded bytes, not whatever shape the JSON steps into next. `false` for
    /// an empty, malformed or mismatched signature: an event that fails this
    /// check must be dropped, never processed or retried.
    pub fn verify_signature(&self, body: &[u8], signature: &str) -> bool {
        let signature = match hex::decode(signature) {
            Ok(decoded) => decoded,
            Err(_) => return false,
        };

        let mut mac =
            match Hmac::<Sha512>::new_from_slice(self.private_key.expose_secret().as_bytes()) {
                Ok(mac) => mac,
                Err(_) => return false,
            };

        mac.update(body);

        // `verify_slice` compares in constant time, so the length and contents
        // of a wrong signature are not leaked through a timing side channel.
        mac.verify_slice(&signature).is_ok()
    }
}

impl PaymentProviderTrait for PaystackClient {
    async fn initialize_payment(
        &self,
        command: InitializePaymentCommand,
    ) -> Result<InitializePaymentResponse, PaymentError> {
        let response = self
            .http_client
            .request(Method::POST, &self.base_url)
            .headers(self.auth_headers()?)
            .json(&command)
            .send()
            .await
            .map_err(|err| PaymentError::UnexpectedError(err.to_string()))?;

        if !response.status().is_success() {
            return Err(PaymentError::UnexpectedError(
                response.text().await.unwrap_or_default(),
            ));
        }

        let parsed = response
            .json::<PaystackInitializeTransactionResponse>()
            .await
            .map_err(|err| PaymentError::UnexpectedError(err.to_string()))?;

        Ok(parsed.into())
    }

    async fn verify_payment(&self, reference: &str) -> Result<VerifyPaymentResponse, PaymentError> {
        // The reference is provider-generated and URL-safe, but it reaches us
        // from an unauthenticated webhook, so it is encoded rather than
        // interpolated. A reference carrying a path separator would otherwise let
        // a caller steer the request at a different endpoint.
        let url = format!(
            "{}/{}",
            self.verify_base_url.trim_end_matches('/'),
            encode_path_segment(reference)
        );

        let response = self
            .http_client
            .request(Method::GET, &url)
            .headers(self.auth_headers()?)
            .send()
            .await
            .map_err(|err| PaymentError::UnexpectedError(err.to_string()))?;

        // Paystack uses 404 for a reference it has never seen. That is an
        // answer, not a fault, and is reported as such so the caller can treat it
        // as "no such payment" rather than retrying something that will never
        // succeed.
        if response.status() == reqwest::StatusCode::NOT_FOUND {
            return Err(PaymentError::NotFound);
        }

        if !response.status().is_success() {
            return Err(PaymentError::UnexpectedError(
                response.text().await.unwrap_or_default(),
            ));
        }

        let parsed = response
            .json::<PaystackVerifyTransactionResponse>()
            .await
            .map_err(|err| PaymentError::UnexpectedError(err.to_string()))?;

        Ok(parsed.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verify_response_requires_both_gates_to_succeed() {
        let response = PaystackVerifyTransactionResponse {
            status: true,
            message: "Verification successful".into(),
            data: Some(PaystackVerifyTransactionData {
                reference: "ref-1".into(),
                amount: Some(5_000),
                currency: Some("NGN".into()),
                status: Some("success".into()),
            }),
        };

        let verified = VerifyPaymentResponse::from(response);

        assert!(verified.succeeded);
        assert_eq!(verified.reference, "ref-1");
        assert_eq!(verified.amount, Some(5_000));
    }

    #[test]
    fn verify_response_is_not_succeeded_when_the_transaction_failed() {
        // The envelope reports a successful *lookup* while the transaction
        // inside it failed. Treating the envelope flag as the payment result
        // would mark an abandoned checkout as paid.
        let response = PaystackVerifyTransactionResponse {
            status: true,
            message: "Verification successful".into(),
            data: Some(PaystackVerifyTransactionData {
                reference: "ref-1".into(),
                amount: None,
                currency: None,
                status: Some("failed".into()),
            }),
        };

        assert!(!VerifyPaymentResponse::from(response).succeeded);
    }

    #[test]
    fn verify_response_without_data_is_not_succeeded() {
        let response = PaystackVerifyTransactionResponse {
            status: false,
            message: "Transaction reference not found".into(),
            data: None,
        };

        let verified = VerifyPaymentResponse::from(response);

        assert!(!verified.succeeded);
        assert!(verified.amount.is_none());
    }

    fn client(secret: &str) -> PaystackClient {
        PaystackClient::new(
            &SecretString::from(secret.to_string()),
            "https://api.paystack.co/transaction/initialize",
            "https://api.paystack.co/transaction/verify",
        )
    }

    fn sign(body: &[u8], secret: &str) -> String {
        let mut mac = Hmac::<Sha512>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(body);
        hex::encode(mac.finalize().into_bytes())
    }

    #[test]
    fn signature_accepts_a_true_hook() {
        let body = br#"{"event":"charge.success","data":{"reference":"ref-1"}}"#;
        let client = client("secret-key");

        assert!(client.verify_signature(body, &sign(body, "secret-key")));
    }

    #[test]
    fn signature_rejects_a_body_signed_with_a_different_key() {
        let body = br#"{"event":"charge.success","data":{"reference":"ref-1"}}"#;
        let client = client("secret-key");

        assert!(!client.verify_signature(body, &sign(body, "other-key")));
    }

    #[test]
    fn signature_rejects_a_mismatched_body() {
        let body = br#"{"event":"charge.success","data":{"reference":"ref-1"}}"#;
        let client = client("secret-key");

        let tampered = br#"{"event":"charge.success","data":{"reference":"ref-2"}}"#;

        assert!(!client.verify_signature(tampered, &sign(body, "secret-key")));
    }

    #[test]
    fn signature_rejects_empty_and_malformed_headers() {
        let body = br#"{"event":"charge.success"}"#;
        let client = client("secret-key");

        assert!(!client.verify_signature(body, ""));
        assert!(!client.verify_signature(body, "not-hex"));
    }
}
