//! Home page component with landing section and info.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{LangToggle, ThemeToggle};
use crate::i18n::use_i18n;

/// Domain trait with description.
#[component]
fn DomainItem(name: &'static str, description: impl IntoView + 'static) -> impl IntoView {
    let color = match name {
        "Neuroticism" => "bg-red-500",
        "Extraversion" => "bg-yellow-500",
        "Openness" => "bg-purple-500",
        "Agreeableness" => "bg-green-500",
        "Conscientiousness" => "bg-blue-500",
        _ => "bg-gray-500",
    };

    view! {
        <div class="flex items-start">
            <div class=format!("w-3 h-3 rounded-full {} mt-1.5 mr-3 flex-shrink-0", color) />
            <div>
                <h4 class="font-medium text-gray-800 dark:text-gray-100">{name}</h4>
                <p class="text-sm text-gray-500 dark:text-gray-400">{description}</p>
            </div>
        </div>
    }
}

/// Home page with landing section.
#[component]
pub fn HomePage() -> impl IntoView {
    let i18n = use_i18n();

    // Domain descriptions
    let domain_n_desc = move || i18n.t("domain_n_desc");
    let domain_e_desc = move || i18n.t("domain_e_desc");
    let domain_o_desc = move || i18n.t("domain_o_desc");
    let domain_a_desc = move || i18n.t("domain_a_desc");
    let domain_c_desc = move || i18n.t("domain_c_desc");

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            // Stats pills at the top
            <div class="flex flex-wrap gap-3 justify-center mb-8">
                <span class="px-3 py-1 bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 rounded-full text-sm font-medium">
                    {move || i18n.t("home_questions_count")}
                </span>
                <span class="px-3 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded-full text-sm font-medium">
                    {move || i18n.t("home_time_estimate")}
                </span>
            </div>

            // Main content card
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-8 transition-colors duration-300">
                // Header with title and toggles
                <header class="flex justify-between items-start mb-6">
                    <h1 class="text-3xl font-bold text-gray-900 dark:text-white">{move || i18n.t("title")}</h1>
                    <div class="flex items-center gap-3">
                        <LangToggle />
                        <ThemeToggle />
                    </div>
                </header>

                // Subtitle and description
                <h2 class="text-2xl font-semibold text-gray-800 dark:text-gray-100 mb-4">
                    {move || i18n.t("home_subtitle")}
                </h2>
                <p class="text-gray-600 dark:text-gray-300 mb-6 leading-relaxed">
                    {move || i18n.t("home_description")}
                </p>

                // Domain traits
                <div class="mb-8">
                    <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">
                        {move || i18n.t("home_what_measured")}
                    </h3>
                    <div class="space-y-4">
                        <DomainItem name="Neuroticism" description=domain_n_desc />
                        <DomainItem name="Extraversion" description=domain_e_desc />
                        <DomainItem name="Openness" description=domain_o_desc />
                        <DomainItem name="Agreeableness" description=domain_a_desc />
                        <DomainItem name="Conscientiousness" description=domain_c_desc />
                    </div>
                </div>

                // Start button
                <A
                    href="test"
                    attr:class="inline-block w-full sm:w-auto text-center px-8 py-3 bg-indigo-600 dark:bg-indigo-500 text-white font-semibold rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 transition-colors"
                >
                    {move || i18n.t("home_start_button")}
                </A>
            </div>
        </div>
    }
}
