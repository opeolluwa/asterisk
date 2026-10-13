use crate::{
    errors::service_error::ServiceError,
    models::newsletter,
    repositories::newsletter_repository::NewsletterRepositoryExt,
    services::{emails_service::EmailDispatchService, mailer::EmailSender},
};

#[derive(Clone)]
pub struct NewsletterService<R: NewsletterRepositoryExt, E: EmailSender> {
    repo: R,
    email_sender: E,
    frontend_url: String,
}

impl<R: NewsletterRepositoryExt, E: EmailSender> NewsletterService<R, E> {
    /// `frontend_url` is trimmed once here so building the explore link stays a
    /// plain concatenation and a trailing slash cannot double up.
    pub fn new(repo: R, email_sender: E, frontend_url: String) -> Self {
        Self {
            repo,
            email_sender,
            frontend_url: frontend_url.trim_end_matches('/').to_string(),
        }
    }

    /// Where the welcome email's button points. Kept in one place so a future
    /// edit to the path cannot leave the sender and the template disagreeing.
    fn explore_link(&self) -> String {
        format!("{}/newsletter", self.frontend_url)
    }

    /// A failure here is logged and swallowed: the subscription row is already
    /// committed by the time this runs, so there is nothing to roll back and
    /// the signup itself still succeeded. The next signup will not retry, so
    /// this is best-effort by design.
    async fn dispatch_welcome_email(&self, subscriber: &newsletter::Model) {
        if let Err(err) = self
            .email_sender
            .send_newsletter_welcome_email(&subscriber.email, "", &self.explore_link())
            .await
        {
            tracing::error!("failed to send newsletter welcome email due to {}", err);
        }
    }
}

pub trait NewsletterServiceExt {
    fn subscribe(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<newsletter::Model, ServiceError>>;

    fn unsubscribe(
        &self,
        email: &str,
    ) -> impl std::future::Future<Output = Result<newsletter::Model, ServiceError>>;
}

impl<R: NewsletterRepositoryExt + Send + Sync, E: EmailSender> NewsletterServiceExt
    for NewsletterService<R, E>
{
    /// Subscribes an address, sending the welcome email only when this call is
    /// what actually joined them to the list.
    ///
    /// The repository is idempotent, so a repeat submission of an address that
    /// is already subscribed performs no write. Sending there would mean every
    /// double-click on the subscribe button produces another email, so the
    /// welcome is tied to the outcome rather than to the call. A returning
    /// subscriber counts as new: from their side they are back on the list and
    /// have had no confirmation since they left.
    async fn subscribe(&self, email: &str) -> Result<newsletter::Model, ServiceError> {
        let outcome = self.repo.subscribe(email).await?;

        if outcome.is_new() {
            self.dispatch_welcome_email(outcome.model()).await;
        }

        Ok(outcome.into_model())
    }

    async fn unsubscribe(&self, email: &str) -> Result<newsletter::Model, ServiceError> {
        Ok(self.repo.unsubscribe(email).await?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::email_service_error::EmailServiceError;
    use crate::repositories::newsletter_repository::{
        MockNewsletterRepositoryExt, SubscribeOutcome,
    };
    use crate::services::mailer::MockEmailSender;

    const EXPLORE_LINK: &str = "https://asterisk.app/newsletter";

    fn subscriber(email: &str, subscribed: bool) -> newsletter::Model {
        newsletter::Model {
            identifier: "news-001".to_string(),
            email: email.to_string(),
            subscribed,
        }
    }

    fn test_service(
        repo: MockNewsletterRepositoryExt,
        email_sender: MockEmailSender,
    ) -> NewsletterService<MockNewsletterRepositoryExt, MockEmailSender> {
        NewsletterService::new(repo, email_sender, "https://asterisk.app".to_string())
    }

    #[tokio::test]
    async fn subscribe_sends_the_welcome_email_for_a_new_subscriber() {
        let mut repo = MockNewsletterRepositoryExt::new();
        repo.expect_subscribe().returning(|email| {
            assert_eq!(email, "reader@test.com");
            Ok(SubscribeOutcome::Created(subscriber(email, true)))
        });

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            assert_eq!(message.to_address, "reader@test.com");
            assert_eq!(message.from_name, "asterisk Accounts");
            assert_eq!(message.subject, "You're on the asterisk newsletter");
            assert!(
                message.html_body.contains(EXPLORE_LINK),
                "explore link missing: {}",
                message.html_body
            );
            Ok(())
        });

        let service = test_service(repo, sender);

        let result = service.subscribe("reader@test.com").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().email, "reader@test.com");
    }

    /// A returning subscriber has had no confirmation since they left, so they
    /// get the welcome again.
    #[tokio::test]
    async fn subscribe_sends_the_welcome_email_when_a_subscriber_returns() {
        let mut repo = MockNewsletterRepositoryExt::new();
        repo.expect_subscribe()
            .returning(|email| Ok(SubscribeOutcome::Reactivated(subscriber(email, true))));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().times(1).returning(|message| {
            assert_eq!(message.to_address, "reader@test.com");
            Ok(())
        });

        let service = test_service(repo, sender);

        assert!(service.subscribe("reader@test.com").await.is_ok());
    }

    /// The idempotent branch writes nothing, so a second submission of the same
    /// address must not produce a second email. Counted rather than merely
    /// configured, since a bare expectation would not catch a stray second call.
    #[tokio::test]
    async fn subscribe_does_not_resend_to_an_address_already_subscribed() {
        let mut repo = MockNewsletterRepositoryExt::new();
        repo.expect_subscribe()
            .returning(|email| Ok(SubscribeOutcome::AlreadySubscribed(subscriber(email, true))));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().never();

        let service = test_service(repo, sender);

        let result = service.subscribe("reader@test.com").await;

        assert!(result.is_ok());
        assert!(result.unwrap().subscribed);
    }

    /// A newsletter signup captures an email and nothing else, so the greeting
    /// has to degrade to "there" rather than printing an empty name.
    #[tokio::test]
    async fn subscribe_greets_a_nameless_subscriber_without_an_empty_name() {
        let mut repo = MockNewsletterRepositoryExt::new();
        repo.expect_subscribe()
            .returning(|email| Ok(SubscribeOutcome::Created(subscriber(email, true))));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            let body = &message.html_body;
            assert_eq!(message.to_name, "there");
            assert!(body.contains("Hi there,"), "greeting missing: {body}");
            assert!(!body.contains("Hi ,"), "empty greeting: {body}");
            Ok(())
        });

        let service = test_service(repo, sender);

        assert!(service.subscribe("reader@test.com").await.is_ok());
    }

    /// The row is committed before the mail is attempted, so a provider failure
    /// must not turn a successful signup into an error for the caller.
    #[tokio::test]
    async fn subscribe_survives_a_failed_welcome_email() {
        let mut repo = MockNewsletterRepositoryExt::new();
        repo.expect_subscribe()
            .returning(|email| Ok(SubscribeOutcome::Created(subscriber(email, true))));

        let mut sender = MockEmailSender::new();
        sender
            .expect_send_email()
            .returning(|_| Err(EmailServiceError::DeliveryError("smtp refused".to_string())));

        let service = test_service(repo, sender);

        let result = service.subscribe("reader@test.com").await;

        assert!(result.is_ok());
        assert!(result.unwrap().subscribed);
    }

    /// A configured base URL with a trailing slash must not produce `//newsletter`.
    #[tokio::test]
    async fn the_explore_link_survives_a_trailing_slash_on_the_base_url() {
        let mut repo = MockNewsletterRepositoryExt::new();
        repo.expect_subscribe()
            .returning(|email| Ok(SubscribeOutcome::Created(subscriber(email, true))));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().returning(|message| {
            let body = &message.html_body;
            assert!(body.contains(EXPLORE_LINK), "link wrong: {body}");
            assert!(
                !body.contains("com//newsletter"),
                "doubled slash in link: {body}"
            );
            Ok(())
        });

        let service =
            NewsletterService::new(repo, sender, "https://asterisk.app/".to_string());

        assert!(service.subscribe("reader@test.com").await.is_ok());
    }

    /// Leaving the list is not a welcome moment, and there is no confirmation
    /// email wired up for it.
    #[tokio::test]
    async fn unsubscribe_does_not_send_anything() {
        let mut repo = MockNewsletterRepositoryExt::new();
        repo.expect_unsubscribe()
            .returning(|email| Ok(subscriber(email, false)));

        let mut sender = MockEmailSender::new();
        sender.expect_send_email().never();

        let service = test_service(repo, sender);

        let result = service.unsubscribe("reader@test.com").await;

        assert!(result.is_ok());
        assert!(!result.unwrap().subscribed);
    }
}
