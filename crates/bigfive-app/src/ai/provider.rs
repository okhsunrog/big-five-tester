//! API provider implementations.

use std::time::{Duration, Instant};

use serde::{Deserialize, Serialize};
use tracing::{debug, info, instrument, warn};

use crate::config::{ApiConfig, Provider};

use super::error::AnalysisError;

/// Default timeout for API calls (3 minutes to allow for slow Claude Opus responses)
const API_TIMEOUT: Duration = Duration::from_secs(180);

/// Call an AI model with the given prompt.
#[instrument(skip(api, prompt), fields(model = %model, max_tokens = %max_tokens, provider = ?api.provider))]
pub async fn call_model(
    api: &ApiConfig,
    model: &str,
    prompt: &str,
    max_tokens: u32,
) -> Result<String, AnalysisError> {
    debug!(prompt_len = prompt.len(), "Calling model");
    let start = Instant::now();
    let result = match api.provider {
        Provider::Anthropic => call_anthropic(api, model, None, prompt, max_tokens).await,
        Provider::OpenAiCompatible => {
            call_openai_compatible(api, model, None, prompt, max_tokens).await
        }
    };
    let elapsed = start.elapsed();
    match &result {
        Ok(response) => info!(
            response_len = response.len(),
            elapsed_ms = elapsed.as_millis(),
            "Model call succeeded"
        ),
        Err(e) => warn!(error = %e, elapsed_ms = elapsed.as_millis(), "Model call failed"),
    }
    result
}

/// Call an AI model with system and user messages.
#[instrument(skip(api, system, user), fields(model = %model, max_tokens = %max_tokens, provider = ?api.provider))]
pub async fn call_model_with_system(
    api: &ApiConfig,
    model: &str,
    system: &str,
    user: &str,
    max_tokens: u32,
) -> Result<String, AnalysisError> {
    debug!(
        system_len = system.len(),
        user_len = user.len(),
        "Calling model with system prompt"
    );
    let start = Instant::now();
    let result = match api.provider {
        Provider::Anthropic => call_anthropic(api, model, Some(system), user, max_tokens).await,
        Provider::OpenAiCompatible => {
            call_openai_compatible(api, model, Some(system), user, max_tokens).await
        }
    };
    let elapsed = start.elapsed();
    match &result {
        Ok(response) => info!(
            response_len = response.len(),
            elapsed_ms = elapsed.as_millis(),
            "Model call succeeded"
        ),
        Err(e) => warn!(error = %e, elapsed_ms = elapsed.as_millis(), "Model call failed"),
    }
    result
}

/// Call Anthropic API.
async fn call_anthropic(
    api: &ApiConfig,
    model: &str,
    system: Option<&str>,
    user: &str,
    max_tokens: u32,
) -> Result<String, AnalysisError> {
    #[derive(Serialize)]
    struct Message<'a> {
        role: &'static str,
        content: &'a str,
    }

    #[derive(Serialize)]
    struct Request<'a> {
        model: &'a str,
        max_tokens: u32,
        #[serde(skip_serializing_if = "Option::is_none")]
        system: Option<&'a str>,
        messages: Vec<Message<'a>>,
    }

    #[derive(Deserialize)]
    struct ContentBlock {
        text: String,
    }

    #[derive(Deserialize)]
    struct Response {
        content: Vec<ContentBlock>,
    }

    let api_key = api.api_key()?;

    let request = Request {
        model,
        max_tokens,
        system,
        messages: vec![Message {
            role: "user",
            content: user,
        }],
    };

    let client = reqwest::Client::builder()
        .timeout(API_TIMEOUT)
        .build()
        .map_err(|e| AnalysisError::Request(e.to_string()))?;
    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", &api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
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
        .content
        .into_iter()
        .next()
        .map(|block| block.text)
        .ok_or(AnalysisError::EmptyResponse)
}

/// Call OpenAI-compatible API (OpenRouter, OpenAI, Ollama, etc.)
async fn call_openai_compatible(
    api: &ApiConfig,
    model: &str,
    system: Option<&str>,
    user: &str,
    max_tokens: u32,
) -> Result<String, AnalysisError> {
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
