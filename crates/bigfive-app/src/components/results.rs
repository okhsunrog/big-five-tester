//! Results page component with visualization and AI analysis.

use bigfive::{Domain, Facet, PersonalityProfile, ScoreLevel};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_router::components::A;
use leptos_router::hooks::{use_navigate, use_params_map};
use pulldown_cmark::{Options, Parser, html};
use serde::{Deserialize, Serialize};

use crate::components::{LangToggle, ThemeToggle};
use crate::i18n::use_i18n;

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY_PROFILE: &str = "bigfive_profile";
#[cfg(target_arch = "wasm32")]
const STORAGE_KEY_CONTEXT: &str = "bigfive_user_context";

/// Polling interval in milliseconds
#[cfg(target_arch = "wasm32")]
const POLL_INTERVAL_MS: u32 = 3000;

/// Maximum number of poll attempts (3 minutes total with 3s interval)
#[cfg(target_arch = "wasm32")]
const MAX_POLL_ATTEMPTS: u32 = 60;

/// Status of a background analysis job (shared between server and client)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum AnalysisStatus {
    /// Job is queued or processing
    Pending,
    /// Job completed successfully with result
    Complete(String),
    /// Job failed with error message
    Error(String),
}

/// Model info for client (subset of ModelPreset)
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct ClientModelInfo {
    pub id: String,
    pub display_name: String,
    pub default: bool,
}

/// Saved result data (shared between server and client).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SavedResultData {
    pub id: String,
    pub profile: PersonalityProfile,
    pub user_context: Option<String>,
    pub ai_analysis: Option<String>,
    pub lang: String,
}

/// Get available model presets for the client.
#[server]
pub async fn get_available_models() -> Result<Vec<ClientModelInfo>, ServerFnError> {
    use crate::config::get_config;

    let config = get_config().map_err(|e| ServerFnError::new(e.to_string()))?;
    Ok(config
        .models
        .iter()
        .map(|m| ClientModelInfo {
            id: m.id.clone(),
            display_name: m.display_name.clone(),
            default: m.default,
        })
        .collect())
}

/// Convert markdown text to HTML
fn markdown_to_html(markdown: &str) -> String {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = Parser::new_ext(markdown, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    html_output
}

/// Save results to database, returns UUID.
#[server]
pub async fn save_results(
    profile: PersonalityProfile,
    user_context: Option<String>,
    lang: String,
) -> Result<String, ServerFnError> {
    use crate::db;

    let id = uuid::Uuid::new_v4().to_string();
    db::save_result(&id, &profile, user_context.as_deref(), &lang)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    tracing::info!(result_id = %id, "Saved test results to database");
    Ok(id)
}

/// Get saved results from database.
#[server]
pub async fn get_saved_results(id: String) -> Result<Option<SavedResultData>, ServerFnError> {
    use crate::db;

    let result = db::get_result(&id)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    Ok(result.map(|r| SavedResultData {
        id: r.id,
        profile: r.profile,
        user_context: r.user_context,
        ai_analysis: r.ai_analysis,
        lang: r.lang,
    }))
}

/// Update AI analysis in database.
#[server]
pub async fn update_saved_analysis(id: String, analysis: String) -> Result<(), ServerFnError> {
    use crate::db;

    db::update_ai_analysis(&id, &analysis)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    tracing::info!(result_id = %id, "Updated AI analysis in database");
    Ok(())
}

/// Start an analysis job and return immediately with a job ID.
/// The analysis runs in the background.
#[server]
pub async fn start_analysis(
    profile: PersonalityProfile,
    lang: String,
    user_context: Option<String>,
    model_id: String,
    result_id: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::jobs::{self, JobStatus};

    // Load .env file for local development
    dotenvy::dotenv().ok();

    // Generate job ID and create job entry
    let job_id = jobs::generate_job_id();
    jobs::create_job(&job_id);

    tracing::info!(
        job_id = %job_id,
        lang = %lang,
        model_id = %model_id,
        has_context = user_context.is_some(),
        "Starting background analysis job"
    );

    // Clone job_id for the spawned task
    let job_id_clone = job_id.clone();

    // Spawn background task
    tokio::spawn(async move {
        use crate::ai;

        let start = std::time::Instant::now();
        jobs::update_job_status(&job_id_clone, JobStatus::Processing);

        match ai::generate_analysis(&model_id, &profile, user_context.as_deref(), &lang).await {
            Ok(description) => {
                tracing::info!(
                    job_id = %job_id_clone,
                    elapsed_ms = start.elapsed().as_millis(),
                    response_len = description.len(),
                    "Background analysis completed"
                );

                if let Some(rid) = &result_id {
                    tracing::info!("Results link: /{}/results/{}", lang, rid);
                }

                jobs::update_job_status(&job_id_clone, JobStatus::Complete(description));
            }
            Err(e) => {
                tracing::error!(
                    job_id = %job_id_clone,
                    error = %e,
                    elapsed_ms = start.elapsed().as_millis(),
                    "Background analysis failed"
                );
                jobs::update_job_status(&job_id_clone, JobStatus::Error(e.to_string()));
            }
        }
    });

    Ok(job_id)
}

/// Get the status of an analysis job.
#[server]
pub async fn get_analysis_status(job_id: String) -> Result<AnalysisStatus, ServerFnError> {
    use crate::jobs::{self, JobStatus};

    match jobs::get_job_status(&job_id) {
        Some(JobStatus::Pending) | Some(JobStatus::Processing) => Ok(AnalysisStatus::Pending),
        Some(JobStatus::Complete(result)) => {
            // Clean up job after returning result
            jobs::remove_job(&job_id);
            Ok(AnalysisStatus::Complete(result))
        }
        Some(JobStatus::Error(err)) => {
            // Clean up job after returning error
            jobs::remove_job(&job_id);
            Ok(AnalysisStatus::Error(err))
        }
        None => Ok(AnalysisStatus::Error("Job not found".to_string())),
    }
}

/// Results page with score visualization and AI-generated description.
#[component]
pub fn ResultsPage() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();
    let params = use_params_map();

    // Profile state - starts as None, loaded via Effect to avoid hydration mismatch
    let (profile, set_profile) = signal::<Option<PersonalityProfile>>(None);

    // Saved result ID (UUID) — set after saving to DB or loading from URL
    let (result_id, set_result_id) = signal::<Option<String>>(None);

    // "Copied!" feedback state
    #[allow(unused_variables)]
    let (link_copied, set_link_copied) = signal(false);

    // Expanded domain state (for facet accordion)
    let (expanded_domain, set_expanded_domain) = signal::<Option<Domain>>(None);

    // AI description state
    let (ai_description, set_ai_description) = signal::<Option<String>>(None);
    let (ai_loading, set_ai_loading) = signal(false);
    let (ai_error, set_ai_error) = signal::<Option<String>>(None);

    // User context for AI (optional self-description)
    let (user_context, set_user_context) = signal(String::new());

    // "Not found" state for invalid shared links
    let (not_found, set_not_found) = signal(false);

    // Load available models from server (Resource runs on both server and client)
    let models_resource =
        Resource::new(|| (), |_| async move { get_available_models().await.ok() });

    // Selected model state
    let (selected_model, set_selected_model) = signal::<Option<String>>(None);

    // Set default model when models load
    Effect::new(move |_| {
        if let Some(Some(models)) = models_resource.get()
            && selected_model.get().is_none()
        {
            let default_id = models
                .iter()
                .find(|m| m.default)
                .or_else(|| models.first())
                .map(|m| m.id.clone());
            if let Some(id) = default_id {
                set_selected_model.set(Some(id));
            }
        }
    });

    // Load profile: from DB (if :id param) or from localStorage (then auto-save to DB)
    Effect::new(move |_| {
        let url_id = params.get().get("id");

        if let Some(id) = url_id {
            // Load from database
            let nav = navigate.clone();
            let prefix = i18n.get_locale().path_prefix().to_string();
            spawn_local(async move {
                match get_saved_results(id).await {
                    Ok(Some(saved)) => {
                        set_result_id.set(Some(saved.id));
                        set_profile.set(Some(saved.profile));
                        if let Some(ctx) = saved.user_context {
                            set_user_context.set(ctx);
                        }
                        if let Some(analysis) = saved.ai_analysis {
                            set_ai_description.set(Some(analysis));
                        }
                    }
                    Ok(None) => {
                        set_not_found.set(true);
                    }
                    Err(_) => {
                        nav(&format!("{}/test", prefix), Default::default());
                    }
                }
            });
        } else {
            // Load from localStorage
            let loaded = load_profile();
            if loaded.is_none() {
                let prefix = i18n.get_locale().path_prefix();
                navigate(&format!("{}/test", prefix), Default::default());
                return;
            }
            let prof = loaded.unwrap();
            set_profile.set(Some(prof.clone()));

            if let Some(ctx) = load_context() {
                set_user_context.set(ctx.clone());
            }

            // Auto-save to DB and navigate to shareable URL
            let nav = navigate.clone();
            let locale = i18n.get_locale();
            let ctx = load_context();
            spawn_local(async move {
                match save_results(prof, ctx, locale.code().to_string()).await {
                    Ok(id) => {
                        set_result_id.set(Some(id.clone()));
                        // Replace URL with shareable link
                        #[cfg(target_arch = "wasm32")]
                        {
                            if let Some(window) = web_sys::window() {
                                let new_url = format!("{}/results/{}", locale.path_prefix(), id);
                                let _ = window.history().and_then(|h| {
                                    h.replace_state_with_url(
                                        &wasm_bindgen::JsValue::NULL,
                                        "",
                                        Some(&new_url),
                                    )
                                });
                            }
                        }
                        let _ = nav;
                    }
                    Err(_e) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&format!("Failed to save results: {}", _e).into());
                    }
                }
            });
        }
    });

    // Request AI description with polling
    let request_ai = move |_| {
        let Some(prof) = profile.get() else { return };
        let Some(model_id) = selected_model.get() else {
            set_ai_error.set(Some("No model selected".to_string()));
            return;
        };
        let locale = i18n.get_locale();
        let context = user_context.get();
        let context_opt = if context.trim().is_empty() {
            None
        } else {
            Some(context)
        };
        let current_result_id = result_id.get();

        set_ai_loading.set(true);
        set_ai_error.set(None);

        spawn_local(async move {
            let lang_str = locale.code();

            // Start the analysis job
            #[cfg(target_arch = "wasm32")]
            web_sys::console::log_1(
                &format!(
                    "Calling start_analysis for lang={}, model={}",
                    lang_str, model_id
                )
                .into(),
            );

            let job_id = match start_analysis(
                prof,
                lang_str.to_string(),
                context_opt,
                model_id,
                current_result_id.clone(),
            )
            .await
            {
                Ok(id) => {
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(&format!("Got job_id: {}", id).into());
                    id
                }
                Err(e) => {
                    #[cfg(target_arch = "wasm32")]
                    web_sys::console::log_1(&format!("start_analysis error: {}", e).into());
                    set_ai_error.set(Some(e.to_string()));
                    set_ai_loading.set(false);
                    return;
                }
            };

            // Poll for results
            #[cfg(target_arch = "wasm32")]
            {
                web_sys::console::log_1(&format!("Starting poll for job {}", job_id).into());
            }

            #[cfg(target_arch = "wasm32")]
            let mut poll_count: u32 = 0;

            loop {
                // Wait before polling
                #[cfg(target_arch = "wasm32")]
                {
                    gloo_timers::future::TimeoutFuture::new(POLL_INTERVAL_MS).await;
                    poll_count += 1;
                }

                // Check for timeout
                #[cfg(target_arch = "wasm32")]
                if poll_count >= MAX_POLL_ATTEMPTS {
                    web_sys::console::log_1(&"Poll timeout reached".into());
                    set_ai_error.set(Some("Analysis timed out. Please try again.".to_string()));
                    set_ai_loading.set(false);
                    break;
                }

                #[cfg(target_arch = "wasm32")]
                web_sys::console::log_1(
                    &format!("Polling attempt {} for job {}", poll_count, job_id).into(),
                );

                match get_analysis_status(job_id.clone()).await {
                    Ok(AnalysisStatus::Complete(description)) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(
                            &format!("Got complete result, len={}", description.len()).into(),
                        );
                        set_ai_description.set(Some(description.clone()));
                        set_ai_loading.set(false);

                        // Persist to DB
                        if let Some(rid) = current_result_id.clone() {
                            let _ = update_saved_analysis(rid, description).await;
                        }
                        break;
                    }
                    Ok(AnalysisStatus::Error(err)) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&format!("Got error: {}", err).into());
                        set_ai_error.set(Some(err));
                        set_ai_loading.set(false);
                        break;
                    }
                    Ok(AnalysisStatus::Pending) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&"Status: pending".into());
                        // Continue polling
                    }
                    Err(e) => {
                        #[cfg(target_arch = "wasm32")]
                        web_sys::console::log_1(&format!("Poll error: {}", e).into());
                        set_ai_error.set(Some(e.to_string()));
                        set_ai_loading.set(false);
                        break;
                    }
                }
            }
        });
    };

    // Copy link to clipboard
    #[allow(unused_variables)]
    let copy_link = move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(id) = result_id.get() {
                if let Some(window) = web_sys::window() {
                    let origin = window.location().origin().unwrap_or_default();
                    let prefix = i18n.get_locale().path_prefix();
                    let url = format!("{}{}/results/{}", origin, prefix, id);

                    let clipboard = window.navigator().clipboard();
                    let _ = clipboard.write_text(&url);

                    set_link_copied.set(true);
                    // Reset after 2 seconds
                    spawn_local(async move {
                        gloo_timers::future::TimeoutFuture::new(2000).await;
                        set_link_copied.set(false);
                    });
                }
            }
        }
    };

    // Toggle domain expansion
    let toggle_domain = move |domain: Domain| {
        set_expanded_domain.update(|current| {
            if *current == Some(domain) {
                *current = None;
            } else {
                *current = Some(domain);
            }
        });
    };

    // Get domain color class
    let domain_color = |domain: &Domain| -> &'static str {
        match domain {
            Domain::Neuroticism => "bg-red-500",
            Domain::Extraversion => "bg-yellow-500",
            Domain::Openness => "bg-purple-500",
            Domain::Agreeableness => "bg-green-500",
            Domain::Conscientiousness => "bg-blue-500",
        }
    };

    // Get level text
    let level_text = move |level: ScoreLevel| -> String {
        match level {
            ScoreLevel::Low => i18n.t("level_low").to_string(),
            ScoreLevel::Neutral => i18n.t("level_neutral").to_string(),
            ScoreLevel::High => i18n.t("level_high").to_string(),
        }
    };

    // Get localized domain name
    let domain_name = move |domain: Domain| -> String {
        match domain {
            Domain::Neuroticism => i18n.t("domain_neuroticism").to_string(),
            Domain::Extraversion => i18n.t("domain_extraversion").to_string(),
            Domain::Openness => i18n.t("domain_openness").to_string(),
            Domain::Agreeableness => i18n.t("domain_agreeableness").to_string(),
            Domain::Conscientiousness => i18n.t("domain_conscientiousness").to_string(),
        }
    };

    // Get localized facet name
    let facet_name = move |facet: Facet| -> String {
        match facet {
            // Neuroticism
            Facet::Anxiety => i18n.t("facet_anxiety").to_string(),
            Facet::Anger => i18n.t("facet_anger").to_string(),
            Facet::Depression => i18n.t("facet_depression").to_string(),
            Facet::SelfConsciousness => i18n.t("facet_self_consciousness").to_string(),
            Facet::Immoderation => i18n.t("facet_immoderation").to_string(),
            Facet::Vulnerability => i18n.t("facet_vulnerability").to_string(),
            // Extraversion
            Facet::Friendliness => i18n.t("facet_friendliness").to_string(),
            Facet::Gregariousness => i18n.t("facet_gregariousness").to_string(),
            Facet::Assertiveness => i18n.t("facet_assertiveness").to_string(),
            Facet::ActivityLevel => i18n.t("facet_activity_level").to_string(),
            Facet::ExcitementSeeking => i18n.t("facet_excitement_seeking").to_string(),
            Facet::Cheerfulness => i18n.t("facet_cheerfulness").to_string(),
            // Openness
            Facet::Imagination => i18n.t("facet_imagination").to_string(),
            Facet::ArtisticInterests => i18n.t("facet_artistic_interests").to_string(),
            Facet::Emotionality => i18n.t("facet_emotionality").to_string(),
            Facet::Adventurousness => i18n.t("facet_adventurousness").to_string(),
            Facet::Intellect => i18n.t("facet_intellect").to_string(),
            Facet::Liberalism => i18n.t("facet_liberalism").to_string(),
            // Agreeableness
            Facet::Trust => i18n.t("facet_trust").to_string(),
            Facet::Morality => i18n.t("facet_morality").to_string(),
            Facet::Altruism => i18n.t("facet_altruism").to_string(),
            Facet::Cooperation => i18n.t("facet_cooperation").to_string(),
            Facet::Modesty => i18n.t("facet_modesty").to_string(),
            Facet::Sympathy => i18n.t("facet_sympathy").to_string(),
            // Conscientiousness
            Facet::SelfEfficacy => i18n.t("facet_self_efficacy").to_string(),
            Facet::Orderliness => i18n.t("facet_orderliness").to_string(),
            Facet::Dutifulness => i18n.t("facet_dutifulness").to_string(),
            Facet::AchievementStriving => i18n.t("facet_achievement_striving").to_string(),
            Facet::SelfDiscipline => i18n.t("facet_self_discipline").to_string(),
            Facet::Cautiousness => i18n.t("facet_cautiousness").to_string(),
        }
    };

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            // Header with language and theme toggles
            <header class="flex justify-between items-center mb-8">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{i18n.t("results_title")}</h1>
                <div class="flex items-center gap-3 no-print">
                    <LangToggle />
                    <ThemeToggle />
                </div>
            </header>

            {move || {
                if not_found.get() {
                    return view! {
                        <div class="text-center py-12">
                            <p class="text-lg text-gray-600 dark:text-gray-300 mb-4">
                                {i18n.t("results_not_found")}
                            </p>
                            <A
                                href=move || i18n.get_locale().path_prefix().to_string()
                                attr:class="px-6 py-2 bg-indigo-600 dark:bg-indigo-500 text-white rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 transition-colors"
                            >
                                {i18n.t("results_home")}
                            </A>
                        </div>
                    }.into_any();
                }

                let Some(prof) = profile.get() else {
                    return view! { <div>"Loading..."</div> }.into_any();
                };

                view! {
                    <div>
                        // Domain scores
                        <div class="space-y-4 mb-8">
                            {prof
                                .domains
                                .iter()
                                .map(|domain_score| {
                                    let domain = domain_score.domain;
                                    let raw = domain_score.raw;
                                    let level = domain_score.level;
                                    let percentage = domain_score.percentage();
                                    let facets = domain_score.facets.clone();
                                    let color = domain_color(&domain);
                                    let is_expanded = move || expanded_domain.get() == Some(domain);

                                    view! {
                                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md overflow-hidden transition-colors duration-300">
                                            // Domain header (clickable)
                                            <button
                                                on:click=move |_| toggle_domain(domain)
                                                class="w-full p-4 flex items-center justify-between hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                                            >
                                                <div class="flex-1">
                                                    <div class="flex items-center justify-between mb-2">
                                                        <h3 class="text-lg font-semibold text-gray-800 dark:text-gray-100">
                                                            {domain_name(domain)}
                                                        </h3>
                                                        <span class="text-sm text-gray-500 dark:text-gray-400">
                                                            {format!("{} ({})", raw, level_text(level))}
                                                        </span>
                                                    </div>
                                                    // Score bar
                                                    <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-3">
                                                        <div
                                                            class=format!(
                                                                "{} h-3 rounded-full transition-all duration-500",
                                                                color,
                                                            )
                                                            style:width=format!("{}%", percentage)
                                                        />
                                                    </div>
                                                </div>
                                                // Expand icon
                                                <svg
                                                    class=move || {
                                                        format!(
                                                            "no-print w-5 h-5 ml-4 text-gray-400 dark:text-gray-500 transition-transform duration-200 {}",
                                                            if is_expanded() { "rotate-180" } else { "" },
                                                        )
                                                    }
                                                    fill="none"
                                                    stroke="currentColor"
                                                    viewBox="0 0 24 24"
                                                >
                                                    <path
                                                        stroke-linecap="round"
                                                        stroke-linejoin="round"
                                                        stroke-width="2"
                                                        d="M19 9l-7 7-7-7"
                                                    />
                                                </svg>
                                            </button>

                                            // Facets (collapsible)
                                            <div class=move || {
                                                format!(
                                                    "print-expand overflow-hidden transition-all duration-300 {}",
                                                    if is_expanded() { "max-h-96" } else { "max-h-0" },
                                                )
                                            }>
                                                <div class="px-4 pb-4 space-y-3 border-t border-gray-100 dark:border-gray-700 pt-4">
                                                    {facets
                                                        .iter()
                                                        .map(|facet_score| {
                                                            let f_name = facet_name(facet_score.facet);
                                                            let facet_raw = facet_score.raw;
                                                            let facet_level = facet_score.level;
                                                            let facet_pct = facet_score.percentage();

                                                            view! {
                                                                <div>
                                                                    <div class="flex justify-between text-sm mb-1">
                                                                        <span class="text-gray-600 dark:text-gray-300">
                                                                            {f_name}
                                                                        </span>
                                                                        <span class="text-gray-500 dark:text-gray-400">
                                                                            {format!("{} ({})", facet_raw, level_text(facet_level))}
                                                                        </span>
                                                                    </div>
                                                                    <div class="w-full bg-gray-100 dark:bg-gray-600 rounded-full h-2">
                                                                        <div
                                                                            class=format!("{} h-2 rounded-full opacity-70", color)
                                                                            style:width=format!("{}%", facet_pct)
                                                                        />
                                                                    </div>
                                                                </div>
                                                            }
                                                        })
                                                        .collect_view()}
                                                </div>
                                            </div>
                                        </div>
                                    }
                                })
                                .collect_view()}
                        </div>

                        // AI Analysis section
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-8 transition-colors duration-300">
                            <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-100 mb-4">
                                {i18n.t("results_ai_title")}
                            </h2>

                            {move || {
                                if let Some(description) = ai_description.get() {
                                    let html_content = markdown_to_html(&description);
                                    view! {
                                        <div
                                            class="markdown max-w-none mb-4 text-gray-700 dark:text-gray-300"
                                            inner_html=html_content
                                        />
                                        <button
                                            on:click=request_ai
                                            class="no-print px-4 py-2 text-sm border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center"
                                        >
                                            <svg
                                                class="w-4 h-4 mr-2"
                                                fill="none"
                                                stroke="currentColor"
                                                viewBox="0 0 24 24"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"
                                                />
                                            </svg>
                                            {i18n.t("results_ai_regenerate")}
                                        </button>
                                    }
                                        .into_any()
                                } else if ai_loading.get() {
                                    view! {
                                        <div class="no-print py-12">
                                            // Animated AI icon
                                            <div class="flex justify-center mb-6">
                                                <div class="relative">
                                                    // Pulsing rings
                                                    <div class="absolute inset-0 animate-ping rounded-full bg-indigo-400 dark:bg-indigo-500 opacity-20" />
                                                    <div class="absolute inset-2 animate-pulse rounded-full bg-indigo-300 dark:bg-indigo-400 opacity-30" />
                                                    // Brain/AI icon
                                                    <div class="relative flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 shadow-lg">
                                                        <svg
                                                            class="w-8 h-8 text-white"
                                                            fill="none"
                                                            stroke="currentColor"
                                                            viewBox="0 0 24 24"
                                                        >
                                                            <path
                                                                stroke-linecap="round"
                                                                stroke-linejoin="round"
                                                                stroke-width="1.5"
                                                                d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"
                                                            />
                                                        </svg>
                                                    </div>
                                                </div>
                                            </div>
                                            // Loading text with animated dots
                                            <div class="text-center">
                                                <p class="text-lg font-medium text-gray-700 dark:text-gray-200 mb-2">
                                                    {i18n.t("results_ai_loading")}
                                                </p>
                                                <div class="flex justify-center gap-1">
                                                    <span
                                                        class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce"
                                                        style="animation-delay: 0ms"
                                                    />
                                                    <span
                                                        class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce"
                                                        style="animation-delay: 150ms"
                                                    />
                                                    <span
                                                        class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce"
                                                        style="animation-delay: 300ms"
                                                    />
                                                </div>
                                                <p class="text-sm text-gray-500 dark:text-gray-400 mt-4">
                                                    {i18n.t("results_ai_loading_hint")}
                                                </p>
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                } else if let Some(error) = ai_error.get() {
                                    view! {
                                        <div class="no-print bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-300 p-4 rounded-lg mb-4">
                                            <p class="font-medium">{i18n.t("results_ai_error")}</p>
                                            <p class="text-sm mt-1">{error}</p>
                                        </div>
                                        <button
                                            on:click=request_ai
                                            class="no-print px-6 py-2 bg-indigo-600 dark:bg-indigo-500 text-white rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 transition-colors"
                                        >
                                            {i18n.t("results_ai_retry")}
                                        </button>
                                    }
                                        .into_any()
                                } else {
                                    view! {
                                        <p class="no-print text-gray-600 dark:text-gray-300 mb-4">
                                            {i18n.t("results_ai_description")}
                                        </p>

                                        // Optional user context
                                        <div class="no-print mb-6">
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                {i18n.t("results_context_label")}
                                                <span class="text-gray-400 dark:text-gray-500 font-normal ml-1">
                                                    {i18n.t("results_context_optional")}
                                                </span>
                                            </label>
                                            <textarea
                                                class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 resize-none text-gray-700 dark:text-gray-200 placeholder:text-gray-400 dark:placeholder:text-gray-500"
                                                rows="3"
                                                placeholder=i18n.t("results_context_placeholder")
                                                prop:value=move || user_context.get()
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    save_context(&value);
                                                    set_user_context.set(value);
                                                }
                                            />
                                            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                                {i18n.t("results_context_hint")}
                                            </p>
                                        </div>

                                        // Model selector
                                        <div class="no-print mb-6">
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                {i18n.t("results_model_select")}
                                            </label>
                                            <select
                                                class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 text-gray-700 dark:text-gray-200"
                                                on:change=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    set_selected_model.set(Some(value));
                                                }
                                            >
                                                {move || {
                                                    let models = models_resource.get().flatten().unwrap_or_default();
                                                    let current = selected_model.get();
                                                    models
                                                        .iter()
                                                        .map(|m| {
                                                            let is_selected = current.as_ref() == Some(&m.id);
                                                            view! {
                                                                <option value=m.id.clone() selected=is_selected>
                                                                    {m.display_name.clone()}
                                                                </option>
                                                            }
                                                        })
                                                        .collect_view()
                                                }}
                                            </select>
                                        </div>

                                        <button
                                            on:click=request_ai
                                            class="no-print px-6 py-3 bg-indigo-600 dark:bg-indigo-500 text-white font-medium rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 transition-colors flex items-center"
                                        >
                                            <svg
                                                class="w-5 h-5 mr-2"
                                                fill="none"
                                                stroke="currentColor"
                                                viewBox="0 0 24 24"
                                            >
                                                <path
                                                    stroke-linecap="round"
                                                    stroke-linejoin="round"
                                                    stroke-width="2"
                                                    d="M13 10V3L4 14h7v7l9-11h-7z"
                                                />
                                            </svg>
                                            {i18n.t("results_ai_button")}
                                        </button>
                                    }
                                        .into_any()
                                }
                            }}
                        </div>

                        // Actions
                        <div class="no-print flex flex-wrap gap-4">
                            // Copy Link button
                            <button
                                on:click=copy_link
                                class="px-6 py-2 bg-indigo-600 dark:bg-indigo-500 text-white rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 transition-colors flex items-center"
                            >
                                {move || {
                                    if link_copied.get() {
                                        view! {
                                            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M5 13l4 4L19 7" />
                                            </svg>
                                        }.into_any()
                                    } else {
                                        view! {
                                            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.828 10.172a4 4 0 00-5.656 0l-4 4a4 4 0 105.656 5.656l1.102-1.101m-.758-4.899a4 4 0 005.656 0l4-4a4 4 0 00-5.656-5.656l-1.1 1.1" />
                                            </svg>
                                        }.into_any()
                                    }
                                }}
                                {move || if link_copied.get() { i18n.t("results_link_copied") } else { i18n.t("results_copy_link") }}
                            </button>
                            // Export as PDF
                            <button
                                on:click=move |_| {
                                    #[cfg(target_arch = "wasm32")]
                                    {
                                        if let Some(window) = web_sys::window() {
                                            let _ = window.print();
                                        }
                                    }
                                }
                                class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center"
                            >
                                <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                    <path
                                        stroke-linecap="round"
                                        stroke-linejoin="round"
                                        stroke-width="2"
                                        d="M17 17h2a2 2 0 002-2v-4a2 2 0 00-2-2H5a2 2 0 00-2 2v4a2 2 0 002 2h2m2 4h6a2 2 0 002-2v-4a2 2 0 00-2-2H9a2 2 0 00-2 2v4a2 2 0 002 2zm8-12V5a2 2 0 00-2-2H9a2 2 0 00-2 2v4h10z"
                                    />
                                </svg>
                                {i18n.t("results_export_pdf")}
                            </button>
                            <A
                                href=move || format!("{}/test", i18n.get_locale().path_prefix())
                                attr:class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                            >
                                {i18n.t("results_retake")}
                            </A>
                            <A
                                href=move || i18n.get_locale().path_prefix().to_string()
                                attr:class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                            >
                                {i18n.t("results_home")}
                            </A>
                        </div>
                    </div>
                }
                    .into_any()
            }}
        </div>
    }
}

fn load_profile() -> Option<PersonalityProfile> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        let json = storage.get_item(STORAGE_KEY_PROFILE).ok()??;
        serde_json::from_str(&json).ok()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn load_context() -> Option<String> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window()?;
        let storage = window.local_storage().ok()??;
        storage.get_item(STORAGE_KEY_CONTEXT).ok()?
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

fn save_context(context: &str) {
    #[cfg(target_arch = "wasm32")]
    {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let _ = storage.set_item(STORAGE_KEY_CONTEXT, context);
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = context;
    }
}
