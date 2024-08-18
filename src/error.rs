use reqwest::StatusCode;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiClientError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid response status: {0}")]
    HttpStatus(StatusCode),

    #[error("Failed to parse JSON response: {0}")]
    JsonParse(#[from] serde_json::Error),

    #[error("Failed to parse JSON response: {0}")]
    DeserializeError(String),

    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    #[error("API returned an error: status {status}, body {body}")]
    ApiError {
        status: StatusCode,
        body: String,
    },

    #[error("Unexpected error: {0}")]
    Unexpected(String),

    #[error("Maximum retries reached")]
    MaxRetriesReached,
}
