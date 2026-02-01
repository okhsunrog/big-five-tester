//! AI analysis module.
//!
//! Provides personality analysis using configurable AI models with optional
//! safeguard (prompt injection detection) and translation pipeline.

pub mod error;
pub mod pipeline;
pub mod prompts;
pub mod provider;

pub use error::AnalysisError;
pub use pipeline::generate_analysis;
