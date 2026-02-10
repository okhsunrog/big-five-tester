//! Test page component with quiz UI.

use bigfive::{Answer, Ipip120};
use leptos::prelude::*;
use leptos_router::hooks::use_navigate;

use crate::components::{LangToggle, ThemeToggle};
use crate::i18n::{Locale, use_i18n};

#[cfg(target_arch = "wasm32")]
const STORAGE_KEY_ANSWERS: &str = "bigfive_answers";
#[cfg(target_arch = "wasm32")]
const STORAGE_KEY_INDEX: &str = "bigfive_current_index";
#[cfg(target_arch = "wasm32")]
const STORAGE_KEY_PROFILE: &str = "bigfive_profile";

/// Test page with one question at a time.
#[component]
pub fn TestPage() -> impl IntoView {
    let i18n = use_i18n();
    let navigate = use_navigate();

    // Load inventory based on language
    let inventory = Memo::new(move |_| match i18n.get_locale() {
        Locale::En => Ipip120::english(),
        Locale::Ru => Ipip120::russian(),
    });

    // Current question index (0-119)
    let (current_index, set_current_index) = signal(load_current_index());

    // Answers map: question_id -> value
    let (answers, set_answers) = signal(load_answers());

    // Save state to localStorage whenever it changes
    Effect::new(move |_| {
        save_current_index(current_index.get());
    });

    Effect::new(move |_| {
        save_answers(&answers.get());
    });

    // Get current question
    let current_question = move || {
        let inv = inventory.get();
        let idx = current_index.get();
        inv.questions().get(idx).cloned()
    };

    // Get current answer value (if any)
    let current_answer = move || {
        let q = current_question()?;
        answers.get().get(&q.id).copied()
    };

    // Check if we can go to results
    let all_answered = move || {
        let inv = inventory.get();
        let ans = answers.get();
        inv.questions().iter().all(|q| ans.contains_key(&q.id))
    };

    // Handle answer selection
    let select_answer = move |value: u8| {
        if let Some(q) = current_question() {
            set_answers.update(|ans| {
                ans.insert(q.id.clone(), value);
            });

            // Auto-advance to next question
            let idx = current_index.get();
            if idx < 119 {
                set_current_index.set(idx + 1);
            }
        }
    };

    // Navigation handlers
    let go_prev = move |_| {
        let idx = current_index.get();
        if idx > 0 {
            set_current_index.set(idx - 1);
        }
    };

    let go_next = move |_| {
        let idx = current_index.get();
        if idx < 119 {
            set_current_index.set(idx + 1);
        }
    };

    // Submit and calculate results - use Action for async-like behavior
    let submit_action = Action::new(move |_: &()| {
        let inv = inventory.get();
        let ans = answers.get();
        let nav = navigate.clone();

        async move {
            let answer_vec: Vec<Answer> = inv
                .questions()
                .iter()
                .filter_map(|q| {
                    ans.get(&q.id).map(|&value| Answer {
                        question_id: q.id.clone(),
                        value,
                    })
                })
                .collect();

            if answer_vec.len() == 120 {
                match bigfive::calculate(&inv, &answer_vec) {
                    Ok(profile) => {
                        save_profile(&profile);
                        clear_test_progress();
                        nav("results", Default::default());
                    }
                    Err(e) => {
                        leptos::logging::error!("Failed to calculate profile: {}", e);
                    }
                }
            }
        }
    });

    // Answer button labels
    let answer_labels = move || {
        vec![
            (1u8, i18n.t("answer_1").to_string()),
            (2u8, i18n.t("answer_2").to_string()),
            (3u8, i18n.t("answer_3").to_string()),
            (4u8, i18n.t("answer_4").to_string()),
            (5u8, i18n.t("answer_5").to_string()),
        ]
    };

    view! {
        <div class="max-w-2xl mx-auto px-4 py-8">
            // Header with language and theme toggles
            <header class="flex justify-between items-center mb-8">
                <h1 class="text-2xl font-bold text-gray-900 dark:text-white">{move || i18n.t("title")}</h1>
                <div class="flex items-center gap-3">
                    <LangToggle />
                    <ThemeToggle />
                </div>
            </header>

            // Progress bar
            <div class="mb-8">
                <div class="flex justify-between text-sm text-gray-600 dark:text-gray-400 mb-2">
                    <span>
                        {move || { format!("{} {}/120", i18n.t("test_question"), current_index.get() + 1) }}
                    </span>
                    <span>{move || { format!("{}%", ((current_index.get() + 1) as f32 / 120.0 * 100.0) as u8) }}</span>
                </div>
                <div class="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-2.5">
                    <div
                        class="bg-indigo-600 dark:bg-indigo-500 h-2.5 rounded-full transition-all duration-300"
                        style:width=move || { format!("{}%", ((current_index.get() + 1) as f32 / 120.0 * 100.0)) }
                    />
                </div>
            </div>

            // Question card
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-6 mb-6 transition-colors duration-300">
                <p class="text-xl text-gray-800 dark:text-gray-100 text-center mb-8 min-h-[3rem]">
                    {move || current_question().map(|q| q.text.clone()).unwrap_or_default()}
                </p>

                // Answer buttons
                <div class="space-y-3">
                    {move || {
                        let current = current_answer();
                        answer_labels()
                            .into_iter()
                            .map(|(value, label)| {
                                let is_selected = current == Some(value);
                                let select = move |_| select_answer(value);

                                view! {
                                    <button
                                        on:click=select
                                        class=move || {
                                            let base = "w-full py-3 px-4 rounded-lg border-2 font-medium transition-all duration-200 text-left";
                                            if is_selected {
                                                format!(
                                                    "{} border-indigo-600 dark:border-indigo-400 bg-indigo-50 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300",
                                                    base,
                                                )
                                            } else {
                                                format!(
                                                    "{} border-gray-200 dark:border-gray-600 hover:border-indigo-300 dark:hover:border-indigo-500 hover:bg-gray-50 dark:hover:bg-gray-700 text-gray-700 dark:text-gray-300",
                                                    base,
                                                )
                                            }
                                        }
                                    >
                                        <span class="flex items-center">
                                            <span class=move || {
                                                let base = "w-6 h-6 rounded-full border-2 mr-3 flex items-center justify-center";
                                                if is_selected {
                                                    format!(
                                                        "{} border-indigo-600 dark:border-indigo-400 bg-indigo-600 dark:bg-indigo-500",
                                                        base,
                                                    )
                                                } else {
                                                    format!("{} border-gray-300 dark:border-gray-500", base)
                                                }
                                            }>
                                                {move || {
                                                    if is_selected {
                                                        view! { <span class="w-2 h-2 rounded-full bg-white" /> }
                                                            .into_any()
                                                    } else {
                                                        view! { <span /> }.into_any()
                                                    }
                                                }}
                                            </span>
                                            {label.clone()}
                                        </span>
                                    </button>
                                }
                            })
                            .collect_view()
                    }}
                </div>
            </div>

            // Navigation buttons
            <div class="flex justify-between items-center">
                <button
                    on:click=go_prev
                    prop:disabled=move || current_index.get() == 0
                    class="px-6 py-2 rounded-lg border border-gray-300 dark:border-gray-600 text-gray-700 dark:text-gray-300 font-medium hover:bg-gray-50 dark:hover:bg-gray-700 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                >
                    {move || i18n.t("test_back")}
                </button>

                {move || {
                    let idx = current_index.get();
                    let all_done = all_answered();
                    let is_last = idx >= 119;
                    if idx == 119 && all_done {

                        view! {
                            <button
                                on:click=move |_| {
                                    submit_action.dispatch(());
                                }
                                class="px-6 py-2 rounded-lg bg-green-600 dark:bg-green-500 text-white font-medium hover:bg-green-700 dark:hover:bg-green-600 transition-colors"
                            >
                                {move || i18n.t("test_show_results")}
                            </button>
                        }
                            .into_any()
                    } else {
                        view! {
                            <button
                                on:click=go_next
                                prop:disabled=is_last
                                class="px-6 py-2 rounded-lg bg-indigo-600 dark:bg-indigo-500 text-white font-medium hover:bg-indigo-700 dark:hover:bg-indigo-600 disabled:opacity-50 disabled:cursor-not-allowed transition-colors"
                            >
                                {move || i18n.t("test_next")}
                            </button>
                        }
                            .into_any()
                    }
                }}
            </div>

            // Answered count
            <div class="mt-6 text-center text-sm text-gray-500 dark:text-gray-400">
                {move || {
                    let answered = answers.get().len();
                    format!("{}: {}/120", i18n.t("test_answered"), answered)
                }}
            </div>
        </div>
    }
}

// localStorage helpers

fn load_current_index() -> usize {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().expect("no window");
        let storage = window
            .local_storage()
            .ok()
            .flatten()
            .expect("no localStorage");
        storage
            .get_item(STORAGE_KEY_INDEX)
            .ok()
            .flatten()
            .and_then(|s| s.parse().ok())
            .unwrap_or(0)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        0
    }
}

#[cfg(target_arch = "wasm32")]
fn save_current_index(index: usize) {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .ok()
        .flatten()
        .expect("no localStorage");
    let _ = storage.set_item(STORAGE_KEY_INDEX, &index.to_string());
}

#[cfg(not(target_arch = "wasm32"))]
fn save_current_index(_index: usize) {}

fn load_answers() -> std::collections::HashMap<String, u8> {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().expect("no window");
        let storage = window
            .local_storage()
            .ok()
            .flatten()
            .expect("no localStorage");
        storage
            .get_item(STORAGE_KEY_ANSWERS)
            .ok()
            .flatten()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default()
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        std::collections::HashMap::new()
    }
}

#[cfg(target_arch = "wasm32")]
fn save_answers(answers: &std::collections::HashMap<String, u8>) {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .ok()
        .flatten()
        .expect("no localStorage");
    if let Ok(json) = serde_json::to_string(answers) {
        let _ = storage.set_item(STORAGE_KEY_ANSWERS, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_answers(_answers: &std::collections::HashMap<String, u8>) {}

#[cfg(target_arch = "wasm32")]
fn save_profile(profile: &bigfive::PersonalityProfile) {
    let window = web_sys::window().expect("no window");
    let storage = window
        .local_storage()
        .ok()
        .flatten()
        .expect("no localStorage");
    if let Ok(json) = serde_json::to_string(profile) {
        let _ = storage.set_item(STORAGE_KEY_PROFILE, &json);
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_profile(_profile: &bigfive::PersonalityProfile) {}

fn clear_test_progress() {
    #[cfg(target_arch = "wasm32")]
    {
        let window = web_sys::window().expect("no window");
        let storage = window
            .local_storage()
            .ok()
            .flatten()
            .expect("no localStorage");
        let _ = storage.remove_item(STORAGE_KEY_ANSWERS);
        let _ = storage.remove_item(STORAGE_KEY_INDEX);
    }
}
