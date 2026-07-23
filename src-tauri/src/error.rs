//! Metric error type shared across providers and commands.

use serde::Serialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[allow(dead_code)] // Unsupported used by Linux/Windows stubs (cfg-gated)
pub enum MetricError {
    #[error("metric unavailable: {0}")]
    Unavailable(String),
    #[error("platform not supported: {0}")]
    Unsupported(String),
    #[error("internal error: {0}")]
    Internal(String),
}

impl Serialize for MetricError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}
