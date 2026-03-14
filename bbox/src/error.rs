use thiserror::Error;

/// All errors that can be returned by the Bbox API client.
#[derive(Debug, Error)]
pub enum BboxError {
    /// General API error, optionally carrying an HTTP status code.
    #[error("{message}")]
    Api {
        message: String,
        status_code: Option<u16>,
    },

    /// Authentication error (base variant).
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Invalid credentials (HTTP 401 during login).
    #[error("Invalid credentials: {0}")]
    InvalidCredentials(String),

    /// Session expired (HTTP 401 on an authenticated request).
    #[error("Session expired: {0}")]
    SessionExpired(String),

    /// Rate limit exceeded (HTTP 429).
    #[error("Rate limit exceeded: {0}")]
    RateLimit(String),

    /// Client used before calling `authenticate()`.
    #[error("Not authenticated: {0}")]
    Unauthenticated(String),

    /// Request or authentication timed out.
    #[error("Timeout: {0}")]
    Timeout(String),

    /// Underlying HTTP transport error.
    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    /// JSON parsing error.
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, BboxError>;
