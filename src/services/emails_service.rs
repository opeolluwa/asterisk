use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use askama::Template;
use heck::ToTitleCase;
use tokio::sync::RwLock;

use crate::{
    errors::{email_service_error::EmailServiceError, service_error::ServiceError},
    extractors::email::{CreateEmailRequest, UpdateEmailRequest},
    models::emails,
    repositories::email_repository::EmailRepositoryExt,
    services::mailer::{
        EmailContext, EmailSender,
        sanitize::clean_reply_html,
        templates::{
            auto_respond::AutoRespondTemplate,
            complaint_reply::ComplaintReplyTemplate,
            email_forward::EmailForwardTemplate,
            invitation::{InvitationEmail, InvitationTemplate},
            notification::NotificationTemplate,
            otp_code::OtpCodeTemplate,
            password_reset::PasswordResetTemplate,
            transaction::{TransactionEmail, TransactionLineItem, TransactionTemplate},
            welcome::WelcomeTemplate,
            welcome_newsletter::WelcomeNewsletterTemplate,
        },
    },
};

/// Where a message submitted through the public site is forwarded.
///
/// A constant rather than config: it names the team's shared inbox, not a
/// deployment-specific endpoint, and every environment forwards to the same
/// place. `EmailContext::from_address` stays the configured sending identity so
/// the forward still passes the provider's domain-verification check.
pub const CONTACT_FORWARD_ADDRESS: &str = "info@asterisk.app";

/// How many submissions one sender address may make per [`CONTACT_FORWARD_WINDOW`].
///
/// Keyed on the sender address rather than the caller's IP: the address lives in
/// the request body, which [`crate::middlewares::rate_limit::RateLimitLayer`] cannot
/// see, so this bounds one sender address rather than one caller.
///
/// The address is caller-supplied, so rotating addresses still gets through; the
/// per-IP governor on the contact route covers caller-level flooding.
pub const CONTACT_FORWARD_LIMIT: u32 = 5;
pub const CONTACT_FORWARD_WINDOW: Duration = Duration::from_secs(3600);

/// A fixed-window send allowance, keyed on an arbitrary string.
///
/// Deliberately not [`crate::middlewares::rate_limit::RateLimitLayer`]: that one
/// reads the peer address off the request and cannot key on the sender address,
/// which is what bounds abuse of the downstream provider here.
#[derive(Clone)]
struct SendAllowance {
    limit: u32,
    window: Duration,
    state: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
}

impl SendAllowance {
    fn new(limit: u32, window: Duration) -> Self {
        Self {
            limit,
            window,
            state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Counts `key` against its window, or reports the allowance as spent.
    ///
    /// Expired entries are dropped as they are encountered rather than by a
    /// background sweep, so the map only ever holds addresses seen since the
    /// last window expiry. Pruning inside the write lock keeps it bounded
    /// without a task that has to be shut down with the service.
    async fn consume(&self, key: &str) -> Result<(), ServiceError> {
        let now = Instant::now();
        let mut map = self.state.write().await;

        map.retain(|_, (_, started)| now.duration_since(*started) < self.window);

        let entry = map.entry(key.to_string()).or_insert((0, now));

        if entry.0 >= self.limit {
            return Err(ServiceError::TooManyRequests(format!(
                "at most {} messages per {} minutes",
                self.limit,
                self.window.as_secs() / 60
            )));
        }

        entry.0 += 1;

        Ok(())
    }
}

#[derive(Clone)]
pub struct EmailsService<R: EmailRepositoryExt, E: EmailSender> {
    repo: R,
    email_sender: E,
    allowance: SendAllowance,
}

impl<R: EmailRepositoryExt, E: EmailSender> EmailsService<R, E> {
    pub fn new(repo: R, email_sender: E) -> Self {
        Self {
            repo,
            email_sender,
            allowance: SendAllowance::new(CONTACT_FORWARD_LIMIT, CONTACT_FORWARD_WINDOW),
        }
    }
}

fn recipient_name(name: &str) -> String {
    let name = name.trim();

    if name.is_empty() {
        "there".to_string()
    } else {
        name.to_title_case()
    }
}

/// A readable name for a message's sender, derived from the address when the
/// sender typed no `first_name`: `paula.okoye@…` becomes "Paula Okoye".
///
/// A guess — `ps145@…` reads as "Ps145" — which is why [`submitter_name`] prefers
/// what the sender typed. Falls back to the whole address when the local part has
/// nothing to title-case; the full address always appears alongside this, so it is
/// presentation only and never the sole identification of who wrote.
fn sender_display_name(address: &str) -> String {
    let local_part = address.split('@').next().unwrap_or(address).trim();

    let words: Vec<&str> = local_part
        .split(['.', '_', '-', '+'])
        .filter(|part| part.chars().any(char::is_alphabetic))
        .collect();

    if words.is_empty() {
        address.trim().to_string()
    } else {
        words.join(" ").to_title_case()
    }
}

/// The name a submission should be attributed to: what the sender typed, falling
/// back to [`sender_display_name`] when the form captured none. `None`, an empty
/// string and whitespace all mean the same thing here.
///
/// Only the usable halves are joined, so a first name with a blank surname does
/// not come out as "Paula ". The joined value is used verbatim rather than
/// title-cased — the text is what a visitor typed, and `to_title_case` would turn
/// "McDonald" into "Mcdonald" and "van der Berg" into "Van Der Berg".
fn submitter_name(first_name: Option<&str>, last_name: Option<&str>, address: &str) -> String {
    let submitted = [first_name, last_name]
        .into_iter()
        .flatten()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>();

    if submitted.is_empty() {
        sender_display_name(address)
    } else {
        submitted.join(" ")
    }
}

/// The name to greet the sender by, from `first_name` only.
///
/// A deliberate asymmetry with [`submitter_name`]: the forward credits the full
/// name because the team is identifying a stranger, while the acknowledgement
/// addresses someone reading their own first name back, and a surname in a
/// greeting can be wrong in ways a first name is not.
///
/// [`sender_display_name`] falls back to the whole address, which identifies who
/// wrote in an inbox table but renders "Hi 12345@example.com," in a greeting, so
/// that case is swapped for the generic greeting other dispatch paths use.
fn submitter_greeting(first_name: Option<&str>, address: &str) -> String {
    let name = submitter_name(first_name, None, address);

    if name == address.trim() {
        recipient_name("")
    } else {
        recipient_name(&name)
    }
}

pub trait EmailDispatchService {
    async fn send_account_created_email(
        &self,
        to_address: &str,
        to_name: &str,
        dashboard_link: &str,
    ) -> Result<(), EmailServiceError>;

    async fn send_password_reset_email(
        &self,
        to_address: &str,
        to_name: &str,
        reset_link: &str,
    ) -> Result<(), EmailServiceError>;

    async fn send_otp_email(
        &self,
        to_address: &str,
        to_name: &str,
        otp_code: &str,
    ) -> Result<(), EmailServiceError>;

    async fn send_invitation_email(
        &self,
        to_address: &str,
        invite: &InvitationEmail,
    ) -> Result<(), EmailServiceError>;

    /// Confirms a newsletter signup.
    ///
    /// `to_name` is the display name to greet them by. A newsletter signup
    /// captures an email address and nothing else, so callers usually have no
    /// name to pass and the empty case falls back to a generic greeting.
    async fn send_newsletter_welcome_email(
        &self,
        to_address: &str,
        to_name: &str,
        explore_link: &str,
    ) -> Result<(), EmailServiceError>;

    /// Sends an admin's reply to a complaint.
    ///
    /// `body` is the stored reply HTML. It is sanitized here as well as on the
    /// write path: `complaint_reply.html` interpolates it with `| safe`, and a
    /// row edited outside the service (a backfill, a manual fix in SQL) would
    /// otherwise reach a customer inbox unsanitized.
    async fn send_complaint_reply_email(
        &self,
        to_address: &str,
        to_name: &str,
        complaint_subject: &str,
        body: &str,
    ) -> Result<(), EmailServiceError>;

    /// Sends a notification raised by a service.
    ///
    /// `notification_delivery_identifier` rides along so the worker can close the
    /// `notification_deliveries` row this message belongs to. It is `None` for
    /// mail that is not the email rendering of a notification.
    async fn send_notification_email(
        &self,
        to_address: &str,
        to_name: &str,
        title: &str,
        body: &str,
        notification_delivery_identifier: Option<&str>,
    ) -> Result<(), EmailServiceError>;

    /// Sends a customer the breakdown of a checkout they just placed.
    ///
    /// This is the customer-facing half of an order: the admins are told by an
    /// in-app notification, and the buyer is told here. It is not the email
    /// rendering of a notification, so it carries no delivery identifier.
    async fn send_transaction_email(
        &self,
        to_address: &str,
        to_name: &str,
        transaction: &TransactionEmail,
    ) -> Result<(), EmailServiceError>;

    /// Acknowledges a public submission to whoever sent it.
    ///
    /// Addressed to `sender_email`, so this reaches the person who filled in the
    /// form. The body is fixed copy and deliberately omits the caller's subject
    /// and body, so an acknowledgement cannot quote text back at a third party or
    /// launder a subject line into a second message.
    async fn send_submitted_message_autorespond(
        &self,
        to_address: &str,
        to_name: &str,
    ) -> Result<(), EmailServiceError>;

    /// Forwards a public submission to [`CONTACT_FORWARD_ADDRESS`], crediting it
    /// to whoever sent it.
    ///
    /// Not fire-and-forget: a caller that asked to send a message should be told
    /// when it did not go out. `recipient_email` is passed through for the
    /// template to surface a mismatch but is never a destination.
    ///
    /// `phone` is passed as `""` when absent, following `WelcomeTemplate`'s
    /// convention for a missing name; the template omits the row rather than
    /// rendering an empty cell.
    ///
    /// Every parameter is a `&str` and easy to transpose, so the order is name,
    /// address, submitted-recipient, phone, subject, body — the order they appear
    /// in the template's own header table.
    async fn send_submitted_message_forward(
        &self,
        sender_name: &str,
        sender_email: &str,
        recipient_email: &str,
        phone: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EmailServiceError>;
}

impl<E: EmailSender> EmailDispatchService for E {
    async fn send_account_created_email(
        &self,
        to_address: &str,
        to_name: &str,
        dashboard_link: &str,
    ) -> Result<(), EmailServiceError> {
        let to_name = to_name.to_title_case();
        let html_body = WelcomeTemplate {
            name: &to_name,
            dashboard_link,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render welcome email: {}", err);
            EmailServiceError::OperationFailed("failed to render welcome email".to_string())
        })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            subject: "Welcome to asterisk".to_string(),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_password_reset_email(
        &self,
        to_address: &str,
        to_name: &str,
        reset_link: &str,
    ) -> Result<(), EmailServiceError> {
        let to_name = to_name.to_title_case();
        let html_body = PasswordResetTemplate {
            reset_link,
            name: &to_name,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render password reset email: {}", err);
            EmailServiceError::OperationFailed("failed to render password reset email".to_string())
        })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            subject: "Password Reset".to_string(),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_otp_email(
        &self,
        to_address: &str,
        to_name: &str,
        otp_code: &str,
    ) -> Result<(), EmailServiceError> {
        let to_name = to_name.to_title_case();
        let html_body = OtpCodeTemplate {
            name: &to_name,
            code: otp_code,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render OTP email: {}", err);
            EmailServiceError::OperationFailed("failed to render OTP email".to_string())
        })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            subject: "Your verification code".to_string(),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_invitation_email(
        &self,
        to_address: &str,
        invite: &InvitationEmail,
    ) -> Result<(), EmailServiceError> {
        let html_body = InvitationTemplate::from(invite.clone())
            .render()
            .map_err(|err| {
                tracing::error!("Failed to render invitation email: {}", err);
                EmailServiceError::OperationFailed("failed to render invitation email".to_string())
            })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name: recipient_name(&invite.invitee_name),
            subject: "You're invited to asterisk".to_string(),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_complaint_reply_email(
        &self,
        to_address: &str,
        to_name: &str,
        complaint_subject: &str,
        body: &str,
    ) -> Result<(), EmailServiceError> {
        let to_name = recipient_name(to_name);
        let body_html = clean_reply_html(body);
        let html_body = ComplaintReplyTemplate {
            customer_name: &to_name,
            complaint_subject,
            body_html: &body_html,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render complaint reply email: {}", err);
            EmailServiceError::OperationFailed("failed to render complaint reply email".to_string())
        })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            subject: format!("Re: {complaint_subject}"),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_notification_email(
        &self,
        to_address: &str,
        to_name: &str,
        title: &str,
        body: &str,
        notification_delivery_identifier: Option<&str>,
    ) -> Result<(), EmailServiceError> {
        let to_name = recipient_name(to_name);
        let html_body = NotificationTemplate {
            name: &to_name,
            title,
            body,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render notification email: {}", err);
            EmailServiceError::OperationFailed("failed to render notification email".to_string())
        })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            subject: title.to_string(),
            html_body,
            notification_delivery_identifier: notification_delivery_identifier.map(str::to_string),
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_transaction_email(
        &self,
        to_address: &str,
        to_name: &str,
        transaction: &TransactionEmail,
    ) -> Result<(), EmailServiceError> {
        let to_name = recipient_name(to_name);

        let items: Vec<TransactionLineItem> = transaction
            .items
            .iter()
            .map(|item| TransactionLineItem {
                description: &item.description,
                quantity: &item.quantity,
                amount: &item.amount,
            })
            .collect();

        let html_body = TransactionTemplate {
            name: &to_name,
            transaction_reference: &transaction.transaction_reference,
            items: &items,
            total: &transaction.total,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render transaction email: {}", err);
            EmailServiceError::OperationFailed("failed to render transaction email".to_string())
        })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            subject: format!("Transaction {}", transaction.transaction_reference),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_submitted_message_autorespond(
        &self,
        to_address: &str,
        to_name: &str,
    ) -> Result<(), EmailServiceError> {
        let to_name = recipient_name(to_name);
        let html_body = AutoRespondTemplate { name: &to_name }
            .render()
            .map_err(|err| {
                tracing::error!("Failed to render submission autorespond: {}", err);
                EmailServiceError::OperationFailed(
                    "failed to render submission autorespond".to_string(),
                )
            })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            // Fixed rather than reusing the submitted subject: an acknowledgement
            // that quoted the caller's subject back would put attacker-chosen text
            // in the subject line of a mail from our domain.
            subject: "We received your message".to_string(),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_submitted_message_forward(
        &self,
        sender_name: &str,
        sender_email: &str,
        recipient_email: &str,
        phone: &str,
        subject: &str,
        body: &str,
    ) -> Result<(), EmailServiceError> {
        let html_body = EmailForwardTemplate {
            sender_name,
            sender_email,
            recipient_email,
            phone,
            subject,
            body,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render submitted message forward: {}", err);
            EmailServiceError::OperationFailed(
                "failed to render submitted message forward".to_string(),
            )
        })?;

        let message = EmailContext {
            to_address: CONTACT_FORWARD_ADDRESS.to_string(),
            to_name: "asterisk Info".to_string(),
            // Prefixed rather than reusing the submitted subject bare: the inbox
            // will hold many of these, and an unprefixed subject makes them
            // indistinguishable from mail addressed to us directly.
            subject: format!("[Contact form] {subject}"),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }

    async fn send_newsletter_welcome_email(
        &self,
        to_address: &str,
        to_name: &str,
        explore_link: &str,
    ) -> Result<(), EmailServiceError> {
        let to_name = recipient_name(to_name);
        let html_body = WelcomeNewsletterTemplate {
            name: &to_name,
            dashboard_link: explore_link,
        }
        .render()
        .map_err(|err| {
            tracing::error!("Failed to render newsletter welcome email: {}", err);
            EmailServiceError::OperationFailed(
                "failed to render newsletter welcome email".to_string(),
            )
        })?;

        let message = EmailContext {
            to_address: to_address.to_string(),
            to_name,
            subject: "You're on the asterisk newsletter".to_string(),
            html_body,
            ..EmailContext::default()
        };

        self.send_email(message).await
    }
}

pub trait EmailsServiceExt {
    async fn create_email(
        &self,
        command: &CreateEmailRequest,
    ) -> Result<emails::Model, ServiceError>;

    async fn find_email_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<emails::Model, ServiceError>;

    async fn find_all_emails(&self) -> Result<Vec<emails::Model>, ServiceError>;

    async fn find_all_emails_filtered(
        &self,
        filter: &crate::repositories::email_repository::EmailFilter,
    ) -> Result<(Vec<emails::Model>, u64), ServiceError>;

    async fn find_emails_by_tag(&self, tag: &str) -> Result<Vec<emails::Model>, ServiceError>;

    async fn find_starred_emails(&self) -> Result<Vec<emails::Model>, ServiceError>;

    async fn find_unread_emails(&self) -> Result<Vec<emails::Model>, ServiceError>;

    async fn update_email(
        &self,
        identifier: &str,
        command: &UpdateEmailRequest,
    ) -> Result<emails::Model, ServiceError>;

    async fn mark_email_as_read(&self, identifier: &str) -> Result<emails::Model, ServiceError>;

    async fn delete_email(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn count_emails(&self) -> Result<i64, ServiceError>;

    async fn count_unread_emails(&self) -> Result<i64, ServiceError>;
}

impl<R: EmailRepositoryExt + Send + Sync, E: EmailSender + Send + Sync> EmailsServiceExt
    for EmailsService<R, E>
{
    /// Records a submitted message, forwards it to the team, and acknowledges it
    /// to the sender.
    ///
    /// Three effects, in this order, and the ordering is load-bearing:
    ///
    /// 1. The sender address is charged against [`CONTACT_FORWARD_LIMIT`] *before*
    ///    anything is written or sent, so a throttled caller costs no database
    ///    write and no queue slot.
    /// 2. The row is written before either delivery, so a send failure still
    ///    leaves a record of what was attempted.
    /// 3. The forward goes to [`CONTACT_FORWARD_ADDRESS`] before the
    ///    acknowledgement, so a failed courtesy cannot starve the delivery the
    ///    endpoint exists for.
    ///
    /// Steps 2 and 3 are not atomic and cannot be made so: the row is committed to
    /// Postgres while the deliveries are queued in Redis. A send failure therefore
    /// returns an error with the row already persisted, and the two deliveries can
    /// diverge. The identifier is logged on that path so they can be reconciled.
    ///
    /// The acknowledgement is the one delivery to a caller-supplied address, so this
    /// will mail an arbitrary address on request. What bounds it: the allowance
    /// above, applied per address; a fixed body that quotes none of the caller's
    /// text; and a fixed subject. Rotating the address defeats the allowance, so
    /// the real bound is the guard on the GraphQL mutation.
    async fn create_email(
        &self,
        command: &CreateEmailRequest,
    ) -> Result<emails::Model, ServiceError> {
        self.allowance.consume(&command.sender_email).await?;

        let sender_name = submitter_name(
            command.first_name.as_deref(),
            command.last_name.as_deref(),
            &command.sender_email,
        );
        let model = self.repo.create_email(command).await?;

        if let Err(err) = self
            .email_sender
            .send_submitted_message_forward(
                &sender_name,
                &command.sender_email,
                &command.recipient_email,
                command.phone.as_deref().unwrap_or(""),
                &command.subject,
                &command.body,
            )
            .await
        {
            tracing::error!(
                identifier = %model.identifier,
                "failed to forward submitted message to {CONTACT_FORWARD_ADDRESS}: {err}"
            );

            return Err(err.into());
        }

        if let Err(err) = self
            .email_sender
            .send_submitted_message_autorespond(
                &command.sender_email,
                &submitter_greeting(command.first_name.as_deref(), &command.sender_email),
            )
            .await
        {
            tracing::error!(
                identifier = %model.identifier,
                recipient = %command.sender_email,
                "failed to acknowledge submitted message to its sender: {err}"
            );

            return Err(err.into());
        }

        Ok(model)
    }

    async fn find_email_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<emails::Model, ServiceError> {
        Ok(self.repo.find_email_by_identifier(identifier).await?)
    }

    async fn find_all_emails(&self) -> Result<Vec<emails::Model>, ServiceError> {
        Ok(self.repo.find_all_emails().await?)
    }

    async fn find_all_emails_filtered(
        &self,
        filter: &crate::repositories::email_repository::EmailFilter,
    ) -> Result<(Vec<emails::Model>, u64), ServiceError> {
        Ok(self.repo.find_all_emails_filtered(filter).await?)
    }

    async fn find_emails_by_tag(&self, tag: &str) -> Result<Vec<emails::Model>, ServiceError> {
        Ok(self.repo.find_emails_by_tag(tag).await?)
    }

    async fn find_starred_emails(&self) -> Result<Vec<emails::Model>, ServiceError> {
        Ok(self.repo.find_starred_emails().await?)
    }

    async fn find_unread_emails(&self) -> Result<Vec<emails::Model>, ServiceError> {
        Ok(self.repo.find_unread_emails().await?)
    }

    async fn update_email(
        &self,
        identifier: &str,
        command: &UpdateEmailRequest,
    ) -> Result<emails::Model, ServiceError> {
        Ok(self.repo.update_email(identifier, command).await?)
    }

    async fn mark_email_as_read(&self, identifier: &str) -> Result<emails::Model, ServiceError> {
        Ok(self.repo.mark_as_read(identifier).await?)
    }

    async fn delete_email(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self.repo.delete_email(identifier).await?)
    }

    async fn count_emails(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_emails().await?)
    }

    async fn count_unread_emails(&self) -> Result<i64, ServiceError> {
        Ok(self.repo.count_unread_emails().await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::repositories::email_repository::MockEmailRepositoryExt;
    use crate::services::mailer::MockEmailSender;
    use sea_orm::sqlx::types::chrono::Utc;
    use std::sync::{
        Arc, Mutex as StdMutex,
        atomic::{AtomicUsize, Ordering},
    };

    /// The dispatch layer has no config, so every message it builds inherits its
    /// identity from the default. Asserted on its own because a derived
    /// `Default` would quietly leave `from_address` blank, and the provider
    /// rejects that only once the job is already in the queue.
    #[test]
    fn the_default_context_carries_the_default_sender_identity() {
        let message = EmailContext::default();

        assert_eq!(message.from_address, "accounts@asterisk.app");
        assert_eq!(message.from_name, "asterisk Accounts");
    }

    #[tokio::test]
    async fn send_account_created_email_builds_and_sends() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            assert_eq!(message.to_address, "user@test.com");
            assert_eq!(message.to_name, "John Doe");
            assert_eq!(message.from_name, "asterisk Accounts");
            assert_eq!(message.subject, "Welcome to asterisk");
            assert!(
                message
                    .html_body
                    .contains("http://localhost:3000/dashboard")
            );
            Ok(())
        });

        let result = sender
            .send_account_created_email(
                "user@test.com",
                "john doe",
                "http://localhost:3000/dashboard",
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_password_reset_email_builds_and_sends() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            assert_eq!(message.to_address, "user@test.com");
            assert_eq!(message.to_name, "Mr White");
            assert_eq!(message.from_name, "asterisk Accounts");
            assert_eq!(message.subject, "Password Reset");
            assert!(message.html_body.contains("https://www.example.com/reset"));
            Ok(())
        });

        let result = sender
            .send_password_reset_email(
                "user@test.com",
                "mr. white",
                "https://www.example.com/reset",
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_invitation_email_builds_and_sends() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            assert_eq!(message.to_address, "invite@test.com");
            assert_eq!(message.to_name, "Ada");
            assert_eq!(message.from_name, "asterisk Accounts");
            assert_eq!(message.subject, "You're invited to asterisk");
            let body = &message.html_body;
            // The greeting, the credit, the role, and the link have to survive
            // rendering — these are the whole point of the enrichment.
            assert!(body.contains("Hi Ada,"), "greeting missing: {body}");
            assert!(
                body.contains("Ngozi Mensah</strong> has invited you"),
                "inviter credit missing: {body}"
            );
            assert!(body.contains("Super Admin"), "role missing: {body}");
            assert!(
                body.contains("http://localhost:5600/auth/signup?token=jwt-token-123"),
                "link missing: {body}"
            );
            // Counted rather than checked once: a bare assertion only proves the
            // link is somewhere, not that the copy-paste fallback is reachable.
            assert_eq!(
                body.matches("http://localhost:5600/auth/signup?token=jwt-token-123")
                    .count(),
                2,
                "expected the link in both the button and the raw fallback: {body}"
            );
            Ok(())
        });

        let result = sender
            .send_invitation_email(
                "invite@test.com",
                &InvitationEmail {
                    signup_link: "http://localhost:5600/auth/signup?token=jwt-token-123"
                        .to_string(),
                    invitee_name: "Ada".to_string(),
                    inviter_name: "Ngozi Mensah".to_string(),
                    role_name: "Super Admin".to_string(),
                    expires_on: "4 October 2026".to_string(),
                },
            )
            .await;
        assert!(result.is_ok());
    }

    /// The minimum viable invitation: no first name, no inviter, no role. Every
    /// optional branch has to collapse to something readable rather than
    /// printing "Hi ," or "  has invited you".
    #[tokio::test]
    async fn send_invitation_email_degrades_without_optional_details() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            let body = &message.html_body;
            assert_eq!(message.to_name, "there");
            assert!(body.contains("Hi there,"), "greeting missing: {body}");
            assert!(!body.contains("Hi ,"), "empty greeting: {body}");
            assert!(
                body.contains("You've been invited to join the team"),
                "fallback opener missing: {body}"
            );
            assert!(
                !body.contains("has invited you"),
                "inviter credit rendered with no inviter: {body}"
            );
            assert!(
                body.contains("expires after 7 days"),
                "fallback expiry missing: {body}"
            );
            Ok(())
        });

        let result = sender
            .send_invitation_email(
                "invite@test.com",
                &InvitationEmail {
                    signup_link: "http://localhost:5600/auth/signup?token=abc".to_string(),
                    ..Default::default()
                },
            )
            .await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn send_newsletter_welcome_email_builds_and_sends() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            assert_eq!(message.to_address, "reader@test.com");
            // No name was captured at signup, so the greeting falls back.
            assert_eq!(message.to_name, "there");
            assert_eq!(message.from_name, "asterisk Accounts");
            assert_eq!(message.subject, "You're on the asterisk newsletter");
            let body = &message.html_body;
            assert!(body.contains("Hi there,"), "greeting missing: {body}");
            assert!(
                body.contains("Thanks for subscribing"),
                "confirmation missing: {body}"
            );
            assert!(
                body.contains("https://asterisk.app/newsletter"),
                "explore link missing: {body}"
            );
            // The copy promises a reply-to unsubscribe, so the address has to be
            // one a person can actually answer.
            assert!(
                body.contains("reply to this email"),
                "opt-out copy missing: {body}"
            );
            Ok(())
        });

        let result = sender
            .send_newsletter_welcome_email(
                "reader@test.com",
                "",
                "https://asterisk.app/newsletter",
            )
            .await;
        assert!(result.is_ok());
    }

    /// A newsletter signup has no name to pass, but a caller that somehow has
    /// one should still get a titled greeting rather than a raw one.
    #[tokio::test]
    async fn send_newsletter_welcome_email_titles_a_supplied_name() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            assert_eq!(message.to_name, "Paula Okoye");
            assert!(message.html_body.contains("Hi Paula Okoye,"));
            Ok(())
        });

        let result = sender
            .send_newsletter_welcome_email(
                "reader@test.com",
                "paula okoye",
                "https://asterisk.app/newsletter",
            )
            .await;
        assert!(result.is_ok());
    }

    fn test_email() -> emails::Model {
        emails::Model {
            identifier: "em-001".to_string(),
            subject: "Test Subject".to_string(),
            body: "Hello world".to_string(),
            sender_email: "sender@example.com".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
            recipient_email: "recipient@example.com".to_string(),
            date_sent: Utc::now().naive_utc().and_utc().into(),
            tag: Some("work".to_string()),
            is_read: false,
            is_starred: false,
            has_attachments: false,
            data: None,
            user_identifier: Some("user-001".to_string()),
            created_at: Utc::now().naive_utc().and_utc().into(),
            updated_at: None,
        }
    }

    fn create_command() -> crate::extractors::email::CreateEmailRequest {
        crate::extractors::email::CreateEmailRequest {
            subject: "Test Subject".to_string(),
            body: "Hello world".to_string(),
            sender_email: "sender@example.com".to_string(),
            first_name: None,
            last_name: None,
            phone: None,
            recipient_email: "recipient@example.com".to_string(),
            tag: Some("work".to_string()),
            has_attachments: None,
            data: None,
        }
    }

    #[tokio::test]
    async fn create_email_returns_model() {
        let mut repo = MockEmailRepositoryExt::new();
        let email = test_email();
        repo.expect_create_email()
            .returning(move |_| Ok(email.clone()));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().times(2).returning(|_| Ok(()));

        let service = EmailsService::new(repo, sender);

        let result = service.create_email(&create_command()).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().subject, "Test Subject");
    }

    #[tokio::test]
    async fn find_email_by_identifier() {
        let mut repo = MockEmailRepositoryExt::new();
        let email = test_email();
        repo.expect_find_email_by_identifier()
            .returning(move |_| Ok(email.clone()));
        let service = EmailsService::new(repo, MockEmailSender::new());

        assert!(service.find_email_by_identifier("em-001").await.is_ok());
    }

    #[tokio::test]
    async fn find_all_emails() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_find_all_emails()
            .returning(|| Ok(vec![test_email()]));
        let service = EmailsService::new(repo, MockEmailSender::new());

        assert_eq!(service.find_all_emails().await.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn count_emails() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_count_emails().returning(|| Ok(10));
        let service = EmailsService::new(repo, MockEmailSender::new());

        assert_eq!(service.count_emails().await.unwrap(), 10);
    }

    #[tokio::test]
    async fn count_unread_emails() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_count_unread_emails().returning(|| Ok(3));
        let service = EmailsService::new(repo, MockEmailSender::new());

        assert_eq!(service.count_unread_emails().await.unwrap(), 3);
    }

    #[tokio::test]
    async fn delete_email_succeeds() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_delete_email().returning(|_| Ok(()));
        let service = EmailsService::new(repo, MockEmailSender::new());

        assert!(service.delete_email("em-001").await.is_ok());
    }

    /// Captures every message handed to the sender, so a test can assert on what
    /// was queued rather than only on how many times the mock was called.
    fn capturing_sender() -> (MockEmailSender, Arc<StdMutex<Vec<EmailContext>>>) {
        let sent = Arc::new(StdMutex::new(Vec::new()));
        let sink = Arc::clone(&sent);
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(move |message| {
            sink.lock().expect("capture sink poisoned").push(message);
            Ok(())
        });

        (sender, sent)
    }

    fn submission() -> crate::extractors::email::CreateEmailRequest {
        let mut command = create_command();
        command.sender_email = "paula.okoye@example.com".to_string();
        command.first_name = Some("Paula".to_string());
        command.last_name = Some("Adeyemi".to_string());
        command.phone = Some("+234 806 657 0002".to_string());
        command.recipient_email = "support@example.com".to_string();
        command.subject = "Nipple line is leaking".to_string();
        command.body = "The header drips overnight.".to_string();
        command
    }

    /// The same submission from a caller that did not capture any name, which is
    /// what every row written before the fields existed looks like.
    fn anonymous_submission() -> crate::extractors::email::CreateEmailRequest {
        let mut command = submission();
        command.first_name = None;
        command.last_name = None;
        command
    }

    /// A submission with no callback number, which is most real contact-form
    /// traffic and the case that decides whether the forward's header table grows
    /// an empty labelled row.
    fn phoneless_submission() -> crate::extractors::email::CreateEmailRequest {
        let mut command = submission();
        command.phone = None;
        command
    }

    /// A submission whose optional fields are present but blank, which is what a
    /// frontend that submits every field produces when a visitor leaves them
    /// empty. Distinct from [`anonymous_submission`], which has them absent.
    fn blanked_submission() -> crate::extractors::email::CreateEmailRequest {
        let mut command = submission();
        command.first_name = Some(String::new());
        command.last_name = Some("   ".to_string());
        command.phone = None;
        command
    }

    #[tokio::test]
    async fn create_email_forwards_to_info_and_acknowledges_the_sender() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();
        assert_eq!(
            messages.len(),
            2,
            "expected a forward and an acknowledgement"
        );

        let forward = &messages[0];
        assert_eq!(forward.to_address, CONTACT_FORWARD_ADDRESS);
        assert_eq!(forward.to_address, "info@asterisk.app");
        assert_eq!(forward.subject, "[Contact form] Nipple line is leaking");
        assert!(
            forward.html_body.contains("paula.okoye@example.com"),
            "forward does not credit the sender: {forward:?}"
        );
        assert!(
            forward.html_body.contains("The header drips overnight."),
            "forward omits the body: {forward:?}"
        );

        let acknowledgement = &messages[1];
        assert_eq!(acknowledgement.to_address, "paula.okoye@example.com");
        assert_eq!(acknowledgement.subject, "We received your message");
        assert!(
            acknowledgement.html_body.contains("Hi Paula,"),
            "acknowledgement does not greet the sender: {acknowledgement:?}"
        );

        // The submitted recipient is recorded and shown, but nothing is sent to
        // it. It used to be a destination, which is how every submission ended
        // up in this same inbox twice.
        assert!(
            !messages
                .iter()
                .any(|message| message.to_address == "support@example.com"),
            "the submitted recipient is still a delivery destination"
        );
    }

    /// The whole point of capturing a first name: the acknowledgement greets the
    /// person by the name they typed, not one guessed from their address. The
    /// address here would have yielded "Paula Okoye", which is a different
    /// answer, so this fails if the derived form creeps back in.
    ///
    /// Also pins that the surname does *not* reach the greeting. The forward
    /// credits "Paula Adeyemi"; only the first name appears here.
    #[tokio::test]
    async fn the_acknowledgement_greets_the_provided_first_name() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();
        let acknowledgement = &messages[1];

        assert_eq!(acknowledgement.to_name, "Paula");
        assert!(
            acknowledgement.html_body.contains("Hi Paula,"),
            "greeting missing: {acknowledgement:?}"
        );
        assert!(
            !acknowledgement.html_body.contains("Paula Okoye"),
            "greeted a name inferred from the address: {acknowledgement:?}"
        );
        assert!(
            !acknowledgement.html_body.contains("Adeyemi"),
            "the surname leaked into the greeting: {acknowledgement:?}"
        );
    }

    /// The forward credits the full name the visitor gave, so the team is reading
    /// the same person the visitor typed rather than one guessed from their
    /// address. "Paula Adeyemi" cannot come from `paula.okoye@example.com`, so the
    /// second assertion fails if the derived form creeps back in.
    #[tokio::test]
    async fn the_forward_credits_the_full_submitted_name() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();
        let forward = &messages[0];

        assert!(
            forward.html_body.contains("Paula Adeyemi"),
            "forward omits the submitted full name: {forward:?}"
        );
        assert!(
            !forward.html_body.contains("Paula Okoye"),
            "forward attributed a name inferred from the address: {forward:?}"
        );
    }

    /// A callback number is the difference between a message the team can act on
    /// and one they can only reply to by email. It is only useful if it reaches
    /// the forward, so it has to survive the trip through the command.
    #[tokio::test]
    async fn the_forward_carries_the_submitted_phone_number() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();

        assert!(
            messages[0].html_body.contains("+234 806 657 0002"),
            "forward omits the phone number: {messages:?}"
        );
        // The number identifies the sender, so it must not be quoted back at
        // them; the acknowledgement is not where it belongs.
        assert!(
            !messages[1].html_body.contains("+234 806 657 0002"),
            "the phone number leaked into the acknowledgement: {messages:?}"
        );
    }

    /// A submission with no number must not leave an empty labelled row in the
    /// team's header table — every other cell there has a value.
    #[tokio::test]
    async fn the_forward_omits_the_phone_row_when_there_is_no_number() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&phoneless_submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();
        let forward = &messages[0];

        assert!(
            !forward.html_body.contains("Phone"),
            "an empty phone row was rendered: {forward:?}"
        );
    }

    /// A submission with no captured name still has to reach a human, so the
    /// address-derived form takes over. This is the path every row written
    /// before the fields existed takes.
    #[tokio::test]
    async fn a_submission_without_a_first_name_falls_back_to_the_address() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&anonymous_submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();

        assert!(
            messages[0].html_body.contains("Paula Okoye"),
            "forward lost the name entirely: {messages:?}"
        );
        assert!(
            messages[1].html_body.contains("Hi Paula Okoye,"),
            "acknowledgement lost the name entirely: {messages:?}"
        );
    }

    /// A frontend that submits every field sends `""` or a run of spaces for the
    /// ones the visitor left empty, which is not the same as omitting them. Both
    /// must land on the derived name rather than on a blank or a stray gap.
    #[tokio::test]
    async fn blank_submitted_names_fall_back_to_the_address() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&blanked_submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();

        assert!(
            messages[0].html_body.contains("Paula Okoye"),
            "blank names produced neither a name nor the fallback: {messages:?}"
        );
        assert!(
            !messages[0].html_body.contains("> <"),
            "a blank name left a gap in the from row: {messages:?}"
        );
        assert!(
            messages[1].html_body.contains("Hi Paula Okoye,"),
            "blank names broke the greeting: {messages:?}"
        );
    }

    /// The acknowledgement quotes none of the caller's text and reuses none of
    /// their subject line. Both are attacker-chosen, and this mail comes from
    /// our sending domain — so a submission must not be able to put its own words
    /// in the subject of a reply, or quote arbitrary text back at a third party.
    #[tokio::test]
    async fn the_acknowledgement_carries_no_caller_supplied_text() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let (sender, sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        service
            .create_email(&submission())
            .await
            .expect("should succeed");

        let messages = sent.lock().expect("capture sink poisoned").clone();
        let acknowledgement = &messages[1];

        assert!(
            !acknowledgement
                .html_body
                .contains("The header drips overnight."),
            "acknowledgement quotes the submitted body: {acknowledgement:?}"
        );
        assert!(
            !acknowledgement.subject.contains("Nipple line is leaking"),
            "acknowledgement reuses the submitted subject: {acknowledgement:?}"
        );
    }

    /// The forward is the reason this feature exists, so a forward that does not
    /// go out has to reach the caller. Only the first call is failed, and
    /// `.times(1)` is the assertion that matters here: the acknowledgement must
    /// not go out behind a forward that failed, or the sender is told their
    /// message is with the team when it is not.
    #[tokio::test]
    async fn a_forward_failure_fails_the_call_without_acknowledging() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().times(1).returning(|_| {
            Err(EmailServiceError::OperationFailed(
                "forward failed".to_string(),
            ))
        });

        let service = EmailsService::new(repo, sender);

        let result = service.create_email(&submission()).await;

        assert!(matches!(result, Err(ServiceError::EmailError(_))));
    }

    /// The team copy is the whole point of the endpoint, so it is attempted
    /// first: a failed courtesy must not stop the forward from ever being tried.
    #[tokio::test]
    async fn an_acknowledgement_failure_does_not_stop_the_forward() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let calls = Arc::new(AtomicUsize::new(0));
        let counter = Arc::clone(&calls);
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().times(2).returning(move |_| {
            if counter.fetch_add(1, Ordering::SeqCst) == 1 {
                Err(EmailServiceError::OperationFailed(
                    "acknowledgement failed".to_string(),
                ))
            } else {
                Ok(())
            }
        });

        let service = EmailsService::new(repo, sender);

        let result = service.create_email(&submission()).await;

        assert_eq!(
            calls.load(Ordering::SeqCst),
            2,
            "the forward was never attempted"
        );
        assert!(matches!(result, Err(ServiceError::EmailError(_))));
    }

    #[tokio::test]
    async fn a_queue_failure_on_the_first_send_fails_the_call() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email().returning(|_| Ok(test_email()));

        let mut sender = MockEmailSender::new();
        sender
            .expect_send_email()
            .times(1)
            .returning(|_| Err(EmailServiceError::OperationFailed("queue down".to_string())));

        let service = EmailsService::new(repo, sender);

        let result = service.create_email(&submission()).await;

        assert!(matches!(result, Err(ServiceError::EmailError(_))));
    }

    /// A throttled caller must be stopped before the write, not after, or the
    /// allowance would be costing a database round trip per rejected submission.
    #[tokio::test]
    async fn the_sender_allowance_is_charged_before_the_write() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email()
            .times(CONTACT_FORWARD_LIMIT as usize)
            .returning(|_| Ok(test_email()));

        let (sender, _sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        for _ in 0..CONTACT_FORWARD_LIMIT {
            service
                .create_email(&submission())
                .await
                .expect("within the allowance");
        }

        let result = service.create_email(&submission()).await;

        assert!(matches!(result, Err(ServiceError::TooManyRequests(_))));
    }

    /// The allowance is per sender address, so one caller exhausting it must not
    /// lock everyone else out.
    #[tokio::test]
    async fn the_sender_allowance_is_scoped_to_one_address() {
        let mut repo = MockEmailRepositoryExt::new();
        repo.expect_create_email()
            .times((CONTACT_FORWARD_LIMIT as usize) + 1)
            .returning(|_| Ok(test_email()));

        let (sender, _sent) = capturing_sender();
        let service = EmailsService::new(repo, sender);

        for _ in 0..CONTACT_FORWARD_LIMIT {
            service
                .create_email(&submission())
                .await
                .expect("within the allowance");
        }

        let mut other = submission();
        other.sender_email = "someone.else@example.com".to_string();

        assert!(service.create_email(&other).await.is_ok());
    }

    #[test]
    fn the_forward_escapes_the_submitted_body() {
        let html = EmailForwardTemplate {
            sender_name: "Paula Okoye",
            sender_email: "paula.okoye@example.com",
            recipient_email: "support@example.com",
            phone: "+234 806 657 0002",
            subject: "<b>Urgent</b>",
            body: "<script>alert(1)</script>",
        }
        .render()
        .expect("should render");

        // The whole point: this renders into the team's inbox, from a public
        // endpoint, with caller-controlled text. Anything live here is an
        // injection into that inbox.
        assert!(!html.contains("<script>"), "body rendered as live markup");
        assert!(
            !html.contains("<b>Urgent</b>"),
            "subject rendered as live markup"
        );

        // Askama 0.15's html escaper emits numeric references rather than the
        // named `&lt;`/`&gt;` pair. Both are inert; the numeric form is simply
        // what this codebase actually produces, so assert on it and not on an
        // escaping style that would never appear.
        assert!(
            html.contains("&#60;script&#62;alert(1)&#60;/script&#62;"),
            "body not present as escaped text"
        );
        assert!(
            html.contains("&#60;b&#62;Urgent&#60;/b&#62;"),
            "subject not present as escaped text"
        );
    }

    #[test]
    fn a_sender_name_is_derived_from_the_address_local_part() {
        assert_eq!(
            sender_display_name("paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(
            sender_display_name("PAULA_OKOYE@example.com"),
            "Paula Okoye"
        );
        assert_eq!(
            sender_display_name("paula+orders@example.com"),
            "Paula Orders"
        );
    }

    /// A mailbox with no letters in it still has to render as something
    /// readable rather than an empty gap in the forward.
    #[test]
    fn a_sender_name_falls_back_to_the_whole_address() {
        assert_eq!(
            sender_display_name("12345@example.com"),
            "12345@example.com"
        );
        assert_eq!(sender_display_name("___@example.com"), "___@example.com");
    }

    /// The greeting is the one place the whole-address fallback is wrong. In the
    /// team's inbox an address identifies who wrote, so `sender_display_name`
    /// falls back to it deliberately; rendered into "Hi {}," the same fallback
    /// produces "Hi 12345@example.com,", which is not an acknowledgement anyone
    /// should receive. So the greeting swaps it for the generic fallback.
    #[test]
    fn the_greeting_falls_back_rather_than_greeting_a_bare_address() {
        assert_eq!(
            submitter_greeting(None, "paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(submitter_greeting(None, "12345@example.com"), "there");
        assert_eq!(submitter_greeting(None, "___@example.com"), "there");
    }

    /// What the visitor typed is the greeting, and nothing is inferred from the
    /// address when it is there — including when the address would have produced
    /// a perfectly presentable guess.
    #[test]
    fn the_greeting_prefers_the_provided_first_name() {
        assert_eq!(
            submitter_greeting(Some("Paula"), "paula.okoye@example.com"),
            "Paula"
        );
        // Lowercase input is titled, the same treatment invitee_name gets.
        assert_eq!(
            submitter_greeting(Some("paula"), "ps145@example.com"),
            "Paula"
        );
    }

    /// The forward credits the whole submitted name. "Paula Adeyemi" is not
    /// something `paula.okoye@example.com` can produce, so this fails if the
    /// derived form is preferred over what the visitor typed.
    #[test]
    fn the_forwarded_name_is_the_full_submitted_name() {
        assert_eq!(
            submitter_name(Some("Paula"), Some("Adeyemi"), "paula.okoye@example.com"),
            "Paula Adeyemi"
        );
        // Either half alone still identifies the sender, so neither falls back.
        assert_eq!(
            submitter_name(Some("Paula"), None, "paula.okoye@example.com"),
            "Paula"
        );
        assert_eq!(
            submitter_name(None, Some("Adeyemi"), "paula.okoye@example.com"),
            "Adeyemi"
        );
    }

    /// A blank half is dropped rather than joined, so no half ever contributes a
    /// stray space. "Paula " in the team's From row is a visible defect.
    #[test]
    fn only_the_usable_halves_of_a_name_are_joined() {
        assert_eq!(
            submitter_name(Some("Paula"), Some("  "), "paula.okoye@example.com"),
            "Paula"
        );
        assert_eq!(
            submitter_name(Some("   "), Some("Adeyemi"), "paula.okoye@example.com"),
            "Adeyemi"
        );
        assert_eq!(
            submitter_name(Some("  Paula  "), Some("  Adeyemi  "), "ps145@example.com"),
            "Paula Adeyemi"
        );
    }

    /// With nothing usable submitted, the address-derived form is the last
    /// fallback for the forward — the team's inbox still has to say who wrote.
    #[test]
    fn a_name_with_no_usable_half_falls_back_to_the_address() {
        assert_eq!(
            submitter_name(None, None, "paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(
            submitter_name(Some(""), Some("  "), "paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(
            submitter_name(None, None, "12345@example.com"),
            "12345@example.com"
        );
    }

    /// What a visitor typed is not title-cased on the way into the forward.
    /// `to_title_case` would turn "McDonald" into "Mcdonald", which is the kind
    /// of silent mangling a visitor would notice and staff would not.
    #[test]
    fn a_submitted_name_is_not_recased() {
        assert_eq!(
            submitter_name(Some("McDonald"), Some("van der Berg"), "ps145@example.com"),
            "McDonald van der Berg"
        );
    }

    /// The greeting deliberately ignores the surname, so the two mails disagree
    /// on purpose: the forward identifies a stranger by their full name, the
    /// acknowledgement addresses someone by their first name alone. If a surname
    /// starts appearing in a greeting, this is the test that says it should not.
    #[test]
    fn the_greeting_uses_the_first_name_only() {
        assert_eq!(
            submitter_greeting(Some("Paula"), "paula.okoye@example.com"),
            "Paula"
        );
    }

    /// A blank name and an absent one mean the same thing. Without this,
    /// `Some("")` greets as "Hi ," and a field that only ever contains a space
    /// greets nobody at all.
    #[test]
    fn a_blank_first_name_is_treated_as_absent() {
        assert_eq!(
            submitter_name(Some(""), None, "paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(
            submitter_name(Some("   "), None, "paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(
            submitter_greeting(Some(""), "paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(
            submitter_greeting(Some("  "), "paula.okoye@example.com"),
            "Paula Okoye"
        );
        assert_eq!(submitter_greeting(Some(""), "12345@example.com"), "there");
    }

    /// A name with padding around it is trimmed before use, so the greeting
    /// never renders "Hi Paula ,".
    #[test]
    fn a_padded_first_name_is_trimmed() {
        assert_eq!(
            submitter_greeting(Some("  Paula  "), "paula.okoye@example.com"),
            "Paula"
        );
    }

    /// The two fallbacks have to compose. A blank name over an address with
    /// nothing title-casable in it must land on the generic greeting rather than
    /// back on the bare address.
    #[test]
    fn the_fallbacks_compose_when_neither_name_is_usable() {
        assert_eq!(submitter_greeting(Some(""), "12345@example.com"), "there");
        assert_eq!(submitter_greeting(Some("  "), "___@example.com"), "there");
    }

    /// The acknowledgement is the only mail a submission sends to a
    /// caller-supplied address, so it must carry nothing the caller chose.
    #[test]
    fn the_acknowledgement_template_quotes_no_submitted_text() {
        let html = AutoRespondTemplate {
            name: "Paula Okoye",
        }
        .render()
        .expect("should render");

        assert!(
            !html.contains("| safe"),
            "the acknowledgement template marks something as live markup"
        );
        assert!(html.contains("We received your message"));
        assert!(html.contains("Hi Paula Okoye,"));
        assert!(
            html.contains("do not reply"),
            "the copy does not warn that it is automatic: {html}"
        );
    }

    /// The template hides the recipient line when it would only repeat the
    /// sender, so the copy stays uncluttered for the common case while a
    /// mismatch between the two still shows up.
    #[test]
    fn the_forward_omits_a_recipient_line_that_would_only_repeat_the_sender() {
        let same = EmailForwardTemplate {
            sender_name: "Paula Okoye",
            sender_email: "paula.okoye@example.com",
            recipient_email: "paula.okoye@example.com",
            phone: "",
            subject: "Hello",
            body: "Hi",
        }
        .render()
        .expect("should render");

        let different = EmailForwardTemplate {
            sender_name: "Paula Okoye",
            sender_email: "paula.okoye@example.com",
            recipient_email: "support@example.com",
            phone: "",
            subject: "Hello",
            body: "Hi",
        }
        .render()
        .expect("should render");

        assert!(
            !same.contains("Sent to"),
            "redundant recipient line rendered"
        );
        assert!(different.contains("Sent to"), "mismatched recipient hidden");
    }
}
