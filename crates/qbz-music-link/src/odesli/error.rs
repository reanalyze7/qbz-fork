//! Odesli client error type.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShareError {
    #[error("Network error: {0}")]
    NetworkError(#[from] reqwest::Error),

    #[error("Odesli API error: {0}")]
    OdesliError(String),

    #[error("No matches found on Odesli")]
    NoMatches,
}
