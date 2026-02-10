//! Home page component with landing section and info.

use leptos::prelude::*;
use leptos_router::components::A;

use crate::components::{LangToggle, ThemeToggle};
use crate::i18n::use_i18n;

/// Domain trait with description.
#[component]
fn DomainItem(color: &'static str, name: &'static str, description: &'static str) -> impl IntoView {
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

/// Domain list component - extracted to reduce type nesting.
#[component]
fn DomainList() -> impl IntoView {
    let i18n = use_i18n();

    // Get translations once per render (reactive via locale signal)
    let domains = Memo::new(move |_| {
        vec![
            ("bg-red-500", i18n.t("domain_neuroticism"), i18n.t("domain_n_desc")),
            ("bg-yellow-500", i18n.t("domain_extraversion"), i18n.t("domain_e_desc")),
            ("bg-purple-500", i18n.t("domain_openness"), i18n.t("domain_o_desc")),
            ("bg-green-500", i18n.t("domain_agreeableness"), i18n.t("domain_a_desc")),
            ("bg-blue-500", i18n.t("domain_conscientiousness"), i18n.t("domain_c_desc")),
        ]
    });

    view! {
        <div class="space-y-4">
            <For each=move || domains.get() key=|(color, name, _)| format!("{}{}", color, name) let:item>
                <DomainItem color=item.0 name=item.1 description=item.2 />
            </For>
        </div>
    }
}

/// Stats pills component.
#[component]
fn StatsPills() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="flex flex-wrap gap-3 justify-center mb-8">
            <span class="px-3 py-1 bg-indigo-100 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 rounded-full text-sm font-medium">
                {move || i18n.t("home_questions_count")}
            </span>
            <span class="px-3 py-1 bg-green-100 dark:bg-green-900/30 text-green-700 dark:text-green-300 rounded-full text-sm font-medium">
                {move || i18n.t("home_time_estimate")}
            </span>
        </div>
    }
}

/// Header with title and toggles.
#[component]
fn PageHeader() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <header class="flex justify-between items-start mb-6">
            <h1 class="text-3xl font-bold text-gray-900 dark:text-white">{move || i18n.t("title")}</h1>
            <div class="flex items-center gap-3">
                <LangToggle />
                <ThemeToggle />
            </div>
        </header>
    }
}

/// Home page with landing section.
#[component]
pub fn HomePage() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            <StatsPills />

            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-8 transition-colors duration-300">
                <PageHeader />

                <h2 class="text-2xl font-semibold text-gray-800 dark:text-gray-100 mb-4">
                    {move || i18n.t("home_subtitle")}
                </h2>
                <p class="text-gray-600 dark:text-gray-300 mb-6 leading-relaxed">
                    {move || i18n.t("home_description")}
                </p>

                <div class="mb-8">
                    <h3 class="text-lg font-semibold text-gray-700 dark:text-gray-200 mb-4">
                        {move || i18n.t("home_what_measured")}
                    </h3>
                    <DomainList />
                </div>

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
