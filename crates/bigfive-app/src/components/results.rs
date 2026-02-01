//! Results page component with visualization and AI analysis.

use bigfive::{Domain, PersonalityProfile, ScoreLevel};
use leptos::prelude::*;
use leptos::task::spawn_local;
use leptos_i18n::{t, t_string};
use leptos_router::components::A;
use leptos_router::hooks::use_navigate;
use pulldown_cmark::{Options, Parser, html};

use crate::components::{LangToggle, ThemeToggle};
use crate::i18n::{Locale, use_i18n};

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY_PROFILE: &str = "bigfive_profile";
#[cfg(target_arch = "wasm32")]
const STORAGE_KEY_CONTEXT: &str = "bigfive_user_context";

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

// Server function for AI description generation
#[server]
pub async fn generate_description(
    profile: PersonalityProfile,
    lang: String,
    user_context: Option<String>,
) -> Result<String, ServerFnError> {
    use crate::ai;

    // Load .env file for local development
    dotenvy::dotenv().ok();

    // Use the new AI pipeline
    let description = ai::generate_analysis(&profile, user_context.as_deref(), &lang)
        .await
        .map_err(|e| ServerFnError::new(e.to_string()))?;

    // Save description to file
    if let Err(e) = save_description_to_file(&description, &lang, user_context.as_deref()) {
        tracing::warn!("Failed to save description to file: {}", e);
    }

    Ok(description)
}

/// Save generated description to a file in the analyses directory.
#[cfg(feature = "ssr")]
fn save_description_to_file(
    description: &str,
    lang: &str,
    user_context: Option<&str>,
) -> std::io::Result<()> {
    use std::fs;
    use std::io::Write;
    use std::path::Path;

    // Get output directory from env or use default
    let output_dir = std::env::var("ANALYSES_DIR").unwrap_or_else(|_| "analyses".to_string());
    let dir_path = Path::new(&output_dir);

    // Create directory if it doesn't exist
    fs::create_dir_all(dir_path)?;

    // Generate filename with timestamp
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let filename = format!("{}_{}.md", timestamp, lang);
    let file_path = dir_path.join(&filename);

    // Build file content
    let mut content = String::new();
    content.push_str(&format!(
        "# Personality Analysis\n\n**Generated:** {}\n**Language:** {}\n",
        chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"),
        lang
    ));

    if let Some(ctx) = user_context
        && !ctx.trim().is_empty() {
            content.push_str(&format!("\n**User Context:**\n{}\n", ctx));
        }

    content.push_str("\n---\n\n");
    content.push_str(description);

    // Write to file
    let mut file = std::fs::File::create(&file_path)?;
    file.write_all(content.as_bytes())?;

    tracing::info!("Saved analysis to {}", file_path.display());

    Ok(())
}

/// Results page with score visualization and AI-generated description.
#[component]
pub fn ResultsPage() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    // Profile state - starts as None, loaded via Effect to avoid hydration mismatch
    let (profile, set_profile) = signal::<Option<PersonalityProfile>>(None);

    // Expanded domain state (for facet accordion)
    let (expanded_domain, set_expanded_domain) = signal::<Option<Domain>>(None);

    // AI description state
    let (ai_description, set_ai_description) = signal::<Option<String>>(None);
    let (ai_loading, set_ai_loading) = signal(false);
    let (ai_error, set_ai_error) = signal::<Option<String>>(None);

    // User context for AI (optional self-description)
    let (user_context, set_user_context) = signal(String::new());

    // Load profile and context from localStorage after hydration
    Effect::new(move |_| {
        let loaded = load_profile();
        if loaded.is_none() {
            navigate("/test", Default::default());
        } else {
            set_profile.set(loaded);
        }
        if let Some(ctx) = load_context() {
            set_user_context.set(ctx);
        }
    });

    // Request AI description
    let request_ai = move |_| {
        let Some(prof) = profile.get() else { return };
        let locale = i18n.get_locale();
        let context = user_context.get();
        let context_opt = if context.trim().is_empty() {
            None
        } else {
            Some(context)
        };

        set_ai_loading.set(true);
        set_ai_error.set(None);

        spawn_local(async move {
            let lang_str = match locale {
                Locale::en => "en",
                Locale::ru => "ru",
            };

            match generate_description(prof, lang_str.to_string(), context_opt).await {
                Ok(description) => {
                    set_ai_description.set(Some(description));
                }
                Err(e) => {
                    set_ai_error.set(Some(e.to_string()));
                }
            }
            set_ai_loading.set(false);
        });
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
            ScoreLevel::Low => t_string!(i18n, level_low).to_string(),
            ScoreLevel::Neutral => t_string!(i18n, level_neutral).to_string(),
            ScoreLevel::High => t_string!(i18n, level_high).to_string(),
        }
    };

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            // Header with language and theme toggles
            <header class="flex justify-between items-center mb-8">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">
                    {t!(i18n, results_title)}
                </h1>
                <div class="flex items-center gap-3">
                    <LangToggle/>
                    <ThemeToggle/>
                </div>
            </header>

            {move || {
                let Some(prof) = profile.get() else {
                    return view! { <div>"Loading..."</div> }.into_any();
                };

                view! {
                    <div>
                        // Domain scores
                        <div class="space-y-4 mb-8">
                            {prof.domains.iter().map(|domain_score| {
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
                                                        {domain.name()}
                                                    </h3>
                                                    <span class="text-sm text-gray-500 dark:text-gray-400">
                                                        {format!("{} ({})", raw, level_text(level))}
                                                    </span>
                                                </div>
                                                // Score bar
                                                <div class="w-full bg-gray-200 dark:bg-gray-600 rounded-full h-3">
                                                    <div
                                                        class=format!("{} h-3 rounded-full transition-all duration-500", color)
                                                        style:width=format!("{}%", percentage)
                                                    />
                                                </div>
                                            </div>
                                            // Expand icon
                                            <svg
                                                class=move || format!(
                                                    "w-5 h-5 ml-4 text-gray-400 dark:text-gray-500 transition-transform duration-200 {}",
                                                    if is_expanded() { "rotate-180" } else { "" }
                                                )
                                                fill="none"
                                                stroke="currentColor"
                                                viewBox="0 0 24 24"
                                            >
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                                            </svg>
                                        </button>

                                        // Facets (collapsible)
                                        <div
                                            class=move || format!(
                                                "overflow-hidden transition-all duration-300 {}",
                                                if is_expanded() { "max-h-96" } else { "max-h-0" }
                                            )
                                        >
                                            <div class="px-4 pb-4 space-y-3 border-t border-gray-100 dark:border-gray-700 pt-4">
                                                {facets.iter().map(|facet_score| {
                                                    let facet_name = facet_score.facet.name();
                                                    let facet_raw = facet_score.raw;
                                                    let facet_level = facet_score.level;
                                                    let facet_pct = facet_score.percentage();

                                                    view! {
                                                        <div>
                                                            <div class="flex justify-between text-sm mb-1">
                                                                <span class="text-gray-600 dark:text-gray-300">{facet_name}</span>
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
                                                }).collect_view()}
                                            </div>
                                        </div>
                                    </div>
                                }
                            }).collect_view()}
                        </div>

                        // AI Analysis section
                        <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-8 transition-colors duration-300">
                            <h2 class="text-xl font-semibold text-gray-800 dark:text-gray-100 mb-4">
                                {t!(i18n, results_ai_title)}
                            </h2>

                            {move || {
                                if let Some(description) = ai_description.get() {
                                    let html_content = markdown_to_html(&description);
                                    view! {
                                        <div
                                            class="prose prose-gray dark:prose-dark max-w-none mb-4 prose-headings:text-gray-800 dark:prose-headings:text-gray-100 prose-p:text-gray-700 dark:prose-p:text-gray-300 prose-strong:text-gray-800 dark:prose-strong:text-gray-100 prose-ul:text-gray-700 dark:prose-ul:text-gray-300 prose-li:text-gray-700 dark:prose-li:text-gray-300"
                                            inner_html=html_content
                                        />
                                        <button
                                            on:click=request_ai
                                            class="px-4 py-2 text-sm border border-gray-300 dark:border-gray-600 text-gray-600 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors flex items-center"
                                        >
                                            <svg class="w-4 h-4 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15"/>
                                            </svg>
                                            {t!(i18n, results_ai_regenerate)}
                                        </button>
                                    }.into_any()
                                } else if ai_loading.get() {
                                    view! {
                                        <div class="py-12">
                                            // Animated AI icon
                                            <div class="flex justify-center mb-6">
                                                <div class="relative">
                                                    // Pulsing rings
                                                    <div class="absolute inset-0 animate-ping rounded-full bg-indigo-400 dark:bg-indigo-500 opacity-20"/>
                                                    <div class="absolute inset-2 animate-pulse rounded-full bg-indigo-300 dark:bg-indigo-400 opacity-30"/>
                                                    // Brain/AI icon
                                                    <div class="relative flex items-center justify-center w-16 h-16 rounded-full bg-gradient-to-br from-indigo-500 to-purple-600 shadow-lg">
                                                        <svg class="w-8 h-8 text-white" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="1.5" d="M9.663 17h4.673M12 3v1m6.364 1.636l-.707.707M21 12h-1M4 12H3m3.343-5.657l-.707-.707m2.828 9.9a5 5 0 117.072 0l-.548.547A3.374 3.374 0 0014 18.469V19a2 2 0 11-4 0v-.531c0-.895-.356-1.754-.988-2.386l-.548-.547z"/>
                                                        </svg>
                                                    </div>
                                                </div>
                                            </div>
                                            // Loading text with animated dots
                                            <div class="text-center">
                                                <p class="text-lg font-medium text-gray-700 dark:text-gray-200 mb-2">
                                                    {t!(i18n, results_ai_loading)}
                                                </p>
                                                <div class="flex justify-center gap-1">
                                                    <span class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce" style="animation-delay: 0ms"/>
                                                    <span class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce" style="animation-delay: 150ms"/>
                                                    <span class="w-2 h-2 bg-indigo-500 rounded-full animate-bounce" style="animation-delay: 300ms"/>
                                                </div>
                                                <p class="text-sm text-gray-500 dark:text-gray-400 mt-4">
                                                    {t!(i18n, results_ai_loading_hint)}
                                                </p>
                                            </div>
                                        </div>
                                    }.into_any()
                                } else if let Some(error) = ai_error.get() {
                                    view! {
                                        <div class="bg-red-50 dark:bg-red-900/30 text-red-700 dark:text-red-300 p-4 rounded-lg mb-4">
                                            <p class="font-medium">{t!(i18n, results_ai_error)}</p>
                                            <p class="text-sm mt-1">{error}</p>
                                        </div>
                                        <button
                                            on:click=request_ai
                                            class="px-6 py-2 bg-indigo-600 dark:bg-indigo-500 text-white rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 transition-colors"
                                        >
                                            {t!(i18n, results_ai_retry)}
                                        </button>
                                    }.into_any()
                                } else {
                                    view! {
                                        <p class="text-gray-600 dark:text-gray-300 mb-4">
                                            {t!(i18n, results_ai_description)}
                                        </p>

                                        // Optional user context
                                        <div class="mb-6">
                                            <label class="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-2">
                                                {t!(i18n, results_context_label)}
                                                <span class="text-gray-400 dark:text-gray-500 font-normal ml-1">
                                                    {t!(i18n, results_context_optional)}
                                                </span>
                                            </label>
                                            <textarea
                                                class="w-full px-4 py-3 border border-gray-300 dark:border-gray-600 bg-white dark:bg-gray-700 rounded-lg focus:ring-2 focus:ring-indigo-500 focus:border-indigo-500 resize-none text-gray-700 dark:text-gray-200 placeholder:text-gray-400 dark:placeholder:text-gray-500"
                                                rows="3"
                                                placeholder=t_string!(i18n, results_context_placeholder)
                                                prop:value=move || user_context.get()
                                                on:input=move |ev| {
                                                    let value = event_target_value(&ev);
                                                    save_context(&value);
                                                    set_user_context.set(value);
                                                }
                                            />
                                            <p class="mt-1 text-xs text-gray-500 dark:text-gray-400">
                                                {t!(i18n, results_context_hint)}
                                            </p>
                                        </div>

                                        <button
                                            on:click=request_ai
                                            class="px-6 py-3 bg-indigo-600 dark:bg-indigo-500 text-white font-medium rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 transition-colors flex items-center"
                                        >
                                            <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                                                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 10V3L4 14h7v7l9-11h-7z"/>
                                            </svg>
                                            {t!(i18n, results_ai_button)}
                                        </button>
                                    }.into_any()
                                }
                            }}
                        </div>

                        // Actions
                        <div class="flex flex-wrap gap-4">
                            <A
                                href="/test"
                                attr:class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                            >
                                {t!(i18n, results_retake)}
                            </A>
                            <A
                                href="/"
                                attr:class="px-6 py-2 border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 transition-colors"
                            >
                                {t!(i18n, results_home)}
                            </A>
                        </div>
                    </div>
                }.into_any()
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
        if let Some(window) = web_sys::window() {
            if let Ok(Some(storage)) = window.local_storage() {
                let _ = storage.set_item(STORAGE_KEY_CONTEXT, context);
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = context;
    }
}
