//! Main application component and routing.

use leptos::prelude::*;
use leptos_meta::{Link, Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Outlet, ParentRoute, Route, Router, Routes},
    path,
};

use crate::components::{HomePage, ResultsPage, TestPage};
use crate::i18n::I18nProvider;

/// Shell function for SSR.
pub fn shell(options: LeptosOptions) -> impl IntoView {
    // Inline script to apply theme before render (prevents flash)
    let theme_script = r#"
        (function() {
            var theme = localStorage.getItem('theme');
            if (theme === 'dark' || (!theme && window.matchMedia('(prefers-color-scheme: dark)').matches)) {
                document.documentElement.setAttribute('data-theme', 'dark');
            } else {
                document.documentElement.setAttribute('data-theme', 'light');
            }
        })();
    "#;

    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="utf-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1" />
                <script inner_html=theme_script></script>
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

/// Layout wrapper that provides i18n context.
#[component]
fn LocaleLayout() -> impl IntoView {
    view! {
        <I18nProvider>
            <main class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300">
                <Outlet />
            </main>
        </I18nProvider>
    }
}

/// Main application component.
#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/bigfive-app.css" />
        <Link rel="icon" type_="image/x-icon" href="/favicon.ico" />
        <Meta
            name="description"
            content="Take the Big Five personality test (IPIP-NEO-120) and get AI-powered insights about your personality."
        />

        <Title text="Big Five Personality Test" />

        <Router>
            <Routes fallback=|| "Page not found.".into_view()>
                // English routes
                <ParentRoute path=path!("/en") view=LocaleLayout>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("test") view=TestPage />
                    <Route path=path!("results") view=ResultsPage />
                    <Route path=path!("results/:id") view=ResultsPage />
                </ParentRoute>

                // Russian routes
                <ParentRoute path=path!("/ru") view=LocaleLayout>
                    <Route path=path!("") view=HomePage />
                    <Route path=path!("test") view=TestPage />
                    <Route path=path!("results") view=ResultsPage />
                    <Route path=path!("results/:id") view=ResultsPage />
                </ParentRoute>

                // Root redirect to /en
                <Route
                    path=path!("")
                    view=|| {
                        view! { <RedirectToLocale /> }
                    }
                />
            </Routes>
        </Router>
    }
}

/// Redirect to default locale.
#[component]
fn RedirectToLocale() -> impl IntoView {
    Effect::new(|_| {
        #[cfg(target_arch = "wasm32")]
        {
            if let Some(window) = web_sys::window() {
                let _ = window.location().set_pathname("/en");
            }
        }
    });

    view! { <div>"Redirecting..."</div> }
}
