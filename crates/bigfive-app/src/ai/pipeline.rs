//! AI analysis pipeline.
//!
//! Supports single-step (direct analysis) and two-step (analyze + translate) pipelines.

use bigfive::PersonalityProfile;
use tracing::{debug, info, instrument, warn};

use crate::config::{AiConfig, ModelPreset, get_config};

use super::error::AnalysisError;
use super::prompts;
use super::provider::{call_model, call_model_with_system};

/// Generate personality analysis using a specific model preset.
///
/// # Arguments
/// * `model_id` - ID of the model preset to use
/// * `profile` - The personality profile to analyze
/// * `user_context` - Optional user-provided context (name, age, profession, etc.)
/// * `interface_language` - The user's interface language ("en", "ru", or "zh")
///
/// # Pipeline
/// 1. If safeguard is enabled, check user_context for prompt injection
/// 2. Generate analysis in model's source_lang
/// 3. If source_lang != interface_language, translate to interface_language
#[instrument(skip_all, fields(model_id = %model_id, lang = %interface_language, has_context = user_context.is_some()))]
pub async fn generate_analysis(
    model_id: &str,
    profile: &PersonalityProfile,
    user_context: Option<&str>,
    interface_language: &str,
) -> Result<String, AnalysisError> {
    info!("Starting personality analysis pipeline");
    let config = get_config()?;

    // Find the model preset
    let preset = config
        .get_model(model_id)
        .ok_or_else(|| AnalysisError::InvalidModel(model_id.to_string()))?;

    info!(
        model = %preset.model,
        source_lang = ?preset.source_lang,
        "Using model preset"
    );

    // Step 0: Safeguard check (if enabled and context provided)
    if let Some(context) = user_context
        && !context.trim().is_empty()
    {
        debug!("Running safeguard check on user context");
        check_safeguard(config, context).await?;
        info!("Safeguard check passed");
    }

    // Generate analysis with the preset
    generate_with_preset(preset, profile, user_context, interface_language).await
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

    let system = prompts::safeguard_system_prompt();
    let response = call_model_with_system(
        &safeguard.api,
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

/// Generate analysis using a model preset.
#[instrument(skip_all, fields(model = %preset.model, source_lang = ?preset.source_lang))]
async fn generate_with_preset(
    preset: &ModelPreset,
    profile: &PersonalityProfile,
    user_context: Option<&str>,
    interface_language: &str,
) -> Result<String, AnalysisError> {
    // Step 1: Generate analysis in source language
    info!(
        model = %preset.model,
        source_lang = ?preset.source_lang,
        "Generating analysis in source language"
    );

    let prompt = prompts::analysis_prompt(preset.source_lang, profile, user_context);

    let analysis = call_model(&preset.api, &preset.model, &prompt, preset.max_tokens).await?;

    info!(analysis_len = analysis.len(), "Analysis generated");

    // Step 2: Translate if source != target
    if preset.source_lang.code() == interface_language {
        info!("Source matches interface language, skipping translation");
        return Ok(analysis);
    }

    // Check if translation is configured
    let translation = match &preset.translation {
        Some(t) => t,
        None => {
            info!("No translation configured, returning analysis in source language");
            return Ok(analysis);
        }
    };

    info!(
        model = %translation.model,
        from = ?preset.source_lang,
        to = %interface_language,
        "Translating analysis"
    );

    let translation_prompt =
        prompts::translation_prompt(&analysis, preset.source_lang, interface_language);

    let translated = call_model(
        &translation.api,
        &translation.model,
        &translation_prompt,
        translation.max_tokens,
    )
    .await?;

    info!(translated_len = translated.len(), "Translation complete");
    Ok(translated)
}
