use thiserror::Error;

#[derive(Debug, Error)]
pub enum AiError {
    #[error("no active AI provider is configured")]
    MissingProvider,
    #[error("missing API key for provider")]
    MissingApiKey,
    #[error("invalid provider URL: {0}")]
    InvalidUrl(String),
    #[error("http request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("provider returned an invalid response: {0}")]
    InvalidResponse(String),
    #[error("keychain operation failed: {0}")]
    SecretStore(String),
    #[error("settings operation failed: {0}")]
    Settings(String),
}

pub type Result<T> = std::result::Result<T, AiError>;
