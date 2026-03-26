use std::time::Instant;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::providers::types::ProviderResponse;

const OPENROUTER_BASE_URL: &str = "https://openrouter.ai/api/v1";

pub struct OpenAiCompatibleClient {
    api_key: String,
    base_url: String,
}

impl OpenAiCompatibleClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("OPENROUTER_API_KEY")
            .context("OPENROUTER_API_KEY is not set in the environment")?;
        Ok(Self {
            api_key,
            base_url: OPENROUTER_BASE_URL.to_owned(),
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
        let response = ureq::post(&format!("{}/chat/completions", self.base_url))
            .set("Authorization", &format!("Bearer {}", self.api_key))
            .set("Content-Type", "application/json")
            .set("HTTP-Referer", "https://github.com/ianzepp/ethics-trials")
            .set("X-Title", "Ethics Trials")
            .send_json(serde_json::to_value(&request)?)
            .map_err(map_ureq_error)?;

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
            tokens_out: parsed.usage.as_ref().and_then(|item| item.completion_tokens),
            latency_ms,
        })
    }
}

fn map_ureq_error(error: ureq::Error) -> anyhow::Error {
    match error {
        ureq::Error::Status(status, response) => {
            let status_text = response.status_text().to_owned();
            anyhow::anyhow!("provider returned HTTP {} {}", status, status_text)
        }
        ureq::Error::Transport(transport) => anyhow::anyhow!("provider transport error: {}", transport),
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
