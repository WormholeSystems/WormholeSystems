use thiserror::Error;

#[derive(Debug, Error)]
pub enum EsiError {
    #[error("ESI request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("ESI returned {status}: {body}")]
    Api { status: u16, body: String },
    /// ESI said to slow down (or nearly did), so the request was not sent.
    #[error("backing off from ESI for another {0}s")]
    Backoff(u64),
    #[error("could not read ESI's reply: {0}")]
    Decode(#[from] serde_json::Error),
    #[error("SSO error: {0}")]
    Auth(String),
    /// EVE refused the refresh token (`invalid_grant`): access was revoked or the token
    /// rotated away. Retrying cannot help; the character has to log in again.
    #[error("EVE refused the refresh token, the character has to log in again")]
    TokenRejected,
    #[error("token validation failed: {0}")]
    Jwt(String),
    #[error("missing required scope: {0}")]
    MissingScope(String),
    #[error("token store error: {0}")]
    Store(String),
}

pub type Result<T> = std::result::Result<T, EsiError>;
