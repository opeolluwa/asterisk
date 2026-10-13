use std::sync::Arc;

use axum::{Json, extract::State};
use sanitizer::Sanitizer;
use validator::Validate;

use crate::errors::service_error::ServiceError;
use crate::extractors::contact::{ContactFormReceipt, SubmitContactFormRequest};
use crate::extractors::email::CreateEmailRequest;
use crate::services::emails_service::{CONTACT_FORWARD_ADDRESS, EmailsServiceExt};
use crate::state::AppState;

const CONTACT_TAG: &str = "contact";
fn is_trap(company: Option<&str>) -> bool {
    company.is_some_and(|trap| !trap.trim().is_empty())
}

pub async fn submit_contact_form(
    State(state): State<Arc<AppState>>,
    Json(mut request): Json<SubmitContactFormRequest>,
) -> Result<Json<ContactFormReceipt>, ServiceError> {
    request.sanitize();
    if is_trap(request.company.as_deref()) {
        tracing::info!("contact form honeypot triggered");
        return Ok(Json(ContactFormReceipt { received: true }));
    }

    request.validate()?;

    state
        .services
        .emails_service
        .create_email(&request.into())
        .await?;

    Ok(Json(ContactFormReceipt { received: true }))
}

impl Into<CreateEmailRequest> for SubmitContactFormRequest {
    fn into(self) -> CreateEmailRequest {
        CreateEmailRequest {
            subject: self.subject,
            body: self.message,
            sender_email: self.email,
            first_name: self.first_name,
            last_name: self.last_name,
            phone: self.phone,
            recipient_email: CONTACT_FORWARD_ADDRESS.to_string(),
            tag: Some(CONTACT_TAG.to_string()),
            has_attachments: None,
            data: None,
        }
    }
}
