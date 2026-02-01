//! AI configuration module.
//!
//! Loads configuration from TOML file specified by `AI_CONFIG_PATH` env var
//! or defaults to `./ai_config.toml`.

use std::path::PathBuf;

use once_cell::sync::OnceCell;
use serde::Deserialize;
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
    /// Default API configuration (used by analysis, inherited by others)
    pub api: ApiConfig,

    /// Optional safeguard configuration for prompt injection protection
    #[serde(default)]
    pub safeguard: Option<SafeguardConfig>,

    /// Main analysis configuration
    pub analysis: AnalysisConfig,

    /// Optional translation configuration for two-step pipeline
    #[serde(default)]
    pub translation: Option<TranslationConfig>,
}

impl AiConfig {
    /// Validate the configuration.
    fn validate(&self) -> Result<(), ConfigError> {
        // Validate main API
        self.api.validate("api")?;

        // Validate safeguard API if present
        if let Some(ref safeguard) = self.safeguard
            && safeguard.enabled
                && let Some(ref api) = safeguard.api {
                    api.validate("safeguard.api")?;
                }

        // Validate translation API if present
        if let Some(ref translation) = self.translation
            && translation.enabled
                && let Some(ref api) = translation.api {
                    api.validate("translation.api")?;
                }

        Ok(())
    }
}

/// API configuration for a provider.
#[derive(Debug, Clone, Deserialize)]
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
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
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

    /// Optional API override (if None, uses main [api])
    pub api: Option<ApiConfig>,
}

/// Main analysis configuration.
#[derive(Debug, Deserialize)]
pub struct AnalysisConfig {
    /// Model to use for personality analysis
    pub model: String,

    /// Maximum tokens for analysis response
    #[serde(default = "default_analysis_max_tokens")]
    pub max_tokens: u32,
}

/// Translation configuration for two-step pipeline.
#[derive(Debug, Deserialize)]
pub struct TranslationConfig {
    /// Whether translation is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Source language for analysis (model's "native" language)
    pub source_language: SourceLanguage,

    /// Model to use for translation
    pub model: String,

    /// Maximum tokens for translation response
    #[serde(default = "default_translation_max_tokens")]
    pub max_tokens: u32,

    /// Optional API override (if None, uses main [api])
    pub api: Option<ApiConfig>,
}

/// Source language for analysis prompt.
#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
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
