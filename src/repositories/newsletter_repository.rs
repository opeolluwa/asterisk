use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, DatabaseConnection, DbErr, EntityTrait,
    QueryFilter,
};
use ulid::Ulid;

use crate::repositories::base::Repository;
use crate::{errors::database_error::DatabaseError, models::newsletter};

/// What a `subscribe` call actually did to the table.
///
/// The three cases look alike from the outside — a caller just wants the row —
/// but they are not the same event. A welcome email should go out when someone
/// joins or comes back, not when they submit the same address a second time, so
/// the write has to be distinguishable from the no-op.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SubscribeOutcome {
    /// The address was not on the list and has been inserted.
    Created(newsletter::Model),
    /// The address had previously unsubscribed and has been switched back on.
    Reactivated(newsletter::Model),
    /// The address was already subscribed. Nothing was written.
    AlreadySubscribed(newsletter::Model),
}

impl SubscribeOutcome {
    /// The row as it stands after the call, whichever branch produced it.
    pub fn model(&self) -> &newsletter::Model {
        match self {
            Self::Created(model) | Self::Reactivated(model) | Self::AlreadySubscribed(model) => {
                model
            }
        }
    }

    /// Consumes the outcome and hands back the row, for callers that only want
    /// the model and have already acted on the distinction.
    pub fn into_model(self) -> newsletter::Model {
        match self {
            Self::Created(model) | Self::Reactivated(model) | Self::AlreadySubscribed(model) => {
                model
            }
        }
    }

    /// True when this call changed the subscription, so the caller knows the
    /// address is a new or returning subscriber rather than a duplicate.
    pub fn is_new(&self) -> bool {
        matches!(self, Self::Created(_) | Self::Reactivated(_))
    }
}

#[cfg_attr(test, mockall::automock)]
#[allow(async_fn_in_trait)]
pub trait NewsletterRepositoryExt {
    async fn subscribe(&self, email: &str) -> Result<SubscribeOutcome, DatabaseError>;

    async fn unsubscribe(&self, email: &str) -> Result<newsletter::Model, DatabaseError>;
}

#[derive(Debug, Clone)]
pub struct NewsletterRepository {
    db: DatabaseConnection,
}

impl Repository for NewsletterRepository {
    fn init(db: &DatabaseConnection) -> Self {
        Self { db: db.clone() }
    }
}

impl NewsletterRepositoryExt for NewsletterRepository {
    /// Idempotent: an address that is already subscribed is returned as-is, and
    /// one that previously unsubscribed is reactivated.
    ///
    /// `email` carries a unique key, so a plain insert would fail for both
    /// cases and surface as a duplicate-entry error to the caller. The
    /// [`SubscribeOutcome`] distinguishes the three cases so the caller can tell
    /// a fresh subscriber from a repeat submission.
    async fn subscribe(&self, email: &str) -> Result<SubscribeOutcome, DatabaseError> {
        let email = email.trim();

        if let Some(subscriber) = newsletter::Entity::find()
            .filter(newsletter::Column::Email.eq(email))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
        {
            if subscriber.subscribed {
                return Ok(SubscribeOutcome::AlreadySubscribed(subscriber));
            }

            let mut subscriber: newsletter::ActiveModel = subscriber.into();
            subscriber.subscribed = Set(true);
            return subscriber
                .update(&self.db)
                .await
                .map(SubscribeOutcome::Reactivated)
                .map_err(DatabaseError::from);
        }

        let model = newsletter::ActiveModel {
            identifier: Set(Ulid::new().to_string()),
            email: Set(email.to_owned()),
            subscribed: Set(true),
        };

        model
            .insert(&self.db)
            .await
            .map(SubscribeOutcome::Created)
            .map_err(|err: DbErr| {
                let msg = err.to_string();
                if msg.contains("UNIQUE constraint failed") || msg.contains("duplicate key") {
                    DatabaseError::DuplicateEntry(email.to_string())
                } else {
                    DatabaseError::from(err)
                }
            })
    }

    async fn unsubscribe(&self, email: &str) -> Result<newsletter::Model, DatabaseError> {
        let Some(subscriber) = newsletter::Entity::find()
            .filter(newsletter::Column::Email.eq(email.trim()))
            .one(&self.db)
            .await
            .map_err(DatabaseError::from)?
        else {
            return Err(DatabaseError::NotFound(email.trim().to_owned()));
        };

        let mut subscriber: newsletter::ActiveModel = subscriber.into();
        subscriber.subscribed = Set(false);
        subscriber
            .update(&self.db)
            .await
            .map_err(DatabaseError::from)
    }
}
