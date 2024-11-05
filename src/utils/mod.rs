mod json;
mod time;
mod traits;

pub use json::*;
pub use time::*;
pub use traits::*;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum UtilError {
    #[error("JSON parsing error: {0}")]
    JsonError(#[from] serde_json::Error),

    #[error("Time parsing error: {0}")]
    TimeError(String),

    #[error("Conversion error: {0}")]
    ConversionError(String),
}

pub type Result<T> = std::result::Result<T, UtilError>;