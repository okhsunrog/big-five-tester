//! Main application component and routing.

use leptos::prelude::*;
use leptos_i18n_router::I18nRoute;
use leptos_meta::{Link, Meta, MetaTags, Stylesheet, Title, provide_meta_context};
use leptos_router::{
    components::{Outlet, Route, Router, Routes},
    path,
};

use crate::components::{HomePage, ResultsPage, TestPage};
use crate::i18n::{I18nContextProvider, Locale};

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

        <I18nContextProvider>
            <Router>
                <Routes fallback=|| "Page not found.".into_view()>
                    <I18nRoute<Locale, _, _> view=|| {
                        view! {
                            <main class="min-h-screen bg-gray-50 dark:bg-gray-900 transition-colors duration-300">
                                <Outlet />
                            </main>
                        }
                    }>
                        <Route path=path!("") view=HomePage />
                        <Route path=path!("test") view=TestPage />
                        <Route path=path!("results") view=ResultsPage />
                    </I18nRoute<Locale, _, _>>
                </Routes>
            </Router>
        </I18nContextProvider>
    }
}
