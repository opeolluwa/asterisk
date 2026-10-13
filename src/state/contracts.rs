use bullmq::Queue;

use crate::config::env::AppConfig;
use crate::errors::app_error::AppError;
use crate::services::imagekit::ImagekitClient;
use crate::services::mailer::{SenderIdentity, queued::QueuedEmailSender};
use crate::services::payment::paystack::PaystackClient;

#[derive(Clone)]
pub struct Contracts {
    pub email: QueuedEmailSender,
    pub imagekit: ImagekitClient,
    pub paystack: PaystackClient,
}

impl Contracts {
    pub fn new(app_config: &AppConfig, email_queue: Queue) -> Result<Self, AppError> {
        // The dispatch layer is generic over `EmailSender` and has no access to
        // the config, so the configured sending identity is handed to the
        // queue-backed sender here, which is the one that runs in the request
        // path.
        let email = QueuedEmailSender::new(
            email_queue,
            SenderIdentity {
                address: app_config.email_from_address.clone(),
                name: app_config.email_from_name.clone(),
            },
        );

        let paystack = PaystackClient::new(
            &app_config.paystack_api_secret,
            &app_config.paystack_base_url,
            &app_config.paystack_verify_base_url,
        );

        let imagekit = ImagekitClient::new(
            &app_config.imagekit_public_key,
            &app_config.imagekit_private_key,
            &app_config.imagekit_upload_url,
        )
        .map_err(|e| AppError::OperationFailed(e.to_string()))?;

        Ok(Self {
            email,
            imagekit,
            paystack,
        })
    }
}
