use std::sync::Arc;

use axum::{
    RequestPartsExt,
    extract::{FromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
use axum_extra::{
    TypedHeader,
    headers::{Authorization, authorization::Bearer},
};
use jsonwebtoken::{Algorithm, Validation, decode};

use crate::dto::claims::{Claims, EntityKind, Keys, TokenType};
use crate::errors::authentication_error::AuthenticationError;
use crate::errors::service_error::ServiceError;
use crate::services::redis::client::RedisEngineTrait;
use crate::services::user_services::UserServiceTrait;
use crate::state::AppState;
use crate::utils::extract_env::extract_env;

fn jwt_validation() -> Validation {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.set_required_spec_claims(&["exp", "iat"]);
    validation
}

/// Decodes the bearer token in `parts` into claims.
///
/// A missing header is reported as `Ok(None)` so callers can treat the request
/// as anonymous. A header that is present but does not decode is an error:
/// silently downgrading a caller with a stale token to an anonymous one would
/// make their writes land in a throwaway guest cart instead of their real one.
async fn extract_claims(
    parts: &mut Parts,
) -> Result<Option<(Claims, String)>, AuthenticationError> {
    let TypedHeader(Authorization(bearer)) =
        match parts.extract::<TypedHeader<Authorization<Bearer>>>().await {
            Ok(header) => header,
            Err(_) => return Ok(None),
        };

    let secret = extract_env::<String>("JWT_SIGNING_KEY").map_err(AuthenticationError::from)?;
    let decoding_key = Keys::new(secret.as_bytes()).decoding;

    let token = bearer.token();
    let token_data = decode::<Claims>(token, &decoding_key, &jwt_validation()).map_err(|_| {
        tracing::warn!("rejecting request with an undecodable bearer token");
        AuthenticationError::InvalidToken
    })?;

    Ok(Some((token_data.claims, token.to_string())))
}

impl<S> FromRequestParts<S> for Claims
where
    S: Send + Sync,
{
    type Rejection = AuthenticationError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let secret = extract_env::<String>("JWT_SIGNING_KEY").map_err(AuthenticationError::from)?;

        let decoding_key = Keys::new(secret.as_bytes()).decoding;
        let TypedHeader(Authorization(bearer)) = parts
            .extract::<TypedHeader<Authorization<Bearer>>>()
            .await
            .map_err(|_| AuthenticationError::MissingCredentials)?;
        let token_data = decode::<Claims>(bearer.token(), &decoding_key, &jwt_validation())
            .map_err(|_| AuthenticationError::InvalidToken)?;

        Ok(token_data.claims)
    }
}

/// [`Claims`] that are additionally proven to belong to an active staff account.
///
/// [`Claims`] alone only proves the caller holds a token this service signed.
/// Customer and staff tokens share the same `Claims { email, identifier,
/// token_type }` shape and the same `JWT_SIGNING_KEY` (see
/// `customer_service.rs`), so a customer token authenticates every staff route.
/// This extractor resolves `identifier` against the `users` table and 403s
/// customers.
///
/// Scoped to the routes that need it: `role_permissions_service` is wired into
/// `AppState` but nothing consults it, so the other staff nests still carry this
/// gap.
#[derive(Debug, Clone)]
pub struct StaffClaims(pub Claims);

impl FromRequestParts<Arc<AppState>> for StaffClaims {
    type Rejection = ServiceError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Delegates so token decoding stays in one place and a malformed token
        // still reports as 401 rather than being flattened into the 403 below.
        let claims = Claims::from_request_parts(parts, state)
            .await
            .map_err(ServiceError::from)?;

        if state
            .services
            .user_service
            .is_active_staff(&claims.identifier)
            .await?
        {
            Ok(StaffClaims(claims))
        } else {
            Err(ServiceError::Forbidden)
        }
    }
}

/// Claims for a route whose authentication is optional.
///
/// Reads the claims [`optional_authenticate`] already decoded and validated
/// instead of parsing the header again, so the middleware stays the single place
/// that decides whether a token is acceptable. This never fails: absence of a
/// token is a legitimate anonymous request, not a rejection.
///
/// Requires [`optional_authenticate`] on the route, otherwise it is always
/// `None`.
#[derive(Debug, Clone)]
pub struct OptionalClaims(pub Option<Claims>);

impl<S> FromRequestParts<S> for OptionalClaims
where
    S: Send + Sync,
{
    type Rejection = std::convert::Infallible;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        Ok(Self(parts.extensions.get::<Claims>().cloned()))
    }
}

async fn authenticate_inner(
    state: &AppState,
    parts: &mut axum::http::request::Parts,
    allowed_types: &[TokenType],
) -> Result<(Claims, String), AuthenticationError> {
    let (claims, token) = extract_claims(parts)
        .await?
        .ok_or(AuthenticationError::MissingCredentials)?;

    let is_blacklisted = state
        .redis
        .is_token_blacklisted(&token)
        .map_err(|e| AuthenticationError::OperationFailed(e.to_string()))?;

    if is_blacklisted {
        return Err(AuthenticationError::InvalidToken);
    }

    if !allowed_types.contains(&claims.token_type) {
        return Err(AuthenticationError::InvalidToken);
    }

    Ok((claims, token))
}

pub async fn authenticate(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AuthenticationError> {
    let (mut parts, body) = request.into_parts();

    let (claims, token) = authenticate_inner(&state, &mut parts, &[TokenType::Access]).await?;

    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(RawToken(token));

    Ok(next.run(request).await)
}

pub async fn authenticate_challenge(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AuthenticationError> {
    let (mut parts, body) = request.into_parts();

    let (claims, token) = authenticate_inner(
        &state,
        &mut parts,
        &[
            TokenType::Access,
            TokenType::TwoFactor,
            TokenType::PasswordReset,
        ],
    )
    .await?;

    let mut request = Request::from_parts(parts, body);
    request.extensions_mut().insert(claims);
    request.extensions_mut().insert(RawToken(token));

    Ok(next.run(request).await)
}

/// Same token handling as [`authenticate`], but a request with no `Authorization`
/// header is allowed through unauthenticated instead of being rejected.
///
/// Handlers read the result with [`OptionalClaims`], which reads these
/// extensions rather than re-decoding the header.
pub async fn optional_authenticate(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, AuthenticationError> {
    let (parts, body) = request.into_parts();

    let has_header = parts
        .headers
        .contains_key(axum::http::header::AUTHORIZATION);

    let mut request = Request::from_parts(parts, body);

    if has_header {
        let (mut parts, body) = request.into_parts();
        let (claims, token) = authenticate_inner(&state, &mut parts, &[TokenType::Access]).await?;
        request = Request::from_parts(parts, body);
        request.extensions_mut().insert(claims);
        request.extensions_mut().insert(RawToken(token));
    }

    Ok(next.run(request).await)
}

fn check_entity(parts: &Parts, expected: EntityKind) -> Result<(), ServiceError> {
    let Some(claims) = parts.extensions.get::<Claims>() else {
        tracing::error!(
            expected = ?expected,
            "entity gate reached without claims; authenticate must be layered outside it"
        );
        return Err(ServiceError::Forbidden);
    };

    if claims.target_entity == expected {
        return Ok(());
    }

    tracing::warn!(
        expected = ?expected,
        actual = ?claims.target_entity,
        "rejecting a token minted for the other entity kind"
    );

    Err(ServiceError::Forbidden)
}

pub async fn require_staff(
    State(state): State<Arc<AppState>>,
    request: Request,
    next: Next,
) -> Result<Response, ServiceError> {
    let (parts, body) = request.into_parts();
    check_entity(&parts, EntityKind::User)?;

    let claims = parts.extensions.get::<Claims>().unwrap();
    if !state
        .services
        .user_service
        .is_active_staff(&claims.identifier)
        .await?
    {
        return Err(ServiceError::Forbidden);
    }

    Ok(next.run(Request::from_parts(parts, body)).await)
}

pub async fn require_customer(request: Request, next: Next) -> Result<Response, ServiceError> {
    let (parts, body) = request.into_parts();
    check_entity(&parts, EntityKind::Customer)?;
    Ok(next.run(Request::from_parts(parts, body)).await)
}

#[derive(Clone)]
pub struct RawToken(pub String);

impl<S> FromRequestParts<S> for RawToken
where
    S: Send + Sync,
{
    type Rejection = AuthenticationError;

    /// Reads the token [`authenticate`] or [`authenticate_challenge`] already
    /// stashed, so a handler can hand it to a service (single-use blacklisting)
    /// without consuming the request body.
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<RawToken>()
            .cloned()
            .ok_or(AuthenticationError::MissingCredentials)
    }
}

/// The header a guest sends to name the cart they are holding.
pub const CART_IDENTIFIER_HEADER: &str = "x-cart-identifier";

/// Reads the guest cart identifier from the request headers.
///
/// A header that is present but blank is treated as absent: it is more likely a
/// client echoing back an empty identifier than a deliberate lookup for the cart
/// whose identifier is the empty string.
pub fn cart_identifier_header(headers: &axum::http::HeaderMap) -> Option<&str> {
    headers
        .get(CART_IDENTIFIER_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
}

#[cfg(test)]
mod tests {
    use axum::body::Body;

    use super::*;
    use crate::dto::claims::TokenType;

    fn claims(kind: EntityKind) -> Claims {
        Claims {
            email: "someone@example.com".to_string(),
            identifier: "someone".to_string(),
            token_type: TokenType::Access,
            target_entity: kind,
        }
    }

    fn parts_with(kind: Option<EntityKind>) -> Parts {
        let mut request = Request::new(Body::empty());
        if let Some(kind) = kind {
            request.extensions_mut().insert(claims(kind));
        }
        request.into_parts().0
    }

    #[test]
    fn a_staff_token_passes_the_staff_gate() {
        assert!(check_entity(&parts_with(Some(EntityKind::User)), EntityKind::User).is_ok());
    }

    #[test]
    fn a_customer_token_passes_the_customer_gate() {
        assert!(
            check_entity(
                &parts_with(Some(EntityKind::Customer)),
                EntityKind::Customer
            )
            .is_ok()
        );
    }

    #[test]
    fn a_customer_token_is_refused_the_staff_gate() {
        let outcome = check_entity(&parts_with(Some(EntityKind::Customer)), EntityKind::User);
        assert!(matches!(outcome, Err(ServiceError::Forbidden)));
    }

    #[test]
    fn a_staff_token_is_refused_the_customer_gate() {
        let outcome = check_entity(&parts_with(Some(EntityKind::User)), EntityKind::Customer);
        assert!(matches!(outcome, Err(ServiceError::Forbidden)));
    }

    #[test]
    fn a_request_with_no_claims_is_refused_by_both_gates() {
        for expected in [EntityKind::User, EntityKind::Customer] {
            let outcome = check_entity(&parts_with(None), expected);
            assert!(matches!(outcome, Err(ServiceError::Forbidden)));
        }
    }
}
