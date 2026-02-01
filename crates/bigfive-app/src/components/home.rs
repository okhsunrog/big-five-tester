//! Home page component.

use leptos::prelude::*;
use leptos_i18n::t;
use leptos_router::components::A;

use crate::components::LangToggle;
use crate::i18n::use_i18n;

/// Home page with test description and start button.
#[component]
pub fn HomePage() -> impl IntoView {
    let i18n = use_i18n();

    view! {
        <div class="max-w-4xl mx-auto px-4 py-8">
            // Header with language toggle
            <header class="flex justify-between items-center mb-12">
                <h1 class="text-3xl font-bold text-gray-900 dark:text-white">
                    {t!(i18n, title)}
                </h1>
                <LangToggle/>
            </header>

            // Main content
            <div class="bg-white dark:bg-gray-800 rounded-lg shadow-md p-8 transition-colors duration-300">
                <h2 class="text-2xl font-semibold text-gray-800 dark:text-gray-100 mb-4">
                    {t!(i18n, home_subtitle)}
                </h2>

                <p class="text-gray-600 dark:text-gray-300 mb-6 leading-relaxed">
                    {t!(i18n, home_description)}
                </p>

                // Test info
                <div class="bg-gray-50 dark:bg-gray-700 rounded-lg p-6 mb-8 transition-colors duration-300">
                    <h3 class="text-lg font-medium text-gray-800 dark:text-gray-100 mb-4">
                        {t!(i18n, home_what_measured)}
                    </h3>
                    <ul class="space-y-2 text-gray-600 dark:text-gray-300">
                        <li class="flex items-start">
                            <span class="text-indigo-500 dark:text-indigo-400 mr-2">"•"</span>
                            <span><strong class="text-gray-800 dark:text-gray-100">"Neuroticism"</strong>" - "{t!(i18n, domain_n_desc)}</span>
                        </li>
                        <li class="flex items-start">
                            <span class="text-indigo-500 dark:text-indigo-400 mr-2">"•"</span>
                            <span><strong class="text-gray-800 dark:text-gray-100">"Extraversion"</strong>" - "{t!(i18n, domain_e_desc)}</span>
                        </li>
                        <li class="flex items-start">
                            <span class="text-indigo-500 dark:text-indigo-400 mr-2">"•"</span>
                            <span><strong class="text-gray-800 dark:text-gray-100">"Openness"</strong>" - "{t!(i18n, domain_o_desc)}</span>
                        </li>
                        <li class="flex items-start">
                            <span class="text-indigo-500 dark:text-indigo-400 mr-2">"•"</span>
                            <span><strong class="text-gray-800 dark:text-gray-100">"Agreeableness"</strong>" - "{t!(i18n, domain_a_desc)}</span>
                        </li>
                        <li class="flex items-start">
                            <span class="text-indigo-500 dark:text-indigo-400 mr-2">"•"</span>
                            <span><strong class="text-gray-800 dark:text-gray-100">"Conscientiousness"</strong>" - "{t!(i18n, domain_c_desc)}</span>
                        </li>
                    </ul>
                </div>

                // Info badges
                <div class="flex flex-wrap gap-4 mb-8">
                    <div class="flex items-center bg-indigo-50 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 px-4 py-2 rounded-full">
                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5H7a2 2 0 00-2 2v12a2 2 0 002 2h10a2 2 0 002-2V7a2 2 0 00-2-2h-2M9 5a2 2 0 002 2h2a2 2 0 002-2M9 5a2 2 0 012-2h2a2 2 0 012 2"/>
                        </svg>
                        {t!(i18n, home_questions_count)}
                    </div>
                    <div class="flex items-center bg-green-50 dark:bg-green-900/30 text-green-700 dark:text-green-300 px-4 py-2 rounded-full">
                        <svg class="w-5 h-5 mr-2" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                            <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 8v4l3 3m6-3a9 9 0 11-18 0 9 9 0 0118 0z"/>
                        </svg>
                        {t!(i18n, home_time_estimate)}
                    </div>
                </div>

                // Start button
                <A
                    href="/test"
                    attr:class="inline-block w-full sm:w-auto px-8 py-4 bg-indigo-600 dark:bg-indigo-500 text-white font-semibold rounded-lg hover:bg-indigo-700 dark:hover:bg-indigo-600 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors text-center text-lg"
                >
                    {t!(i18n, home_start_button)}
                </A>
            </div>
        </div>
    }
}
