//! API provider implementations.
//!
//! Uses llm-relay for all LLM calls.

use std::time::{Duration, Instant};

use llm_relay::{ChatOptions, ClientConfig, LlmClient};
use tracing::{debug, info, instrument, warn};

use crate::config::{ApiConfig, Provider, ThinkingConfig};

use super::error::AnalysisError;

/// Default timeout for API calls (3 minutes to allow for slow Claude Opus responses).
pub const API_TIMEOUT: Duration = Duration::from_secs(180);

/// Call an AI model with the given prompt.
#[instrument(skip(api, prompt), fields(model = %model, max_tokens = %max_tokens, provider = ?api.provider))]
pub async fn call_model(
    api: &ApiConfig,
    model: &str,
    prompt: &str,
    max_tokens: u32,
    thinking: Option<&ThinkingConfig>,
) -> Result<String, AnalysisError> {
    debug!(prompt_len = prompt.len(), "Calling model");
    let start = Instant::now();
    let result = do_call(api, model, None, prompt, max_tokens, thinking).await;
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
    thinking: Option<&ThinkingConfig>,
) -> Result<String, AnalysisError> {
    debug!(
        system_len = system.len(),
        user_len = user.len(),
        "Calling model with system prompt"
    );
    let start = Instant::now();
    let result = do_call(api, model, Some(system), user, max_tokens, thinking).await;
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

async fn do_call(
    api: &ApiConfig,
    model: &str,
    system: Option<&str>,
    user: &str,
    max_tokens: u32,
    thinking: Option<&ThinkingConfig>,
) -> Result<String, AnalysisError> {
    let api_key = api.api_key()?;
    let config = match api.provider {
        Provider::Anthropic => ClientConfig::anthropic(&api_key, model),
        Provider::OpenAiCompatible => {
            let api_url = api.api_url.as_ref().ok_or_else(|| {
                AnalysisError::Config(crate::config::ConfigError::Validation(
                    "api_url is required for OpenAI-compatible provider".to_string(),
                ))
            })?;
            ClientConfig::openai_compatible(api_url, &api_key, model)
        }
    };
    let config = config.max_tokens(max_tokens).timeout(API_TIMEOUT);
    let client = LlmClient::new(config).map_err(|e| AnalysisError::Request(e.to_string()))?;
    let options = ChatOptions {
        system,
        thinking,
        ..Default::default()
    };
    let resp = client.complete(user, options).await.map_err(|e| match e {
        llm_relay::LlmError::ApiError { status, body } => AnalysisError::ApiError { status, body },
        llm_relay::LlmError::EmptyResponse => AnalysisError::EmptyResponse,
        llm_relay::LlmError::ParseResponse(msg) => AnalysisError::ParseResponse(msg),
        other => AnalysisError::Request(other.to_string()),
    })?;
    Ok(resp.text())
}
