mod api;
mod types;

pub use api::Client;
pub use types::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum XError {
    #[error("Authentication failed: {0}")]
    AuthError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("API request failed: {0}")]
    ApiError(String),

    #[error("Invalid response format: {0}")]
    ParseError(String),

    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Cookie authentication failed: {0}")]
    CookieAuthError(String),
}

pub type Result<T> = std::result::Result<T, XError>;