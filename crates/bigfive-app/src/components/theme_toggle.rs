//! Theme toggle component for light/dark mode switching.

use leptos::prelude::*;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// Get system preferred color scheme
#[cfg(target_arch = "wasm32")]
fn get_system_theme() -> &'static str {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(media)) = window.match_media("(prefers-color-scheme: dark)") {
            if media.matches() {
                return "dark";
            }
        }
    }
    "light"
}

/// Apply theme to document
#[cfg(target_arch = "wasm32")]
fn apply_theme(theme: &str) {
    if let Some(window) = web_sys::window() {
        if let Some(document) = window.document() {
            if let Some(html) = document.document_element() {
                let _ = html.set_attribute("data-theme", theme);
            }
        }
    }
}

/// Save theme preference to localStorage (None = auto/system)
#[cfg(target_arch = "wasm32")]
fn save_theme_preference(theme: Option<&str>) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            match theme {
                Some(t) => {
                    let _ = storage.set_item("theme", t);
                }
                None => {
                    let _ = storage.remove_item("theme");
                }
            }
        }
    }
}

/// Load theme preference from localStorage (None = auto/system)
#[cfg(target_arch = "wasm32")]
fn load_theme_preference() -> Option<String> {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(theme)) = storage.get_item("theme") {
                return Some(theme);
            }
        }
    }
    None
}

/// Theme toggle component that cycles through auto/light/dark modes.
#[component]
pub fn ThemeToggle() -> impl IntoView {
    // "auto", "light", or "dark"
    let (mode, set_mode) = signal("auto".to_string());

    // Initialize theme on mount
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            let saved = load_theme_preference();
            let current_mode = saved.as_deref().unwrap_or("auto");
            set_mode.set(current_mode.to_string());

            let actual_theme = match current_mode {
                "light" => "light",
                "dark" => "dark",
                _ => get_system_theme(),
            };
            apply_theme(actual_theme);
        }
    });

    let cycle_theme = move |_| {
        // Cycle: auto -> light -> dark -> auto
        let new_mode = match mode.get().as_str() {
            "auto" => "light",
            "light" => "dark",
            "dark" => "auto",
            _ => "auto",
        };

        set_mode.set(new_mode.to_string());

        #[cfg(target_arch = "wasm32")]
        {
            let actual_theme = match new_mode {
                "light" => "light",
                "dark" => "dark",
                _ => get_system_theme(),
            };

            apply_theme(actual_theme);
            save_theme_preference(if new_mode == "auto" {
                None
            } else {
                Some(new_mode)
            });
        }
    };

    view! {
        <button
            class="theme-toggle"
            on:click=cycle_theme
            title=move || {
                match mode.get().as_str() {
                    "auto" => "Theme: auto (system)",
                    "light" => "Theme: light",
                    "dark" => "Theme: dark",
                    _ => "Theme",
                }
            }
        >
            {move || match mode.get().as_str() {
                "light" => "☀️",
                "dark" => "🌙",
                _ => "🌓", // auto
            }}
        </button>
    }
}
