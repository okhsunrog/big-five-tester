use leptos_i18n_build::{Config, TranslationsInfos};
use std::error::Error;
use std::path::PathBuf;
use std::process::Command;

fn main() -> Result<(), Box<dyn Error>> {
    println!("cargo::rerun-if-changed=build.rs");
    println!("cargo::rerun-if-changed=Cargo.toml");
    println!("cargo::rerun-if-changed=locales");

    // Git hash and build time
    let git_hash = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string());

    let build_time = chrono::Utc::now().format("%Y-%m-%d %H:%M UTC").to_string();

    println!("cargo:rustc-env=GIT_HASH={}", git_hash);
    println!("cargo:rustc-env=BUILD_TIME={}", build_time);
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads/");

    // Output directory for generated i18n module
    let i18n_mod_directory = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("i18n");

    // Configure locales: "en" is default, "ru" is additional
    let cfg = Config::new("en")?.add_locale("ru")?;

    // Parse translations
    let translations_infos = TranslationsInfos::parse(cfg)?;

    // Emit warnings/errors from parsing
    translations_infos.emit_diagnostics();

    // Rerun build if locale files change
    translations_infos.rerun_if_locales_changed();

    // Generate the i18n module
    translations_infos.generate_i18n_module(i18n_mod_directory)?;

    Ok(())
}
