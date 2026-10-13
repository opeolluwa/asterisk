use std::sync::Arc;

use axum::{
    body::Body,
    extract::Request,
    http::{Method, StatusCode},
    response::Response,
};
use tower::Layer;

#[derive(Clone)]
pub struct CsrfLayer {
    allowed_origins: Arc<Vec<String>>,
    enforce: bool,
}

impl CsrfLayer {
    /// `enforce` is false outside production so local development is not gated on
    /// the console's origin staying in the allowlist. It mirrors how
    /// [`crate::config::cors::init_cors`] relaxes `allow_origin` per environment;
    /// production keeps the strict behaviour on both layers.
    pub fn new(allowed_origins: Vec<String>, enforce: bool) -> Self {
        Self {
            allowed_origins: Arc::new(allowed_origins),
            enforce,
        }
    }
}

impl<S> Layer<S> for CsrfLayer {
    type Service = CsrfService<S>;

    fn layer(&self, inner: S) -> Self::Service {
        CsrfService {
            inner,
            allowed_origins: self.allowed_origins.clone(),
            enforce: self.enforce,
        }
    }
}

#[derive(Clone)]
pub struct CsrfService<S> {
    inner: S,
    allowed_origins: Arc<Vec<String>>,
    enforce: bool,
}

/// Whether a mutating request may proceed.
///
/// - `Content-Type: application/json` is not a pass. Treating it as one makes the
///   check inversely proportional to risk: the JSON writes that change data skip
///   it while a bodyless `PUT` is held to the allowlist. (`application/json` is not
///   CORS-safelisted, so a cross-origin JSON write is already stopped at preflight.)
/// - A `Referer` that will not parse is not accepted. `Option::is_none_or` returns
///   `true` for `None`, which would read as "allowed".
///
/// An absent `Origin` *and* `Referer` is allowed: every browser-initiated
/// cross-origin request carries an `Origin`, so neither header means the caller is
/// not a browser and there is no ambient credential for a malicious page to ride.
fn is_origin_allowed(
    method: &Method,
    origin: Option<&str>,
    referer: Option<&str>,
    allowed: &[String],
    enforce: bool,
) -> bool {
    if matches!(*method, Method::GET | Method::HEAD | Method::OPTIONS) {
        return true;
    }

    if !enforce {
        return true;
    }

    match (origin, referer) {
        // No browser indicators at all: the caller is not a browser.
        (None, None) => true,
        (Some(origin), _) => allowed.iter().any(|a| a == origin),
        // A referer that cannot be parsed is a present-but-unmatchable origin,
        // not an absent one, so `is_some_and` rather than `is_none_or` here.
        (None, Some(referer)) => {
            extract_origin(referer).is_some_and(|candidate| allowed.contains(&candidate))
        }
    }
}

impl<S> tower::Service<Request<Body>> for CsrfService<S>
where
    S: tower::Service<Request<Body>, Response = Response> + Clone + Send + 'static,
    S::Future: Send,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = std::pin::Pin<
        Box<dyn std::future::Future<Output = Result<Self::Response, Self::Error>> + Send>,
    >;

    fn poll_ready(
        &mut self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        let method = req.method().clone();
        let origin = first_header(&req, "origin");
        let referer = first_header(&req, "referer");

        let allowed = self.allowed_origins.clone();
        let enforce = self.enforce;
        let mut inner = self.inner.clone();

        Box::pin(async move {
            let is_valid = is_origin_allowed(
                &method,
                origin.as_deref(),
                referer.as_deref(),
                &allowed,
                enforce,
            );

            if is_valid {
                inner.call(req).await
            } else {
                tracing::warn!(
                    method = %method,
                    ?origin,
                    ?referer,
                    "CSRF check failed"
                );
                Ok(Response::builder()
                    .status(StatusCode::FORBIDDEN)
                    .body(axum::body::Body::from("forbidden"))
                    .unwrap())
            }
        })
    }
}

fn first_header(req: &Request<Body>, name: &str) -> Option<String> {
    req.headers()
        .get_all(name)
        .iter()
        .find_map(|v| v.to_str().ok().map(String::from))
}

fn extract_origin(url: &str) -> Option<String> {
    let end = url.find("://")?;
    let scheme = &url[..end];
    let rest = &url[end + 3..];
    let host_end = rest.find('/').unwrap_or(rest.len());
    let host = &rest[..host_end];
    Some(format!("{scheme}://{host}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn allowed() -> Vec<String> {
        vec!["http://localhost:5600".to_string()]
    }

    #[test]
    fn test_extract_origin() {
        assert_eq!(
            extract_origin("https://example.com/path?q=1"),
            Some("https://example.com".to_string())
        );
        assert_eq!(
            extract_origin("http://localhost:3000/dashboard"),
            Some("http://localhost:3000".to_string())
        );
        assert_eq!(extract_origin("not-a-url"), None);
    }

    #[test]
    fn safe_methods_are_allowed_whatever_the_origin() {
        for method in [Method::GET, Method::HEAD, Method::OPTIONS] {
            assert!(is_origin_allowed(
                &method,
                Some("https://evil.example"),
                None,
                &allowed(),
                true
            ));
        }
    }

    #[test]
    fn a_mutating_request_with_no_origin_or_referer_is_allowed() {
        // curl, Postman and integration tests send neither header. There is no
        // ambient credential for a malicious page to ride in that case.
        assert!(is_origin_allowed(
            &Method::PUT,
            None,
            None,
            &allowed(),
            true
        ));
    }

    #[test]
    fn an_allowlisted_origin_is_allowed() {
        assert!(is_origin_allowed(
            &Method::PUT,
            Some("http://localhost:5600"),
            None,
            &allowed(),
            true
        ));
    }

    #[test]
    fn a_foreign_origin_is_denied() {
        assert!(!is_origin_allowed(
            &Method::POST,
            Some("https://evil.example"),
            None,
            &allowed(),
            true
        ));
    }

    #[test]
    fn an_allowlisted_referer_is_accepted_and_a_foreign_one_is_not() {
        assert!(is_origin_allowed(
            &Method::PUT,
            None,
            Some("http://localhost:5600/orders"),
            &allowed(),
            true
        ));
        assert!(!is_origin_allowed(
            &Method::PUT,
            None,
            Some("https://evil.example/orders"),
            &allowed(),
            true
        ));
    }

    #[test]
    fn a_referer_that_cannot_be_parsed_is_denied_not_accepted() {
        // Regression: `Option::is_none_or` returned `true` when `extract_origin`
        // yielded `None`, so an unparseable referer read as an allowed origin.
        assert!(!is_origin_allowed(
            &Method::PUT,
            None,
            Some("not-a-url"),
            &allowed(),
            true
        ));
    }

    #[test]
    fn enforcement_off_permits_any_origin() {
        assert!(is_origin_allowed(
            &Method::POST,
            Some("https://evil.example"),
            None,
            &[],
            false
        ));
    }
}
