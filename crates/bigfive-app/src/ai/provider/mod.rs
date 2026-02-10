//! API provider implementations.
//!
//! Dispatches to provider-specific modules based on [`Provider`] type.

mod anthropic;
mod openai;

use std::time::Instant;

use tracing::{debug, info, instrument, warn};

use crate::config::{ApiConfig, Provider, ThinkingConfig};

use super::error::AnalysisError;

/// Default timeout for API calls (3 minutes to allow for slow Claude Opus responses)
pub use anthropic::API_TIMEOUT;

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
    let result = match api.provider {
        Provider::Anthropic => {
            anthropic::call(api, model, None, prompt, max_tokens, thinking).await
        }
        Provider::OpenAiCompatible => openai::call(api, model, None, prompt, max_tokens).await,
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
    thinking: Option<&ThinkingConfig>,
) -> Result<String, AnalysisError> {
    debug!(
        system_len = system.len(),
        user_len = user.len(),
        "Calling model with system prompt"
    );
    let start = Instant::now();
    let result = match api.provider {
        Provider::Anthropic => {
            anthropic::call(api, model, Some(system), user, max_tokens, thinking).await
        }
        Provider::OpenAiCompatible => {
            openai::call(api, model, Some(system), user, max_tokens).await
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
