use crate::{
    config::env::Event,
    errors::service_error::ServiceError,
    extractors::complaint::{
        CreateComplaintRequest, ReplyComplaintRequest, UpdateComplaintRequest,
    },
    extractors::customer::CustomerProfile,
    extractors::notifications::{NotificationEvent, NotificationRecipientInput},
    models::{
        complaint_replies, complaints, orders,
        sea_orm_active_enums::{ComplaintStatus, NotificationActorType, NotificationRecipientType},
    },
    repositories::complaint_repository::ComplaintRepositoryExt,
    services::emails_service::EmailDispatchService,
    services::mailer::{EmailSender, sanitize::clean_reply_html},
    services::notifications_services::{
        NotificationEmitter, emit_to_active_admins, emit_to_active_admins_except,
        emit_to_recipients,
    },
    utils::notification_copy::{console_preview, join_facts},
};

/// Complaint business logic.
///
/// `Sender` is the outbound mail channel. Replies are stored first and dispatched
/// second, so the sender is only ever reached after the write has succeeded; it
/// is held here rather than looked up so the service stays a plain dependency
/// and can be unit tested with `MockEmailSender`.
///
/// `Emitter` is the admin notification channel, held for the same reason:
/// `MockNotificationEmitter` keeps these tests free of a database.
#[derive(Clone)]
pub struct ComplaintService<
    Repo: ComplaintRepositoryExt,
    Sender: EmailSender,
    N: NotificationEmitter,
> {
    repository: Repo,
    sender: Sender,
    emitter: N,
}

impl<Repo: ComplaintRepositoryExt, Sender: EmailSender, N: NotificationEmitter>
    ComplaintService<Repo, Sender, N>
{
    pub fn new(repository: Repo, sender: Sender, emitter: N) -> Self {
        Self {
            repository,
            sender,
            emitter,
        }
    }
}

pub trait ComplaintServiceExt {
    async fn create_complaint(
        &self,
        customer_identifier: &str,
        command: CreateComplaintRequest,
    ) -> Result<complaints::Model, ServiceError>;

    async fn find_all_complaints(
        &self,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<CustomerProfile>,
            Option<orders::Model>,
        )>,
        ServiceError,
    >;

    async fn find_complaints_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<CustomerProfile>,
            Option<orders::Model>,
        )>,
        ServiceError,
    >;

    async fn find_complaint_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<
        (
            complaints::Model,
            Option<CustomerProfile>,
            Option<orders::Model>,
        ),
        ServiceError,
    >;

    /// The one mutable operation on a complaint: marking it resolved.
    ///
    /// This is not a general update. `subject` and `description` are not
    /// carried by [`UpdateComplaintRequest`] at all, and the status transition
    /// is one-way — a complaint can be resolved, never un-resolved. Both halves
    /// are enforced here so the rule is legible in one place; the repository
    /// additionally guards the write with a conditional `UPDATE` so two
    /// concurrent resolves cannot race a reopen.
    async fn update_complaint(
        &self,
        identifier: &str,
        command: UpdateComplaintRequest,
    ) -> Result<complaints::Model, ServiceError>;

    async fn archive_complaint(&self, identifier: &str) -> Result<(), ServiceError>;

    async fn count_complaints(&self) -> Result<i64, ServiceError>;

    async fn find_complaint_replies(
        &self,
        identifier: &str,
    ) -> Result<Vec<complaint_replies::Model>, ServiceError>;

    /// `actor_identifier` is the admin who replied; `sender_email` is the address
    /// the reply is delivered from, which is not always theirs.
    async fn reply_to_complaint(
        &self,
        identifier: &str,
        actor_identifier: &str,
        sender_email: &str,
        command: ReplyComplaintRequest,
    ) -> Result<complaint_replies::Model, ServiceError>;
}

impl<Repo, Sender, N> ComplaintServiceExt for ComplaintService<Repo, Sender, N>
where
    Repo: ComplaintRepositoryExt + Sync + Send,
    Sender: EmailSender + Sync + Send,
    N: NotificationEmitter + Sync + Send,
{
    async fn create_complaint(
        &self,
        customer_identifier: &str,
        command: CreateComplaintRequest,
    ) -> Result<complaints::Model, ServiceError> {
        let complaint = self
            .repository
            .create_complaint(customer_identifier, &command)
            .await?;

        // The complaint is the durable record by this point, so a failure to
        // notify is logged rather than returned: the admin inbox losing a
        // message must not undo the complaint the customer just filed.
        emit_to_active_admins(
            &self.emitter,
            NotificationEvent {
                event_type: Event::ComplaintCreated,
                actor_identifier: Some(customer_identifier.to_string()),
                actor_type: NotificationActorType::Customer,
                entity_type: Some("complaint".to_string()),
                entity_identifier: Some(complaint.identifier.clone()),
                title: format!("New complaint: {}", complaint.subject),
                // A complaint description is written by a customer with no length
                // limit, and the console card clamps to two lines. Preview it
                // rather than shipping a paragraph that truncates mid-word.
                body: console_preview(&complaint.description),
                metadata: Some(serde_json::json!({
                    "complaintIdentifier": complaint.identifier,
                    "subject": complaint.subject,
                    "status": complaint.status,
                })),
                recipients: Vec::new(),
            },
        )
        .await;

        Ok(complaint)
    }

    async fn find_all_complaints(
        &self,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<CustomerProfile>,
            Option<orders::Model>,
        )>,
        ServiceError,
    > {
        Ok(self
            .repository
            .find_all_complaints()
            .await?
            .into_iter()
            .map(|(complaint, customer, order)| {
                (
                    complaint,
                    customer.as_ref().map(CustomerProfile::from),
                    order,
                )
            })
            .collect())
    }

    async fn find_complaints_by_customer_identifier(
        &self,
        customer_identifier: &str,
    ) -> Result<
        Vec<(
            complaints::Model,
            Option<CustomerProfile>,
            Option<orders::Model>,
        )>,
        ServiceError,
    > {
        Ok(self
            .repository
            .find_complaints_by_customer_identifier(customer_identifier)
            .await?
            .into_iter()
            .map(|(complaint, customer, order)| {
                (
                    complaint,
                    customer.as_ref().map(CustomerProfile::from),
                    order,
                )
            })
            .collect())
    }

    async fn find_complaint_by_identifier(
        &self,
        identifier: &str,
    ) -> Result<
        (
            complaints::Model,
            Option<CustomerProfile>,
            Option<orders::Model>,
        ),
        ServiceError,
    > {
        let (complaint, customer, order) = self
            .repository
            .find_complaint_by_identifier(identifier)
            .await?;
        Ok((
            complaint,
            customer.as_ref().map(CustomerProfile::from),
            order,
        ))
    }

    async fn update_complaint(
        &self,
        identifier: &str,
        command: UpdateComplaintRequest,
    ) -> Result<complaints::Model, ServiceError> {
        // `subject` and `description` are the customer's own account of what went
        // wrong, so they are append-only and no longer updatable. Status is the
        // only mutable field, which is why the repository exposes a
        // status-only write for it.
        //
        // An absent status is a no-op rather than an error, so an empty body is
        // accepted and simply returns the complaint unchanged.
        let Some(status) = command.status else {
            let (complaint, _, _) = self
                .repository
                .find_complaint_by_identifier(identifier)
                .await?;

            return Ok(complaint);
        };

        // Resolving is the only accepted outcome. Every other target is refused
        // as an illegal transition rather than as bad input: the status value
        // itself parses fine, the complaint just cannot go there. That keeps one
        // rule behind every rejection — `open` may become `resolved`, and nothing
        // else moves — which is what stops a complaint being reopened.
        if status != ComplaintStatus::Resolved {
            return Err(ServiceError::Conflict(format!(
                "a complaint can only be marked resolved, not {status:?}"
            )));
        }

        match self.repository.resolve_complaint(identifier).await? {
            Some(complaint) => Ok(complaint),
            // The guarded update matched no row. Re-read to separate a missing
            // complaint (404) from one that has already been resolved (409); the
            // repository cannot distinguish them because both are zero rows.
            None => {
                // Propagating `NotFound` here yields 404, so only a complaint
                // that does exist but is no longer open falls through to 409.
                let (complaint, _, _) = self
                    .repository
                    .find_complaint_by_identifier(identifier)
                    .await?;

                // Reports the state it actually found rather than assuming
                // `resolved`: the guard excludes any non-`open` status, and
                // naming the real one is what makes the 409 diagnosable.
                Err(ServiceError::Conflict(format!(
                    "complaint {} is already {:?} and cannot be changed",
                    complaint.identifier,
                    complaint.status.unwrap_or(ComplaintStatus::Resolved)
                )))
            }
        }
    }

    async fn archive_complaint(&self, identifier: &str) -> Result<(), ServiceError> {
        Ok(self
            .repository
            .archive_complaint_by_identifier(identifier)
            .await?)
    }

    async fn count_complaints(&self) -> Result<i64, ServiceError> {
        Ok(self.repository.count_complaints().await?)
    }

    async fn find_complaint_replies(
        &self,
        identifier: &str,
    ) -> Result<Vec<complaint_replies::Model>, ServiceError> {
        Ok(self
            .repository
            .find_replies_by_complaint(identifier)
            .await?)
    }

    /// Stores a reply and emails it to the customer.
    ///
    /// Order matters: the row is written and sanitized first, then dispatched.
    /// A dispatch failure is logged and swallowed rather than surfaced, because
    /// the reply is already durable at that point and failing the request would
    /// tell the admin it was lost when it was not. The queue is the retry
    /// boundary, and it is only reached once the write has landed.
    async fn reply_to_complaint(
        &self,
        identifier: &str,
        actor_identifier: &str,
        sender_email: &str,
        command: ReplyComplaintRequest,
    ) -> Result<complaint_replies::Model, ServiceError> {
        // Resolve the complaint first so an unknown identifier fails before any
        // write or dispatch, and so the customer's address is available.
        let (complaint, customer, _order) = self
            .repository
            .find_complaint_by_identifier(identifier)
            .await?;

        let sanitized = clean_reply_html(&command.body);

        let reply = self
            .repository
            .create_complaint_reply(identifier, sender_email, &sanitized)
            .await?;

        if let Some(customer) = customer {
            let customer_name = {
                let first = customer.first_name.clone().unwrap_or_default();
                let last = customer.last_name.clone().unwrap_or_default();
                format!("{first} {last}").trim().to_string()
            };

            if let Err(err) = self
                .sender
                .send_complaint_reply_email(
                    &customer.email,
                    &customer_name,
                    &complaint.subject,
                    &sanitized,
                )
                .await
            {
                tracing::error!(
                    "Failed to dispatch complaint reply {}: {}",
                    reply.identifier,
                    err
                );
            }

            // The customer filed the complaint and is the one waiting on the
            // answer, so they are the audience for this event — the same
            // `complaint_replied` the asterisk sees, addressed to the reader it
            // is about. The email above and this row are the same fact on two
            // channels: one lands in their inbox, the other in the notification
            // list they read on their way back in.
            //
            // Emitted after the reply is durable, for the same reason as the
            // dispatch: the work is already committed, so a failure here costs a
            // notification rather than the reply.
            emit_to_recipients(
                &self.emitter,
                NotificationEvent {
                    event_type: Event::ComplaintReplied,
                    actor_identifier: Some(actor_identifier.to_string()),
                    actor_type: NotificationActorType::Admin,
                    entity_type: Some("complaint".to_string()),
                    entity_identifier: Some(complaint.identifier.clone()),
                    // The subject is the thread this reply belongs to, and it is
                    // what the customer recognises; `Reply sent:` above is the
                    // asterisk's framing of the same event.
                    title: format!("Re: {}", complaint.subject),
                    body: console_preview(&clean_reply_html(&command.body)),
                    metadata: Some(serde_json::json!({
                        "complaintIdentifier": complaint.identifier,
                        "replyIdentifier": reply.identifier,
                        "subject": complaint.subject,
                    })),
                    recipients: vec![NotificationRecipientInput {
                        recipient_identifier: customer.identifier,
                        recipient_type: NotificationRecipientType::Customer,
                        email_address: customer.email,
                        display_name: Some(customer_name),
                    }],
                },
            )
            .await;
        } else {
            tracing::error!(
                "Complaint {} has no resolvable customer; reply {} stored but not dispatched",
                complaint.identifier,
                reply.identifier
            );
        }

        // The asterisk hears about the reply so a colleague can pick up the
        // thread, but not the admin who wrote it: they just did this, and an
        // echo of their own action is the kind of noise that teaches people to
        // ignore the inbox. Emitted after the reply is durable, alongside the
        // dispatch above and for the same reason: the work is already committed.
        emit_to_active_admins_except(
            &self.emitter,
            NotificationEvent {
                event_type: Event::ComplaintReplied,
                actor_identifier: Some(actor_identifier.to_string()),
                actor_type: NotificationActorType::Admin,
                entity_type: Some("complaint".to_string()),
                entity_identifier: Some(complaint.identifier.clone()),
                title: format!("Reply sent: {}", complaint.subject),
                // The reply body is the agent's own prose and just as
                // unbounded, and an admin scanning the inbox needs to recognise
                // which reply it was, not re-read it here.
                body: join_facts(&[
                    Some(console_preview(&clean_reply_html(&command.body))),
                    Some(format!("to {}", complaint.customer_identifier)),
                ]),
                metadata: Some(serde_json::json!({
                    "complaintIdentifier": complaint.identifier,
                    "replyIdentifier": reply.identifier,
                    "subject": complaint.subject,
                })),
                recipients: Vec::new(),
            },
            Some(actor_identifier),
        )
        .await;

        Ok(reply)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::database_error::DatabaseError;
    use crate::errors::email_service_error::EmailServiceError;
    use crate::models::customers;
    use crate::models::sea_orm_active_enums::ComplaintStatus;
    use crate::repositories::complaint_repository::MockComplaintRepositoryExt;
    use crate::repositories::notifications_repository::EmittedNotification;
    use crate::services::mailer::MockEmailSender;
    use crate::services::notifications_services::MockNotificationEmitter;
    use sea_orm::sqlx::types::chrono::Utc;
    use std::sync::{Arc, Mutex};

    /// An emitter for tests that are not about notifications.
    ///
    /// An empty admin audience makes `emit_to_active_admins` return before it ever
    /// reaches `emit`. The customer-side emit is addressed directly rather than
    /// resolved, so it reaches `emit` and needs a stub here — without one,
    /// `mockall` would panic in every reply test. Tests that care about either
    /// emit build their own.
    fn quiet_emitter() -> MockNotificationEmitter {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(|| Ok(Vec::new()));

        emitter.expect_emit().returning(|event| {
            Ok(EmittedNotification {
                notification: crate::models::notifications::Model {
                    identifier: "notification-1".to_string(),
                    event_type: event.event_type.to_string(),
                    actor_identifier: event.actor_identifier.clone(),
                    actor_type: event.actor_type.clone(),
                    entity_type: event.entity_type.clone(),
                    entity_identifier: event.entity_identifier.clone(),
                    title: event.title.clone(),
                    body: event.body.clone(),
                    metadata: event.metadata.clone(),
                    created_at: Utc::now().fixed_offset(),
                    updated_at: None,
                },
                recipients: Vec::new(),
                deliveries: Vec::new(),
            })
        });

        emitter
    }

    /// A asterisk reader, for the tests about who an admin-facing event reaches.
    fn admin(identifier: &str) -> NotificationRecipientInput {
        NotificationRecipientInput {
            recipient_identifier: identifier.to_string(),
            recipient_type: NotificationRecipientType::Admin,
            email_address: format!("{identifier}@asterisk.app"),
            display_name: None,
        }
    }

    fn complaint() -> complaints::Model {
        complaints::Model {
            identifier: "complaint-1".to_string(),
            customer_identifier: "customer-1".to_string(),
            order_identifier: None,
            subject: "Damaged on arrival".to_string(),
            description: "The box was crushed.".to_string(),
            status: Some(ComplaintStatus::Open),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            archived_at: None,
        }
    }

    fn customer() -> customers::Model {
        customers::Model {
            identifier: "customer-1".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
            email: "ada@example.com".to_string(),
            first_name: Some("Ada".to_string()),
            last_name: Some("Lovelace".to_string()),
            password: "hashed".to_string(),
            phone: None,
            picture: None,
            deleted_at: None,
            completed_onboarding: true,
        }
    }

    fn reply() -> complaint_replies::Model {
        complaint_replies::Model {
            identifier: "reply-1".to_string(),
            complaint_identifier: "complaint-1".to_string(),
            body: "<p>Sorry about that.</p>".to_string(),
            sender_email: "admin@asterisk.app".to_string(),
            created_at: Utc::now().fixed_offset(),
            updated_at: None,
        }
    }

    fn command(body: &str) -> ReplyComplaintRequest {
        ReplyComplaintRequest {
            body: body.to_string(),
        }
    }

    /// Repo configured to resolve `complaint-1` to a customer, and to echo back
    /// whatever body it was handed so the sanitized value is observable.
    fn repo_returning(stored_body: String) -> MockComplaintRepositoryExt {
        let mut repo = MockComplaintRepositoryExt::new();
        let _ = stored_body;
        repo.expect_find_complaint_by_identifier()
            .returning(|_| Ok((complaint(), Some(customer()), None)));
        repo.expect_create_complaint_reply()
            .returning(move |_, _, body| {
                let mut model = reply();
                model.body = body.to_string();
                Ok(model)
            });
        repo
    }

    #[tokio::test]
    async fn reply_persists_then_dispatches_to_the_customer() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().times(1).returning(|message| {
            assert_eq!(message.to_address, "ada@example.com");
            assert_eq!(message.to_name, "Ada Lovelace");
            assert_eq!(message.subject, "Re: Damaged on arrival");
            Ok(())
        });

        let service = ComplaintService::new(
            repo_returning("<p>Sorry about that.</p>".to_string()),
            sender,
            quiet_emitter(),
        );

        let stored = service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>hi</p>"),
            )
            .await
            .expect("reply should succeed");

        assert_eq!(stored.identifier, "reply-1");
        assert_eq!(stored.complaint_identifier, "complaint-1");
        assert_eq!(stored.sender_email, "admin@asterisk.app");
    }

    /// The write path is the authoritative sanitize: a client that never ran
    /// DOMPurify must not be able to persist script into the table.
    #[tokio::test]
    async fn reply_sanitizes_the_body_before_persisting() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|_| Ok(()));

        let service = ComplaintService::new(repo_returning(String::new()), sender, quiet_emitter());

        let stored = service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>hello</p><script>alert(1)</script><img src=x onerror=alert(1)>"),
            )
            .await
            .expect("reply should succeed");

        assert!(stored.body.contains("hello"));
        assert!(!stored.body.contains("script"));
        assert!(!stored.body.contains("onerror"));
        assert!(!stored.body.contains("img"));
    }

    /// Formatting the editor can legitimately produce must survive, otherwise
    /// the allowlist in `sanitize` is too tight.
    #[tokio::test]
    async fn reply_preserves_editor_formatting() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|_| Ok(()));

        let service = ComplaintService::new(repo_returning(String::new()), sender, quiet_emitter());

        let stored = service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command(
                    "<p><strong>bold</strong> <em>italic</em></p><ul><li>one</li></ul><blockquote>q</blockquote><a href=\"https://example.com\">link</a>",
                ),
            )
            .await
            .expect("reply should succeed");

        for fragment in [
            "<strong>bold</strong>",
            "<em>italic</em>",
            "<ul>",
            "<li>one</li>",
            "<blockquote>",
            "https://example.com",
        ] {
            assert!(
                stored.body.contains(fragment),
                "expected sanitized body to keep {fragment}, got {}",
                stored.body
            );
        }
    }

    /// A dispatch failure must not lose the reply. The row is already written by
    /// the time the sender is called, so failing the request would report a lost
    /// reply that is actually stored.
    #[tokio::test]
    async fn dispatch_failure_still_returns_the_stored_reply() {
        let mut sender = MockEmailSender::new();
        sender
            .expect_send_email()
            .returning(|_| Err(EmailServiceError::OperationFailed("queue down".to_string())));

        let service = ComplaintService::new(
            repo_returning("<p>Sorry about that.</p>".to_string()),
            sender,
            quiet_emitter(),
        );

        let stored = service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>hi</p>"),
            )
            .await
            .expect("reply should survive a dispatch failure");

        assert_eq!(stored.identifier, "reply-1");
    }

    /// Nothing is written or sent for an unknown complaint — the lookup happens
    /// before both.
    #[tokio::test]
    async fn unknown_complaint_fails_before_any_write_or_send() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_find_complaint_by_identifier().returning(|_| {
            Err(crate::errors::database_error::DatabaseError::NotFound(
                "complaint not found".to_string(),
            ))
        });

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().never();

        let service = ComplaintService::new(repo, sender, quiet_emitter());

        assert!(
            service
                .reply_to_complaint(
                    "nope",
                    "admin-1",
                    "admin@asterisk.app",
                    command("<p>hi</p>")
                )
                .await
                .is_err()
        );
    }

    /// A complaint whose customer cannot be resolved still stores the reply; only
    /// the dispatch is skipped.
    #[tokio::test]
    async fn missing_customer_still_stores_the_reply() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_find_complaint_by_identifier()
            .returning(|_| Ok((complaint(), None, None)));
        repo.expect_create_complaint_reply()
            .returning(|_, _, _| Ok(reply()));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().never();

        let service = ComplaintService::new(repo, sender, quiet_emitter());

        let stored = service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>hi</p>"),
            )
            .await
            .expect("reply should still be stored");

        assert_eq!(stored.identifier, "reply-1");
    }

    /// Records every emitted event so a test can assert on the whole set. Two
    /// emits are expected per reply — one to the customer, one to the asterisk
    /// — so asserting on "the" emit is not possible without this.
    fn recording_emitter(
        admins: Vec<NotificationRecipientInput>,
        events: Arc<Mutex<Vec<NotificationEvent>>>,
    ) -> MockNotificationEmitter {
        let mut emitter = MockNotificationEmitter::default();

        emitter
            .expect_active_admin_recipients()
            .returning(move || Ok(admins.clone()));

        emitter.expect_emit().returning(move |event| {
            events.lock().unwrap().push(event.clone());

            Ok(EmittedNotification {
                notification: crate::models::notifications::Model {
                    identifier: "notification-1".to_string(),
                    event_type: event.event_type.to_string(),
                    actor_identifier: event.actor_identifier.clone(),
                    actor_type: event.actor_type.clone(),
                    entity_type: event.entity_type.clone(),
                    entity_identifier: event.entity_identifier.clone(),
                    title: event.title.clone(),
                    body: event.body.clone(),
                    metadata: event.metadata.clone(),
                    created_at: Utc::now().fixed_offset(),
                    updated_at: None,
                },
                recipients: Vec::new(),
                deliveries: Vec::new(),
            })
        });

        emitter
    }

    /// The customer filed the complaint and is waiting on the answer, so the
    /// notification goes to them rather than to the asterisk.
    #[tokio::test]
    async fn reply_notifies_the_customer_who_asked() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|_| Ok(()));

        let events = Arc::new(Mutex::new(Vec::new()));
        let service = ComplaintService::new(
            repo_returning("<p>hi</p>".to_string()),
            sender,
            recording_emitter(Vec::new(), Arc::clone(&events)),
        );

        service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>Sorry about that.</p>"),
            )
            .await
            .expect("reply should succeed");

        let events = events.lock().unwrap();
        let customer_event = events
            .iter()
            .find(|event| !event.recipients.is_empty())
            .expect("the customer should have been notified");

        assert_eq!(customer_event.event_type, Event::ComplaintReplied);
        // The admin is the actor on a notification the customer reads; that is
        // what tells the two apart in the customer's list.
        assert_eq!(customer_event.actor_type, NotificationActorType::Admin);
        assert_eq!(customer_event.actor_identifier.as_deref(), Some("admin-1"));
        assert_eq!(
            customer_event.entity_identifier.as_deref(),
            Some("complaint-1")
        );
        assert_eq!(customer_event.recipients.len(), 1);
        assert_eq!(
            customer_event.recipients[0].recipient_identifier,
            "customer-1"
        );
        assert_eq!(
            customer_event.recipients[0].recipient_type,
            NotificationRecipientType::Customer
        );
        assert_eq!(
            customer_event.recipients[0].email_address,
            "ada@example.com"
        );
    }

    /// The acting admin just wrote this reply; telling them so is an echo of
    /// their own action, and it is what teaches people to ignore the inbox.
    #[tokio::test]
    async fn reply_does_not_notify_the_admin_who_wrote_it() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|_| Ok(()));

        let events = Arc::new(Mutex::new(Vec::new()));
        let service = ComplaintService::new(
            repo_returning("<p>hi</p>".to_string()),
            sender,
            recording_emitter(
                vec![admin("admin-1"), admin("admin-2")],
                Arc::clone(&events),
            ),
        );

        service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>Sorry about that.</p>"),
            )
            .await
            .expect("reply should succeed");

        let events = events.lock().unwrap();
        let admin_event = events
            .iter()
            .find(|event| event.title.starts_with("Reply sent"))
            .expect("the asterisk should still hear about the reply");

        let audience = admin_event
            .recipients
            .iter()
            .map(|recipient| recipient.recipient_identifier.as_str())
            .collect::<Vec<_>>();

        assert_eq!(
            audience,
            vec!["admin-2"],
            "the author should be excluded but their colleagues should not"
        );
    }

    /// Sole admin: excluding them leaves nobody to tell, which is a no-op rather
    /// than an emit with an empty audience.
    #[tokio::test]
    async fn reply_from_the_only_admin_notifies_no_admin() {
        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|_| Ok(()));

        let events = Arc::new(Mutex::new(Vec::new()));
        let service = ComplaintService::new(
            repo_returning("<p>hi</p>".to_string()),
            sender,
            recording_emitter(vec![admin("admin-1")], Arc::clone(&events)),
        );

        service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>Sorry about that.</p>"),
            )
            .await
            .expect("reply should succeed");

        let events = events.lock().unwrap();

        assert!(
            !events
                .iter()
                .any(|event| event.title.starts_with("Reply sent")),
            "an admin-facing emit with an empty audience should not happen"
        );
        // The customer is still told; the exclusion is about the asterisk.
        assert!(events.iter().any(|event| !event.recipients.is_empty()));
    }

    /// With no customer to address, the notification has no audience and must not
    /// be written at all.
    #[tokio::test]
    async fn reply_without_a_customer_emits_nothing() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_find_complaint_by_identifier()
            .returning(|_| Ok((complaint(), None, None)));
        repo.expect_create_complaint_reply()
            .returning(|_, _, _| Ok(reply()));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().never();

        let events = Arc::new(Mutex::new(Vec::new()));
        let service = ComplaintService::new(
            repo,
            sender,
            recording_emitter(vec![admin("admin-2")], Arc::clone(&events)),
        );

        service
            .reply_to_complaint(
                "complaint-1",
                "admin-1",
                "admin@asterisk.app",
                command("<p>hi</p>"),
            )
            .await
            .expect("reply should still be stored");

        let events = events.lock().unwrap();

        // Only the asterisk emit survives, and it is not addressed to a
        // customer who does not exist.
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].recipients[0].recipient_identifier, "admin-2");
    }

    #[tokio::test]
    async fn find_replies_passes_through_in_repository_order() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_find_replies_by_complaint()
            .returning(|_| Ok(vec![reply(), reply()]));

        let sender = MockEmailSender::new();
        let service = ComplaintService::new(repo, sender, quiet_emitter());

        let replies = service
            .find_complaint_replies("complaint-1")
            .await
            .expect("replies should load");

        assert_eq!(replies.len(), 2);
    }

    /// The complaint as it looks once the guarded update has run.
    fn resolved_complaint() -> complaints::Model {
        complaints::Model {
            status: Some(ComplaintStatus::Resolved),
            ..complaint()
        }
    }

    fn resolve_command(status: Option<ComplaintStatus>) -> UpdateComplaintRequest {
        UpdateComplaintRequest { status }
    }

    fn create_command() -> CreateComplaintRequest {
        CreateComplaintRequest {
            order_identifier: None,
            subject: "Damaged on arrival".to_string(),
            description: "The box was crushed.".to_string(),
        }
    }

    #[tokio::test]
    async fn a_complaint_is_created_against_the_caller_rather_than_the_body() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_create_complaint()
            .withf(|customer_identifier, _| customer_identifier == "customer-1")
            .times(1)
            .returning(|_, _| Ok(complaint()));

        let service = ComplaintService::new(repo, MockEmailSender::new(), quiet_emitter());

        let created = service
            .create_complaint("customer-1", create_command())
            .await
            .expect("a complaint should be created");

        assert_eq!(created.identifier, "complaint-1");
        assert_eq!(created.status, Some(ComplaintStatus::Open));
    }

    #[tokio::test]
    async fn a_failed_write_reports_rather_than_notifying_admins() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_create_complaint()
            .returning(|_, _| Err(DatabaseError::NotFound("customer not found".to_string())));
        let mut emitter = quiet_emitter();
        emitter.expect_emit().never();

        let service = ComplaintService::new(repo, MockEmailSender::new(), emitter);

        let error = service
            .create_complaint("customer-1", create_command())
            .await
            .expect_err("the insert failed");

        assert!(
            matches!(
                error,
                ServiceError::DatabaseError(DatabaseError::NotFound(_))
            ),
            "expected the repository error to propagate, got {error:?}"
        );
    }

    #[tokio::test]
    async fn resolving_an_open_complaint_returns_it_as_resolved() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_resolve_complaint()
            .withf(|identifier| identifier == "complaint-1")
            .returning(|_| Ok(Some(resolved_complaint())));

        let service = ComplaintService::new(repo, MockEmailSender::new(), quiet_emitter());

        let resolved = service
            .update_complaint(
                "complaint-1",
                resolve_command(Some(ComplaintStatus::Resolved)),
            )
            .await
            .expect("an open complaint should resolve");

        assert_eq!(resolved.status, Some(ComplaintStatus::Resolved));
    }

    #[tokio::test]
    async fn a_status_other_than_resolved_is_refused_without_touching_the_repository() {
        // `MockComplaintRepositoryExt` has no `resolve_complaint` expectation, so
        // any repository call would panic the test rather than quietly pass.
        for status in [
            ComplaintStatus::Open,
            ComplaintStatus::InProgress,
            ComplaintStatus::Closed,
        ] {
            let service = ComplaintService::new(
                MockComplaintRepositoryExt::new(),
                MockEmailSender::new(),
                quiet_emitter(),
            );

            let error = service
                .update_complaint("complaint-1", resolve_command(Some(status.clone())))
                .await
                .expect_err("only resolving is a permitted transition");

            assert!(
                matches!(error, ServiceError::Conflict(_)),
                "{status:?} should be a 409, got {error:?}"
            );
        }
    }

    #[tokio::test]
    async fn reopening_a_resolved_complaint_is_a_conflict() {
        let mut repo = MockComplaintRepositoryExt::new();
        // Zero rows matched, which is what the `status = 'open'` guard returns
        // once the complaint has already been resolved.
        repo.expect_resolve_complaint().returning(|_| Ok(None));
        repo.expect_find_complaint_by_identifier()
            .returning(|_| Ok((resolved_complaint(), Some(customer()), None)));

        let service = ComplaintService::new(repo, MockEmailSender::new(), quiet_emitter());

        let error = service
            .update_complaint(
                "complaint-1",
                resolve_command(Some(ComplaintStatus::Resolved)),
            )
            .await
            .expect_err("a resolved complaint must not be resolved again");

        assert!(
            matches!(error, ServiceError::Conflict(_)),
            "expected 409, got {error:?}"
        );
    }

    #[tokio::test]
    async fn resolving_an_unknown_complaint_reports_not_found_rather_than_conflict() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_resolve_complaint().returning(|_| Ok(None));
        repo.expect_find_complaint_by_identifier()
            .returning(|_| Err(DatabaseError::NotFound("complaint not found".to_string())));

        let service = ComplaintService::new(repo, MockEmailSender::new(), quiet_emitter());

        let error = service
            .update_complaint("missing", resolve_command(Some(ComplaintStatus::Resolved)))
            .await
            .expect_err("there is nothing to resolve");

        // The re-read is what separates 404 from 409; without it a missing
        // complaint would be reported as an already-resolved one.
        assert!(
            matches!(
                &error,
                ServiceError::DatabaseError(DatabaseError::NotFound(_))
            ),
            "expected 404, got {error:?}"
        );
        assert_eq!(
            error.status_code(),
            axum::http::StatusCode::NOT_FOUND,
            "a missing complaint must not surface as a conflict"
        );
    }

    #[tokio::test]
    async fn a_body_without_a_status_leaves_the_complaint_untouched() {
        let mut repo = MockComplaintRepositoryExt::new();
        repo.expect_find_complaint_by_identifier()
            .returning(|_| Ok((complaint(), Some(customer()), None)));
        // No `expect_resolve_complaint`: reaching the repository here would be a
        // panic, which is the point — an empty body must not write.

        let service = ComplaintService::new(repo, MockEmailSender::new(), quiet_emitter());

        let unchanged = service
            .update_complaint("complaint-1", resolve_command(None))
            .await
            .expect("an empty body is a no-op, not an error");

        assert_eq!(unchanged.status, Some(ComplaintStatus::Open));
        assert_eq!(unchanged.subject, "Damaged on arrival");
    }
}
