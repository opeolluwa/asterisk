use std::fmt;

#[derive(Debug)]
pub enum ImagekitError {
    UploadFailed(String),
    IoError(String),
    RequestError(String),
}

impl fmt::Display for ImagekitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImagekitError::UploadFailed(msg) => write!(f, "Upload failed: {}", msg),
            ImagekitError::IoError(msg) => write!(f, "I/O error: {}", msg),
            ImagekitError::RequestError(msg) => write!(f, "Request error: {}", msg),
        }
    }
}

impl std::error::Error for ImagekitError {}
