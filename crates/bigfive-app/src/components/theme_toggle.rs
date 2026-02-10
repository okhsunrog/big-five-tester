//! Theme toggle component for light/dark mode switching.

use leptos::prelude::*;

/// Get system preferred color scheme
#[cfg(target_arch = "wasm32")]
fn get_system_theme() -> &'static str {
    if let Some(window) = web_sys::window()
        && let Ok(Some(media)) = window.match_media("(prefers-color-scheme: dark)")
        && media.matches()
    {
        return "dark";
    }
    "light"
}

/// Apply theme to document
#[cfg(target_arch = "wasm32")]
fn apply_theme(theme: &str) {
    if let Some(window) = web_sys::window()
        && let Some(document) = window.document()
        && let Some(html) = document.document_element()
    {
        let _ = html.set_attribute("data-theme", theme);
    }
}

/// Save theme preference to localStorage
#[cfg(target_arch = "wasm32")]
fn save_theme_preference(theme: &str) {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
    {
        let _ = storage.set_item("theme", theme);
    }
}

/// Load theme preference from localStorage
#[cfg(target_arch = "wasm32")]
fn load_theme_preference() -> Option<String> {
    if let Some(window) = web_sys::window()
        && let Ok(Some(storage)) = window.local_storage()
        && let Ok(Some(theme)) = storage.get_item("theme")
    {
        return Some(theme);
    }
    None
}

/// Theme toggle component that switches between light and dark modes.
/// Uses the system color scheme as default when no preference is saved.
#[component]
pub fn ThemeToggle() -> AnyView {
    let (mode, set_mode) = signal("light".to_string());

    // Initialize theme on mount
    Effect::new(move |_| {
        #[cfg(target_arch = "wasm32")]
        {
            let current_mode = load_theme_preference()
                .unwrap_or_else(|| get_system_theme().to_string());
            set_mode.set(current_mode.clone());
            apply_theme(&current_mode);
        }
    });

    let toggle_theme = move |_| {
        let new_mode = match mode.get().as_str() {
            "light" => "dark",
            _ => "light",
        };

        set_mode.set(new_mode.to_string());

        #[cfg(target_arch = "wasm32")]
        {
            apply_theme(new_mode);
            save_theme_preference(new_mode);
        }
    };

    view! {
        <button
            class="theme-toggle"
            on:click=toggle_theme
            title=move || {
                match mode.get().as_str() {
                    "light" => "Theme: light",
                    _ => "Theme: dark",
                }
            }
        >
            {move || match mode.get().as_str() {
                "light" => "☀️",
                _ => "🌙",
            }}
        </button>
    }
    .into_any()
}
