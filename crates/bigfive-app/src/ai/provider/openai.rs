//! OpenAI-compatible API provider.
//!
//! Works with OpenRouter, OpenAI, Ollama, and other OpenAI-compatible endpoints.

use serde::{Deserialize, Serialize};

use crate::ai::error::AnalysisError;
use crate::config::ApiConfig;

use super::anthropic::API_TIMEOUT;

// ── Request types ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

// ── Response types ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct Choice {
    message: ChoiceMessage,
}

#[derive(Deserialize)]
struct ChoiceMessage {
    content: String,
}

#[derive(Deserialize)]
struct Response {
    choices: Vec<Choice>,
}

// ── Public API ──────────────────────────────────────────────────────────

/// Call an OpenAI-compatible API endpoint.
pub async fn call(
    api: &ApiConfig,
    model: &str,
    system: Option<&str>,
    user: &str,
    max_tokens: u32,
) -> Result<String, AnalysisError> {
    let api_key = api.api_key()?;
    let api_url = api.api_url.as_ref().ok_or_else(|| {
        AnalysisError::Config(crate::config::ConfigError::Validation(
            "api_url is required for OpenAI-compatible provider".to_string(),
        ))
    })?;

    let mut messages = Vec::with_capacity(2);
    if let Some(sys) = system {
        messages.push(Message {
            role: "system",
            content: sys,
        });
    }
    messages.push(Message {
        role: "user",
        content: user,
    });

    let request = Request {
        model,
        max_tokens,
        messages,
    };

    let client = reqwest::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .map_err(|e| AnalysisError::Request(e.to_string()))?;

    let response = client
        .post(api_url)
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await
        .map_err(|e| AnalysisError::Request(e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status().as_u16();
        let body = response.text().await.unwrap_or_default();
        return Err(AnalysisError::ApiError { status, body });
    }

    let api_response: Response = response
        .json()
        .await
        .map_err(|e| AnalysisError::ParseResponse(e.to_string()))?;

    api_response
        .choices
        .into_iter()
        .next()
        .map(|choice| choice.message.content)
        .ok_or(AnalysisError::EmptyResponse)
}
