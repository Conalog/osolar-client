use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt;
use std::io;
use thiserror::Error;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsolarApiErrorBody {
    pub success: Option<bool>,
    pub message: Option<String>,
    #[serde(flatten)]
    pub extra: Value,
}

#[derive(Debug, Clone)]
pub enum OsolarErrorPayload {
    Json(OsolarApiErrorBody),
    Text(String),
}

impl fmt::Display for OsolarErrorPayload {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Json(body) => {
                let s = serde_json::to_string(body).unwrap_or_else(|_| "invalid json".to_string());
                write!(f, "{}", s)
            }
            Self::Text(text) => write!(f, "{}", text),
        }
    }
}

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

    #[error(
        "insecure base url: {base_url} (https required; call allow_insecure_http() to override)"
    )]
    InsecureBaseUrl { base_url: String },

    #[error("response too large (content_length={content_length:?}, limit={limit_bytes} bytes)")]
    ResponseTooLarge {
        content_length: Option<u64>,
        limit_bytes: u64,
    },

    #[error("osolar api returned status {status}: {body}")]
    Http {
        status: u16,
        body: OsolarErrorPayload,
    },
}
