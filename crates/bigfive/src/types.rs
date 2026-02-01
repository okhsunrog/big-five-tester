//! Core types for the Big Five personality test.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

/// The five personality domains in the Big Five model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Domain {
    /// Neuroticism (N) - tendency to experience negative emotions
    Neuroticism,
    /// Extraversion (E) - tendency to seek stimulation and enjoy company
    Extraversion,
    /// Openness to Experience (O) - tendency to be creative and curious
    Openness,
    /// Agreeableness (A) - tendency to be compassionate and cooperative
    Agreeableness,
    /// Conscientiousness (C) - tendency to be organized and dependable
    Conscientiousness,
}

impl Domain {
    /// Returns the single-letter code for the domain.
    pub fn code(&self) -> &'static str {
        match self {
            Domain::Neuroticism => "N",
            Domain::Extraversion => "E",
            Domain::Openness => "O",
            Domain::Agreeableness => "A",
            Domain::Conscientiousness => "C",
        }
    }

    /// Returns the full name of the domain.
    pub fn name(&self) -> &'static str {
        match self {
            Domain::Neuroticism => "Neuroticism",
            Domain::Extraversion => "Extraversion",
            Domain::Openness => "Openness to Experience",
            Domain::Agreeableness => "Agreeableness",
            Domain::Conscientiousness => "Conscientiousness",
        }
    }

    /// Returns the facets for this domain.
    pub fn facets(&self) -> &'static [Facet] {
        match self {
            Domain::Neuroticism => &[
                Facet::Anxiety,
                Facet::Anger,
                Facet::Depression,
                Facet::SelfConsciousness,
                Facet::Immoderation,
                Facet::Vulnerability,
            ],
            Domain::Extraversion => &[
                Facet::Friendliness,
                Facet::Gregariousness,
                Facet::Assertiveness,
                Facet::ActivityLevel,
                Facet::ExcitementSeeking,
                Facet::Cheerfulness,
            ],
            Domain::Openness => &[
                Facet::Imagination,
                Facet::ArtisticInterests,
                Facet::Emotionality,
                Facet::Adventurousness,
                Facet::Intellect,
                Facet::Liberalism,
            ],
            Domain::Agreeableness => &[
                Facet::Trust,
                Facet::Morality,
                Facet::Altruism,
                Facet::Cooperation,
                Facet::Modesty,
                Facet::Sympathy,
            ],
            Domain::Conscientiousness => &[
                Facet::SelfEfficacy,
                Facet::Orderliness,
                Facet::Dutifulness,
                Facet::AchievementStriving,
                Facet::SelfDiscipline,
                Facet::Cautiousness,
            ],
        }
    }

    /// Returns all domains.
    pub fn all() -> &'static [Domain] {
        &[
            Domain::Neuroticism,
            Domain::Extraversion,
            Domain::Openness,
            Domain::Agreeableness,
            Domain::Conscientiousness,
        ]
    }

    /// Parse from single-letter code.
    pub fn from_code(code: &str) -> Option<Domain> {
        match code {
            "N" => Some(Domain::Neuroticism),
            "E" => Some(Domain::Extraversion),
            "O" => Some(Domain::Openness),
            "A" => Some(Domain::Agreeableness),
            "C" => Some(Domain::Conscientiousness),
            _ => None,
        }
    }
}

/// The 30 facets in the IPIP-NEO model (6 per domain).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Facet {
    // Neuroticism facets (1-6)
    Anxiety,
    Anger,
    Depression,
    SelfConsciousness,
    Immoderation,
    Vulnerability,

    // Extraversion facets (1-6)
    Friendliness,
    Gregariousness,
    Assertiveness,
    ActivityLevel,
    ExcitementSeeking,
    Cheerfulness,

    // Openness facets (1-6)
    Imagination,
    ArtisticInterests,
    Emotionality,
    Adventurousness,
    Intellect,
    Liberalism,

    // Agreeableness facets (1-6)
    Trust,
    Morality,
    Altruism,
    Cooperation,
    Modesty,
    Sympathy,

    // Conscientiousness facets (1-6)
    SelfEfficacy,
    Orderliness,
    Dutifulness,
    AchievementStriving,
    SelfDiscipline,
    Cautiousness,
}

impl Facet {
    /// Returns the name of the facet.
    pub fn name(&self) -> &'static str {
        match self {
            // Neuroticism
            Facet::Anxiety => "Anxiety",
            Facet::Anger => "Anger",
            Facet::Depression => "Depression",
            Facet::SelfConsciousness => "Self-Consciousness",
            Facet::Immoderation => "Immoderation",
            Facet::Vulnerability => "Vulnerability",
            // Extraversion
            Facet::Friendliness => "Friendliness",
            Facet::Gregariousness => "Gregariousness",
            Facet::Assertiveness => "Assertiveness",
            Facet::ActivityLevel => "Activity Level",
            Facet::ExcitementSeeking => "Excitement-Seeking",
            Facet::Cheerfulness => "Cheerfulness",
            // Openness
            Facet::Imagination => "Imagination",
            Facet::ArtisticInterests => "Artistic Interests",
            Facet::Emotionality => "Emotionality",
            Facet::Adventurousness => "Adventurousness",
            Facet::Intellect => "Intellect",
            Facet::Liberalism => "Liberalism",
            // Agreeableness
            Facet::Trust => "Trust",
            Facet::Morality => "Morality",
            Facet::Altruism => "Altruism",
            Facet::Cooperation => "Cooperation",
            Facet::Modesty => "Modesty",
            Facet::Sympathy => "Sympathy",
            // Conscientiousness
            Facet::SelfEfficacy => "Self-Efficacy",
            Facet::Orderliness => "Orderliness",
            Facet::Dutifulness => "Dutifulness",
            Facet::AchievementStriving => "Achievement-Striving",
            Facet::SelfDiscipline => "Self-Discipline",
            Facet::Cautiousness => "Cautiousness",
        }
    }

    /// Returns the domain this facet belongs to.
    pub fn domain(&self) -> Domain {
        match self {
            Facet::Anxiety
            | Facet::Anger
            | Facet::Depression
            | Facet::SelfConsciousness
            | Facet::Immoderation
            | Facet::Vulnerability => Domain::Neuroticism,

            Facet::Friendliness
            | Facet::Gregariousness
            | Facet::Assertiveness
            | Facet::ActivityLevel
            | Facet::ExcitementSeeking
            | Facet::Cheerfulness => Domain::Extraversion,

            Facet::Imagination
            | Facet::ArtisticInterests
            | Facet::Emotionality
            | Facet::Adventurousness
            | Facet::Intellect
            | Facet::Liberalism => Domain::Openness,

            Facet::Trust
            | Facet::Morality
            | Facet::Altruism
            | Facet::Cooperation
            | Facet::Modesty
            | Facet::Sympathy => Domain::Agreeableness,

            Facet::SelfEfficacy
            | Facet::Orderliness
            | Facet::Dutifulness
            | Facet::AchievementStriving
            | Facet::SelfDiscipline
            | Facet::Cautiousness => Domain::Conscientiousness,
        }
    }

    /// Returns the facet index (1-6) within its domain.
    pub fn index(&self) -> u8 {
        match self {
            // Neuroticism
            Facet::Anxiety => 1,
            Facet::Anger => 2,
            Facet::Depression => 3,
            Facet::SelfConsciousness => 4,
            Facet::Immoderation => 5,
            Facet::Vulnerability => 6,
            // Extraversion
            Facet::Friendliness => 1,
            Facet::Gregariousness => 2,
            Facet::Assertiveness => 3,
            Facet::ActivityLevel => 4,
            Facet::ExcitementSeeking => 5,
            Facet::Cheerfulness => 6,
            // Openness
            Facet::Imagination => 1,
            Facet::ArtisticInterests => 2,
            Facet::Emotionality => 3,
            Facet::Adventurousness => 4,
            Facet::Intellect => 5,
            Facet::Liberalism => 6,
            // Agreeableness
            Facet::Trust => 1,
            Facet::Morality => 2,
            Facet::Altruism => 3,
            Facet::Cooperation => 4,
            Facet::Modesty => 5,
            Facet::Sympathy => 6,
            // Conscientiousness
            Facet::SelfEfficacy => 1,
            Facet::Orderliness => 2,
            Facet::Dutifulness => 3,
            Facet::AchievementStriving => 4,
            Facet::SelfDiscipline => 5,
            Facet::Cautiousness => 6,
        }
    }

    /// Get facet from domain and index (1-6).
    pub fn from_domain_and_index(domain: Domain, index: u8) -> Option<Facet> {
        if !(1..=6).contains(&index) {
            return None;
        }
        Some(domain.facets()[(index - 1) as usize])
    }
}

/// Score level categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum ScoreLevel {
    /// Low score (roughly bottom third)
    Low,
    /// Neutral/average score (roughly middle third)
    Neutral,
    /// High score (roughly top third)
    High,
}

/// A single question in the inventory.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Question {
    /// Unique identifier for the question.
    pub id: String,
    /// The question text.
    pub text: String,
    /// The domain this question measures.
    pub domain: Domain,
    /// The facet index (1-6) within the domain.
    pub facet_index: u8,
    /// Whether this question uses reverse scoring.
    pub reversed: bool,
}

impl Question {
    /// Get the facet this question measures.
    pub fn facet(&self) -> Option<Facet> {
        Facet::from_domain_and_index(self.domain, self.facet_index)
    }
}

/// An answer to a question.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct Answer {
    /// The question ID this answer is for.
    pub question_id: String,
    /// The response value (1-5).
    /// 1 = Very Inaccurate, 2 = Moderately Inaccurate, 3 = Neither,
    /// 4 = Moderately Accurate, 5 = Very Accurate
    pub value: u8,
}

/// Score for a single facet.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct FacetScore {
    /// The facet being scored.
    pub facet: Facet,
    /// Raw score (4-20 for IPIP-NEO-120, 4 questions per facet).
    pub raw: u8,
    /// Categorized level.
    pub level: ScoreLevel,
}

impl FacetScore {
    /// Calculate percentage (0-100) based on min=4, max=20.
    pub fn percentage(&self) -> f32 {
        ((self.raw as f32 - 4.0) / 16.0) * 100.0
    }
}

/// Score for a domain.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DomainScore {
    /// The domain being scored.
    pub domain: Domain,
    /// Raw score (24-120 for IPIP-NEO-120, sum of 6 facets).
    pub raw: u8,
    /// Categorized level.
    pub level: ScoreLevel,
    /// Individual facet scores.
    pub facets: Vec<FacetScore>,
}

impl DomainScore {
    /// Calculate percentage (0-100) based on min=24, max=120.
    pub fn percentage(&self) -> f32 {
        ((self.raw as f32 - 24.0) / 96.0) * 100.0
    }
}

/// Complete personality profile with all domain and facet scores.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PersonalityProfile {
    /// Scores for all five domains.
    pub domains: Vec<DomainScore>,
}

impl PersonalityProfile {
    /// Get score for a specific domain.
    pub fn domain_score(&self, domain: Domain) -> Option<&DomainScore> {
        self.domains.iter().find(|d| d.domain == domain)
    }

    /// Get score for a specific facet.
    pub fn facet_score(&self, facet: Facet) -> Option<&FacetScore> {
        let domain = facet.domain();
        self.domain_score(domain)
            .and_then(|d| d.facets.iter().find(|f| f.facet == facet))
    }
}
