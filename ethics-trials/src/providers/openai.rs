use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::providers::types::ProviderResponse;

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";
const MAX_RETRIES: usize = 5;
const BASE_BACKOFF_MS: u64 = 500;
const REQUEST_TIMEOUT_SECS: u64 = 90;

#[derive(Clone)]
pub struct OpenAiCompatibleClient {
    api_key: String,
    base_url: String,
    agent: ureq::Agent,
}

impl OpenAiCompatibleClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY is not set in the environment")?;
        Ok(Self {
            api_key,
            base_url: OPENROUTER_BASE_URL.to_owned(),
            agent: ureq::AgentBuilder::new()
                .timeout(Duration::from_secs(REQUEST_TIMEOUT_SECS))
                .build(),
        })
    }

    pub fn chat_completion(
        &self,
        model: &str,
        system: Option<&str>,
        context: Option<&str>,
        user: &str,
    ) -> Result<ProviderResponse> {
        let mut messages = Vec::new();

        if let Some(system_prompt) = system {
            messages.push(ChatMessage {
                role: "system".to_owned(),
                content: system_prompt.to_owned(),
            });
        }

        let user_content = if let Some(context_block) = context {
            format!("{context_block}\n\n{user}")
        } else {
            user.to_owned()
        };
        messages.push(ChatMessage {
            role: "user".to_owned(),
            content: user_content,
        });

        let request = ChatCompletionRequest {
            model: model.to_owned(),
            messages,
            temperature: Some(0.0),
            max_tokens: Some(800),
        };

        let started = Instant::now();
        let request_body = serde_json::to_value(&request)?;
        let response = self.send_with_retry(|client| {
            client
                .agent
                .post(&format!("{}/chat/completions", client.base_url))
                .set("Authorization", &format!("Bearer {}", client.api_key))
                .set("Content-Type", "application/json")
                .set("HTTP-Referer", "https://github.com/ianzepp/ethics-trials")
                .set("X-Title", "Ethics Trials")
                .send_json(request_body.clone())
        })?;

        let latency_ms = started.elapsed().as_millis();

        let parsed: ChatCompletionResponse = response
            .into_json()
            .context("failed to parse provider response body")?;
        let content = parsed
            .choices
            .first()
            .map(|item| item.message.content.clone())
            .unwrap_or_default();

        Ok(ProviderResponse {
            content,
            tokens_in: parsed.usage.as_ref().and_then(|item| item.prompt_tokens),
            tokens_out: parsed
                .usage
                .as_ref()
                .and_then(|item| item.completion_tokens),
            latency_ms,
        })
    }

    pub fn list_models(&self) -> Result<OpenRouterModelsResponse> {
        let response = self.send_with_retry(|client| {
            client
                .agent
                .get(&format!("{}/models", client.base_url))
                .set("Authorization", &format!("Bearer {}", client.api_key))
                .set("HTTP-Referer", "https://github.com/ianzepp/ethics-trials")
                .set("X-Title", "Ethics Trials")
                .call()
        })?;

        response
            .into_json()
            .context("failed to parse models response body")
    }

    fn send_with_retry<F>(&self, mut operation: F) -> Result<ureq::Response>
    where
        F: FnMut(&Self) -> std::result::Result<ureq::Response, ureq::Error>,
    {
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            match operation(self) {
                Ok(response) => return Ok(response),
                Err(error) => {
                    let retryable = is_retryable(&error);
                    let mapped = map_ureq_error(error);
                    if attempt + 1 == MAX_RETRIES || !retryable {
                        return Err(mapped);
                    }
                    last_error = Some(mapped);
                    std::thread::sleep(Duration::from_millis(
                        BASE_BACKOFF_MS * 2_u64.pow(attempt as u32),
                    ));
                }
            }
        }

        Err(last_error.unwrap_or_else(|| anyhow::anyhow!("request failed after retries")))
    }
}

fn map_ureq_error(error: ureq::Error) -> anyhow::Error {
    match error {
        ureq::Error::Status(status, response) => {
            let status_text = response.status_text().to_owned();
            let body = response
                .into_string()
                .unwrap_or_else(|_| "<unreadable response body>".to_owned());
            anyhow::anyhow!(
                "provider returned HTTP {} {}: {}",
                status,
                status_text,
                truncate(&body, 500)
            )
        }
        ureq::Error::Transport(transport) => {
            anyhow::anyhow!("provider transport error: {}", transport)
        }
    }
}

fn is_retryable(error: &ureq::Error) -> bool {
    match error {
        ureq::Error::Status(status, _) => *status == 429 || *status >= 500,
        ureq::Error::Transport(_) => true,
    }
}

fn truncate(value: &str, limit: usize) -> String {
    if value.len() <= limit {
        value.to_owned()
    } else {
        let boundary = value
            .char_indices()
            .map(|(idx, _)| idx)
            .take_while(|idx| *idx < limit)
            .last()
            .unwrap_or(0);

        if boundary == 0 {
            "...".to_owned()
        } else {
            format!("{}...", &value[..boundary])
        }
    }
}

#[derive(Debug, Serialize)]
struct ChatCompletionRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChatCompletionResponse {
    choices: Vec<ChatChoice>,
    #[serde(default)]
    usage: Option<ChatUsage>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct ChatUsage {
    #[serde(default)]
    prompt_tokens: Option<u32>,
    #[serde(default)]
    completion_tokens: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterModelsResponse {
    pub data: Vec<OpenRouterModel>,
}

#[derive(Debug, Deserialize)]
pub struct OpenRouterModel {
    pub id: String,
    #[serde(default)]
    pub canonical_slug: Option<String>,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub created: Option<i64>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub context_length: Option<u64>,
    #[serde(default)]
    pub architecture: Option<serde_json::Value>,
    #[serde(default)]
    pub pricing: Option<serde_json::Value>,
    #[serde(default)]
    pub top_provider: Option<serde_json::Value>,
    #[serde(default)]
    pub per_request_limits: Option<serde_json::Value>,
}

#[cfg(test)]
#[path = "openai_test.rs"]
mod tests;
