//! Language selector dropdown component.

use leptos::prelude::*;

use crate::i18n::{Locale, use_i18n};

/// Get the native name of a locale
fn locale_name(locale: Locale) -> &'static str {
    match locale {
        Locale::en => "English",
        Locale::ru => "Русский",
    }
}

/// Language selector dropdown component.
#[component]
pub fn LangToggle() -> impl IntoView {
    let i18n = use_i18n();
    let (is_open, set_is_open) = signal(false);

    let toggle_dropdown = move |_| {
        set_is_open.update(|open| *open = !*open);
    };

    let close_dropdown = move |_| {
        set_is_open.set(false);
    };

    let select_locale = move |locale: Locale| {
        move |_| {
            i18n.set_locale(locale);
            set_is_open.set(false);
        }
    };

    let current_locale = move || i18n.get_locale();
    let current_name = move || locale_name(current_locale());

    // All available locales
    let locales = [Locale::en, Locale::ru];

    view! {
        <div class="relative">
            // Dropdown button
            <button
                on:click=toggle_dropdown
                class="flex items-center gap-2 px-3 py-2 text-sm font-medium text-gray-700 dark:text-gray-200 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-600 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 transition-colors"
            >
                <svg class="w-4 h-4" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M21 12a9 9 0 01-9 9m9-9a9 9 0 00-9-9m9 9H3m9 9a9 9 0 01-9-9m9 9c1.657 0 3-4.03 3-9s-1.343-9-3-9m0 18c-1.657 0-3-4.03-3-9s1.343-9 3-9m-9 9a9 9 0 019-9"/>
                </svg>
                <span>{current_name}</span>
                <svg
                    class=move || format!("w-4 h-4 transition-transform {}", if is_open.get() { "rotate-180" } else { "" })
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                >
                    <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M19 9l-7 7-7-7"/>
                </svg>
            </button>

            // Dropdown menu
            <Show when=move || is_open.get()>
                // Backdrop to close on click outside
                <div
                    class="fixed inset-0 z-10"
                    on:click=close_dropdown
                />

                // Menu
                <div class="absolute right-0 z-20 mt-2 w-40 bg-white dark:bg-gray-800 rounded-lg shadow-lg border border-gray-200 dark:border-gray-700 py-1">
                    {locales.iter().map(|&locale| {
                        let is_current = move || current_locale() == locale;
                        view! {
                            <button
                                on:click=select_locale(locale)
                                class=move || format!(
                                    "w-full px-4 py-2 text-left text-sm transition-colors {}",
                                    if is_current() {
                                        "bg-indigo-50 dark:bg-indigo-900/30 text-indigo-700 dark:text-indigo-300 font-medium"
                                    } else {
                                        "text-gray-700 dark:text-gray-300 hover:bg-gray-50 dark:hover:bg-gray-700"
                                    }
                                )
                            >
                                <span class="flex items-center gap-2">
                                    {if is_current() {
                                        view! {
                                            <svg class="w-4 h-4" fill="currentColor" viewBox="0 0 20 20">
                                                <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd"/>
                                            </svg>
                                        }.into_any()
                                    } else {
                                        view! { <span class="w-4"/> }.into_any()
                                    }}
                                    {locale_name(locale)}
                                </span>
                            </button>
                        }
                    }).collect_view()}
                </div>
            </Show>
        </div>
    }
}
