//! AI analysis pipeline.
//!
//! Supports single-step (direct analysis) and two-step (analyze + translate) pipelines.

use bigfive::PersonalityProfile;
use tracing::{debug, info, instrument, warn};

use crate::config::{AiConfig, SourceLanguage, get_config};

use super::error::AnalysisError;
use super::prompts;
use super::provider::{call_model, call_model_with_system};

/// Generate personality analysis.
///
/// # Arguments
/// * `profile` - The personality profile to analyze
/// * `user_context` - Optional user-provided context (name, age, profession, etc.)
/// * `interface_language` - The user's interface language ("en" or "ru")
///
/// # Pipeline
/// 1. If safeguard is enabled, check user_context for prompt injection
/// 2. Generate analysis (in source_language if translation enabled, otherwise in interface_language)
/// 3. If translation enabled and source != target, translate to interface_language
#[instrument(skip_all, fields(lang = %interface_language, has_context = user_context.is_some()))]
pub async fn generate_analysis(
    profile: &PersonalityProfile,
    user_context: Option<&str>,
    interface_language: &str,
) -> Result<String, AnalysisError> {
    info!("Starting personality analysis pipeline");
    let config = get_config()?;

    // Step 0: Safeguard check (if enabled and context provided)
    if let Some(context) = user_context
        && !context.trim().is_empty()
    {
        debug!("Running safeguard check on user context");
        check_safeguard(config, context).await?;
        info!("Safeguard check passed");
    }

    // Determine if we use translation pipeline
    let use_translation = config
        .translation
        .as_ref()
        .map(|t| t.enabled)
        .unwrap_or(false);

    if use_translation {
        // Two-step pipeline: analyze in source_language, then translate
        info!("Using translation pipeline");
        generate_with_translation(config, profile, user_context, interface_language).await
    } else {
        // Single-step pipeline: analyze directly in interface_language
        info!("Using direct pipeline (no translation)");
        generate_direct(config, profile, user_context, interface_language).await
    }
}

/// Check user context for prompt injection using safeguard model.
#[instrument(skip_all)]
async fn check_safeguard(config: &AiConfig, user_context: &str) -> Result<(), AnalysisError> {
    let safeguard = match &config.safeguard {
        Some(s) if s.enabled => s,
        _ => {
            debug!("Safeguard not configured or disabled, skipping");
            return Ok(());
        }
    };

    info!(model = %safeguard.model, "Running safeguard check");

    // Use safeguard's API override or fall back to main API
    let api = safeguard.api.as_ref().unwrap_or(&config.api);

    let system = prompts::safeguard_system_prompt();
    let response = call_model_with_system(
        api,
        &safeguard.model,
        system,
        user_context,
        safeguard.max_tokens,
    )
    .await?;

    // Parse response - looking for "SAFE" or "UNSAFE"
    let response_upper = response.trim().to_uppercase();

    if response_upper.contains("UNSAFE") || !response_upper.contains("SAFE") {
        warn!(response = %response.chars().take(200).collect::<String>(), "Safeguard detected unsafe input");
        return Err(AnalysisError::UnsafeInput);
    }

    debug!("Input classified as safe");
    Ok(())
}

/// Generate analysis directly in the interface language.
#[instrument(skip_all)]
async fn generate_direct(
    config: &AiConfig,
    profile: &PersonalityProfile,
    user_context: Option<&str>,
    interface_language: &str,
) -> Result<String, AnalysisError> {
    // Determine prompt language from interface language
    let prompt_lang = match interface_language {
        "ru" => SourceLanguage::Ru,
        "zh" => SourceLanguage::Zh,
        _ => SourceLanguage::En,
    };

    info!(
        model = %config.analysis.model,
        prompt_lang = ?prompt_lang,
        "Generating direct analysis"
    );

    let prompt = prompts::analysis_prompt(prompt_lang, profile, user_context);

    let result = call_model(
        &config.api,
        &config.analysis.model,
        &prompt,
        config.analysis.max_tokens,
    )
    .await?;

    info!(result_len = result.len(), "Direct analysis complete");
    Ok(result)
}

/// Generate analysis with translation pipeline.
#[instrument(skip_all)]
async fn generate_with_translation(
    config: &AiConfig,
    profile: &PersonalityProfile,
    user_context: Option<&str>,
    interface_language: &str,
) -> Result<String, AnalysisError> {
    let translation = config.translation.as_ref().unwrap(); // Safe: we checked enabled above

    // Step 1: Generate analysis in source language
    info!(
        model = %config.analysis.model,
        source_lang = ?translation.source_language,
        "Generating analysis in source language"
    );

    let prompt = prompts::analysis_prompt(translation.source_language, profile, user_context);

    let analysis = call_model(
        &config.api,
        &config.analysis.model,
        &prompt,
        config.analysis.max_tokens,
    )
    .await?;

    info!(analysis_len = analysis.len(), "Analysis generated");

    // Step 2: Translate if source != target
    if translation.source_language.code() == interface_language {
        info!("Source matches interface language, skipping translation");
        return Ok(analysis);
    }

    // Use translation's API override or fall back to main API
    let api = translation.api.as_ref().unwrap_or(&config.api);

    info!(
        model = %translation.model,
        from = ?translation.source_language,
        to = %interface_language,
        "Translating analysis"
    );

    let translation_prompt =
        prompts::translation_prompt(&analysis, translation.source_language, interface_language);

    let translated = call_model(
        api,
        &translation.model,
        &translation_prompt,
        translation.max_tokens,
    )
    .await?;

    info!(translated_len = translated.len(), "Translation complete");
    Ok(translated)
}
