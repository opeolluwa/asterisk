use super::adapter::{InitializePaymentCommand, InitializePaymentResponse, VerifyPaymentResponse};
use super::error::PaymentError;

pub trait PaymentProviderTrait {
    fn initialize_payment(
        &self,
        command: InitializePaymentCommand,
    ) -> impl std::future::Future<Output = Result<InitializePaymentResponse, PaymentError>> + Send;

    /// Asks the provider to confirm a reference, server to server.
    ///
    /// This is what a payment is checked against rather than a webhook body. The
    /// webhook arrives unauthenticated, so the reference in it identifies a
    /// transaction but proves nothing about it; only the provider can say whether
    /// it settled, and for how much.
    fn verify_payment(
        &self,
        reference: &str,
    ) -> impl std::future::Future<Output = Result<VerifyPaymentResponse, PaymentError>> + Send;
}
