#[derive(Debug, Clone, thiserror::Error)]
pub enum PaymentError {
    #[error("invalid request")]
    InvalidRequest,
    #[error("unauthorized")]
    Unauthorized,
    #[error("not found")]
    NotFound,
    #[error("server error")]
    ServerError,
    #[error("unexpected error: ({0})")]
    UnexpectedError(String),
}
