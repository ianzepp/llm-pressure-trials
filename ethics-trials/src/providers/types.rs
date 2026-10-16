use std::fmt;

/// A successfully parsed chat completion.
///
/// `content` is `""` when the provider returned `message.content: null`; in
/// that case `content_null` is `true`. `raw_body` carries the full JSON text of
/// the HTTP response, but only for anomalous responses (null/empty content, or
/// a finish_reason other than "stop") to keep stored data small.
#[derive(Debug, Clone)]
pub struct ProviderResponse {
    pub content: String,
    pub tokens_in: Option<u32>,
    pub tokens_out: Option<u32>,
    pub latency_ms: u128,
    pub finish_reason: Option<String>,
    pub native_finish_reason: Option<String>,
    pub refusal: Option<String>,
    pub reasoning: Option<String>,
    pub content_null: bool,
    pub raw_body: Option<String>,
}

/// An error from the provider that may carry the raw HTTP response body.
///
/// Travels inside `anyhow::Error`; the orchestrator downcasts it to persist
/// `raw_body` alongside the failed trial.
#[derive(Debug)]
pub struct ProviderError {
    pub message: String,
    pub raw_body: Option<String>,
}

impl fmt::Display for ProviderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for ProviderError {}

/// Extract the raw provider body from an error chain, if one was attached.
pub fn raw_body_of(error: &anyhow::Error) -> Option<String> {
    error
        .downcast_ref::<ProviderError>()
        .and_then(|item| item.raw_body.clone())
}
