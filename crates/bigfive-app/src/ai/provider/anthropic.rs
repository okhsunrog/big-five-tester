//! Anthropic API provider.
//!
//! Supports three thinking modes:
//! - **Adaptive**: `thinking: {type: "adaptive"}` with optional `output_config: {effort}` (Opus 4.6+)
//! - **Manual**: `thinking: {type: "enabled", budget_tokens: N}` (Sonnet 4.5, Sonnet 4, Haiku 4.5)
//! - **Disabled**: no `thinking` field (all models)

use std::time::Duration;

use serde::{Deserialize, Serialize};
use tracing::debug;

use crate::ai::error::AnalysisError;
use crate::config::{ApiConfig, EffortLevel, ThinkingConfig};

/// Default timeout for API calls (3 minutes to allow for slow Claude Opus responses).
pub const API_TIMEOUT: Duration = Duration::from_secs(180);

// ── Request types ───────────────────────────────────────────────────────

#[derive(Serialize)]
struct Message<'a> {
    role: &'static str,
    content: &'a str,
}

/// Thinking parameter for the Anthropic API.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ThinkingParam {
    /// Adaptive thinking — Claude decides when and how much to think.
    Adaptive,
    /// Manual thinking with explicit budget.
    Enabled { budget_tokens: u32 },
}

/// Output configuration (used with adaptive thinking to set effort level).
#[derive(Serialize)]
struct OutputConfig<'a> {
    effort: &'a str,
}

#[derive(Serialize)]
struct Request<'a> {
    model: &'a str,
    max_tokens: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    system: Option<&'a str>,
    messages: Vec<Message<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    thinking: Option<ThinkingParam>,
    #[serde(skip_serializing_if = "Option::is_none")]
    output_config: Option<OutputConfig<'a>>,
}

// ── Response types ──────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ContentBlock {
    #[serde(rename = "type")]
    block_type: String,
    /// Present on `text` blocks.
    #[serde(default)]
    text: Option<String>,
    // `thinking` and `signature` fields on thinking blocks are ignored.
}

#[derive(Deserialize)]
struct Response {
    content: Vec<ContentBlock>,
}

// ── Public API ──────────────────────────────────────────────────────────

/// Call the Anthropic Messages API.
pub async fn call(
    api: &ApiConfig,
    model: &str,
    system: Option<&str>,
    user: &str,
    max_tokens: u32,
    thinking: Option<&ThinkingConfig>,
) -> Result<String, AnalysisError> {
    let api_key = api.api_key()?;

    // Build thinking and output_config params from config.
    let (thinking_param, output_config) = build_thinking_params(thinking);

    debug!(
        ?thinking_param,
        has_output_config = output_config.is_some(),
        "Building Anthropic request"
    );

    let request = Request {
        model,
        max_tokens,
        system,
        messages: vec![Message {
            role: "user",
            content: user,
        }],
        thinking: thinking_param,
        output_config,
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

    extract_text(api_response.content)
}

// ── Helpers ─────────────────────────────────────────────────────────────

/// Build the `thinking` and `output_config` parameters from [`ThinkingConfig`].
fn build_thinking_params(
    config: Option<&ThinkingConfig>,
) -> (Option<ThinkingParam>, Option<OutputConfig<'static>>) {
    match config {
        Some(ThinkingConfig::Adaptive { effort }) => {
            let output_config = if *effort == EffortLevel::High {
                // High is the default — no need to send it explicitly.
                None
            } else {
                Some(OutputConfig {
                    effort: effort.as_str(),
                })
            };
            (Some(ThinkingParam::Adaptive), output_config)
        }
        Some(ThinkingConfig::Enabled { budget_tokens }) => (
            Some(ThinkingParam::Enabled {
                budget_tokens: *budget_tokens,
            }),
            None,
        ),
        None => (None, None),
    }
}

/// Extract text from content blocks.
///
/// When thinking is enabled, the response contains both `thinking` and `text` blocks.
/// We filter for `text` blocks and concatenate them.
fn extract_text(content: Vec<ContentBlock>) -> Result<String, AnalysisError> {
    let texts: Vec<String> = content
        .into_iter()
        .filter(|b| b.block_type == "text")
        .filter_map(|b| b.text)
        .collect();

    if texts.is_empty() {
        return Err(AnalysisError::EmptyResponse);
    }

    Ok(texts.join(""))
}
