use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("response parse failed: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("osolar api returned status {status}: {body}")]
    Http { status: u16, body: Value },
}
