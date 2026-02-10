//! AI configuration module.
//!
//! Loads configuration from TOML file specified by `AI_CONFIG_PATH` env var
//! or defaults to `./ai_config.toml`.

use std::path::PathBuf;

use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Global config instance (loaded once on first access)
static CONFIG: OnceCell<AiConfig> = OnceCell::new();

/// Get the global AI configuration, loading it on first access.
pub fn get_config() -> Result<&'static AiConfig, ConfigError> {
    CONFIG.get_or_try_init(load_config)
}

/// Load configuration from file.
fn load_config() -> Result<AiConfig, ConfigError> {
    let path = std::env::var("AI_CONFIG_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("ai_config.toml"));

    let content = std::fs::read_to_string(&path).map_err(|e| ConfigError::ReadFile {
        path: path.clone(),
        source: e,
    })?;

    let config: AiConfig =
        toml::from_str(&content).map_err(|e| ConfigError::Parse { path, source: e })?;

    // Validate configuration
    config.validate()?;

    Ok(config)
}

/// Configuration loading errors.
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Failed to read config file '{path}': {source}")]
    ReadFile {
        path: PathBuf,
        source: std::io::Error,
    },

    #[error("Failed to parse config file '{path}': {source}")]
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },

    #[error("Invalid configuration: {0}")]
    Validation(String),
}

/// Root AI configuration structure.
#[derive(Debug, Deserialize)]
pub struct AiConfig {
    /// Optional safeguard configuration for prompt injection protection (shared for all models)
    #[serde(default)]
    pub safeguard: Option<SafeguardConfig>,

    /// Available model presets
    pub models: Vec<ModelPreset>,
}

impl AiConfig {
    /// Validate the configuration.
    fn validate(&self) -> Result<(), ConfigError> {
        // Must have at least one model
        if self.models.is_empty() {
            return Err(ConfigError::Validation(
                "At least one model preset is required".to_string(),
            ));
        }

        // Validate safeguard API if present
        if let Some(ref safeguard) = self.safeguard
            && safeguard.enabled
        {
            safeguard.api.validate("safeguard.api")?;
        }

        // Validate each model preset
        for (i, preset) in self.models.iter().enumerate() {
            preset.validate(&format!("models[{}]", i))?;
        }

        // Check that exactly one model is marked as default (or none, then first is default)
        let default_count = self.models.iter().filter(|m| m.default).count();
        if default_count > 1 {
            return Err(ConfigError::Validation(
                "Only one model can be marked as default".to_string(),
            ));
        }

        Ok(())
    }

    /// Get the default model preset.
    pub fn default_model(&self) -> &ModelPreset {
        self.models
            .iter()
            .find(|m| m.default)
            .unwrap_or_else(|| self.models.first().unwrap())
    }

    /// Get a model preset by ID.
    pub fn get_model(&self, id: &str) -> Option<&ModelPreset> {
        self.models.iter().find(|m| m.id == id)
    }
}

/// API configuration for a provider.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ApiConfig {
    /// API provider type
    pub provider: Provider,

    /// Name of environment variable containing the API key
    pub api_key_env: String,

    /// API endpoint URL (required for OpenAI-compatible providers)
    pub api_url: Option<String>,
}

impl ApiConfig {
    /// Validate the API configuration.
    fn validate(&self, section: &str) -> Result<(), ConfigError> {
        if self.provider == Provider::OpenAiCompatible && self.api_url.is_none() {
            return Err(ConfigError::Validation(format!(
                "[{section}] api_url is required for 'openai' provider"
            )));
        }
        Ok(())
    }

    /// Get the API key from environment.
    pub fn api_key(&self) -> Result<String, ConfigError> {
        std::env::var(&self.api_key_env).map_err(|_| {
            ConfigError::Validation(format!(
                "Environment variable '{}' not set",
                self.api_key_env
            ))
        })
    }
}

/// API provider type.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// Anthropic API (Claude models)
    Anthropic,

    /// OpenAI-compatible API (OpenRouter, OpenAI, Ollama, etc.)
    #[serde(alias = "openai")]
    OpenAiCompatible,
}

/// Safeguard configuration for prompt injection protection.
#[derive(Debug, Deserialize)]
pub struct SafeguardConfig {
    /// Whether safeguard is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Model to use for safeguard checks
    pub model: String,

    /// Maximum tokens for safeguard response
    #[serde(default = "default_safeguard_max_tokens")]
    pub max_tokens: u32,

    /// API configuration for safeguard
    pub api: ApiConfig,
}

/// Model preset configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ModelPreset {
    /// Unique identifier for this preset
    pub id: String,

    /// Display name shown in UI
    pub display_name: String,

    /// Model identifier (e.g., "claude-opus-4-6")
    pub model: String,

    /// Source language for analysis (model's "native" language)
    pub source_lang: SourceLanguage,

    /// Maximum tokens for analysis response
    #[serde(default = "default_analysis_max_tokens")]
    pub max_tokens: u32,

    /// Whether this is the default model
    #[serde(default)]
    pub default: bool,

    /// API configuration for this model
    pub api: ApiConfig,

    /// Optional thinking configuration (Anthropic models only)
    #[serde(default)]
    pub thinking: Option<ThinkingConfig>,

    /// Optional translation configuration
    pub translation: Option<TranslationConfig>,
}

impl ModelPreset {
    /// Validate the model preset.
    fn validate(&self, section: &str) -> Result<(), ConfigError> {
        self.api.validate(&format!("{}.api", section))?;

        if let Some(ref translation) = self.translation {
            translation
                .api
                .validate(&format!("{}.translation.api", section))?;
        }

        Ok(())
    }
}

/// Thinking mode configuration for Anthropic models.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum ThinkingConfig {
    /// Adaptive thinking — Claude decides when and how much to think (Opus 4.6+).
    /// No budget_tokens needed; use `effort` to guide thinking depth.
    Adaptive {
        /// Effort level for adaptive thinking. Default: high.
        #[serde(default)]
        effort: EffortLevel,
    },
    /// Manual thinking with explicit token budget (Sonnet 4.5, Sonnet 4, Haiku 4.5, etc.).
    Enabled {
        /// Token budget for thinking.
        budget_tokens: u32,
    },
}

/// Effort level for adaptive thinking.
///
/// Controls how much thinking Claude does in adaptive mode:
/// - `max`: always thinks (Opus 4.6 only)
/// - `high`: almost always thinks (default)
/// - `medium`: moderate thinking, may skip for simple queries
/// - `low`: minimizes thinking, skips for simple tasks
#[derive(Debug, Clone, Copy, Deserialize, Serialize, Default, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum EffortLevel {
    Max,
    #[default]
    High,
    Medium,
    Low,
}

impl EffortLevel {
    /// Get the effort level as a string for the API.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Max => "max",
            Self::High => "high",
            Self::Medium => "medium",
            Self::Low => "low",
        }
    }
}

/// Translation configuration for two-step pipeline.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TranslationConfig {
    /// Model to use for translation
    pub model: String,

    /// Maximum tokens for translation response
    #[serde(default = "default_translation_max_tokens")]
    pub max_tokens: u32,

    /// API configuration for translation
    pub api: ApiConfig,
}

/// Source language for analysis prompt.
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SourceLanguage {
    /// English
    En,
    /// Russian
    Ru,
    /// Chinese
    Zh,
}

impl SourceLanguage {
    /// Get the language code.
    pub fn code(&self) -> &'static str {
        match self {
            Self::En => "en",
            Self::Ru => "ru",
            Self::Zh => "zh",
        }
    }

    /// Get language name for prompts.
    pub fn name(&self) -> &'static str {
        match self {
            Self::En => "English",
            Self::Ru => "Russian",
            Self::Zh => "Chinese",
        }
    }
}

/// Model info for client (subset of ModelPreset)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub display_name: String,
    pub default: bool,
}

impl From<&ModelPreset> for ModelInfo {
    fn from(preset: &ModelPreset) -> Self {
        Self {
            id: preset.id.clone(),
            display_name: preset.display_name.clone(),
            default: preset.default,
        }
    }
}

// Default values
fn default_safeguard_max_tokens() -> u32 {
    1024
}

fn default_analysis_max_tokens() -> u32 {
    8192
}

fn default_translation_max_tokens() -> u32 {
    8192
}
