//! Big Five Personality Test web application.

#![recursion_limit = "2048"]
#![allow(clippy::module_inception)]

pub mod app;
pub mod components;
pub mod i18n;

#[cfg(feature = "ssr")]
pub mod ai;
#[cfg(feature = "ssr")]
pub mod config;
#[cfg(feature = "ssr")]
pub mod jobs;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::mount::hydrate_body(app::App);
}
