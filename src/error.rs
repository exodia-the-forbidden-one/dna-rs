use thiserror::Error;

/// All errors that can be returned by the DNA client.
#[derive(Debug, Error)]
pub enum DnaError {
    /// HTTP transport / connection error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// The API returned a non-2xx status code.
    #[error("API error [{code}]: {message}")]
    Api {
        code: String,
        message: String,
        details: String,
    },

    /// The API response body could not be parsed.
    #[error("Failed to deserialize API response: {0}")]
    Deserialize(#[from] serde_json::Error),

    /// A required field was missing from the API response.
    #[error("Unexpected API response: {0}")]
    UnexpectedResponse(String),

    /// Invalid argument supplied by the caller.
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
}

/// Convenience alias.
pub type DnaResult<T> = Result<T, DnaError>;
