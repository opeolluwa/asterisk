use std::sync::Arc;

use axum::{Json, extract::State};
use sanitizer::Sanitizer;

use crate::errors::service_error::ServiceError;
use crate::extractors::newsletter::{NewsletterSubscription, SubscribeNewsletterRequest};
use crate::middlewares::validator::ValidatedRequest;
use crate::services::newsletter_services::NewsletterServiceExt;
use crate::state::AppState;

pub async fn subscribe_to_newsletter(
    State(state): State<Arc<AppState>>,
    ValidatedRequest(mut request): ValidatedRequest<SubscribeNewsletterRequest>,
) -> Result<Json<NewsletterSubscription>, ServiceError> {
    request.sanitize();

    if request
        .website
        .as_deref()
        .is_some_and(|trap| !trap.is_empty())
    {
        // Answer as a success so a bot gets no signal that it was filtered.
        // The submitted address is intentionally not logged.
        tracing::info!("newsletter honeypot triggered");

        return Ok(Json(NewsletterSubscription {
            email: request.email,
            subscribed: true,
        }));
    }

    let subscriber = state
        .services
        .newsletter_service
        .subscribe(&request.email)
        .await?;

    Ok(Json(NewsletterSubscription {
        email: subscriber.email,
        subscribed: subscriber.subscribed,
    }))
}
