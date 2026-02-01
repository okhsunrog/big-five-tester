//! AI pipeline error types.

use thiserror::Error;

/// Errors that can occur during AI analysis.
#[derive(Debug, Error)]
pub enum AnalysisError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(#[from] crate::config::ConfigError),

    /// API request failed
    #[error("API request failed: {0}")]
    Request(String),

    /// API returned an error response
    #[error("API error ({status}): {body}")]
    ApiError { status: u16, body: String },

    /// Failed to parse API response
    #[error("Failed to parse API response: {0}")]
    ParseResponse(String),

    /// Empty response from API
    #[error("Empty response from API")]
    EmptyResponse,

    /// User input was flagged as unsafe (prompt injection)
    #[error(
        "Your input was flagged as potentially unsafe. Please provide only personal context information."
    )]
    UnsafeInput,

    /// Invalid model ID
    #[error("Invalid model: {0}")]
    InvalidModel(String),
}
