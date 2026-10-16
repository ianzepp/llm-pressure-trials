pub struct ProviderResponse {
    pub content: String,
    pub tokens_in: Option<u32>,
    pub tokens_out: Option<u32>,
    pub latency_ms: u128,
}
