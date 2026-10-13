use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Operation failed: {0}")]
    OperationFailed(String),
}
