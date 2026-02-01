//! Big Five personality test (IPIP-NEO-120) library.
//!
//! This crate provides types and scoring logic for the Big Five personality test,
//! based on the IPIP-NEO-120 inventory.
//!
//! # Example
//!
//! ```
//! use bigfive::{Ipip120, Answer, calculate};
//!
//! // Load the English inventory
//! let inventory = Ipip120::english();
//!
//! // Collect answers (120 total, one per question)
//! let answers: Vec<Answer> = inventory
//!     .questions()
//!     .iter()
//!     .map(|q| Answer {
//!         question_id: q.id.clone(),
//!         value: 3, // Neutral answer for demo
//!     })
//!     .collect();
//!
//! // Calculate the personality profile
//! let profile = calculate(&inventory, &answers).unwrap();
//!
//! // Access domain scores
//! for domain_score in &profile.domains {
//!     println!("{}: {} ({:?})",
//!         domain_score.domain.name(),
//!         domain_score.raw,
//!         domain_score.level
//!     );
//! }
//! ```
//!
//! # Features
//!
//! - `serde` (default): Enables serialization/deserialization of types

mod inventory;
mod scoring;
mod types;

pub use inventory::Ipip120;
pub use scoring::calculate;
pub use types::{
    Answer, Domain, DomainScore, Facet, FacetScore, PersonalityProfile, Question, ScoreLevel,
};

use thiserror::Error;

/// Errors that can occur in the bigfive crate.
#[derive(Debug, Error)]
pub enum Error {
    /// Unsupported language requested.
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),

    /// Failed to parse question data.
    #[error("failed to parse question data: {0}")]
    ParseError(String),

    /// Invalid domain code in question data.
    #[error("invalid domain code: {0}")]
    InvalidDomain(String),

    /// Wrong number of questions loaded.
    #[error("expected 120 questions, got {0}")]
    InvalidQuestionCount(usize),

    /// Wrong number of answers provided.
    #[error("expected 120 answers, got {0}")]
    InvalidAnswerCount(usize),

    /// Answer value out of valid range (1-5).
    #[error("invalid answer value: {0} (must be 1-5)")]
    InvalidAnswerValue(u8),

    /// Missing answer for a question.
    #[error("missing answer for question: {0}")]
    MissingAnswer(String),

    /// Missing facet data during calculation.
    #[error("missing facet data for domain {0:?} facet {1}")]
    MissingFacetData(Domain, u8),

    /// Wrong number of questions for a facet.
    #[error("expected 4 questions for domain {0:?} facet {1}, got {2}")]
    InvalidFacetQuestionCount(Domain, u8, usize),
}
