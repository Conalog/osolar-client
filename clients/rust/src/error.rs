use serde_json::Value;
use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("request failed: {0}")]
    Request(#[from] reqwest::Error),

    #[error("io failed: {0}")]
    Io(#[from] io::Error),

    #[error("response parse failed: {0}")]
    Parse(#[from] serde_json::Error),

    #[error("invalid base url: {base_url}")]
    InvalidBaseUrl { base_url: String },

    #[error("insecure base url: {base_url} (https required; call allow_insecure_http() to override)")]
    InsecureBaseUrl { base_url: String },

    #[error("response too large (content_length={content_length:?}, limit={limit_bytes} bytes)")]
    ResponseTooLarge {
        content_length: Option<u64>,
        limit_bytes: u64,
    },

    #[error("osolar api returned status {status}: {body}")]
    Http { status: u16, body: Value },
}
