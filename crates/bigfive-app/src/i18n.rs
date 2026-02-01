//! Internationalization support using leptos_i18n.

// Include the generated i18n module from build.rs output
include!(concat!(env!("OUT_DIR"), "/i18n/mod.rs"));

// Re-export commonly used items
pub use i18n::*;
