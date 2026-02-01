//! IPIP-NEO-120 inventory loader.

use crate::Error;
use crate::types::{Domain, Question};
use serde::Deserialize;

/// Raw question format from the Alheimsins JSON data.
#[derive(Debug, Deserialize)]
struct RawQuestion {
    id: String,
    text: String,
    keyed: String,
    domain: String,
    facet: u8,
}

/// The IPIP-NEO-120 personality inventory.
///
/// Contains 120 questions measuring the Big Five personality traits,
/// with 24 questions per domain and 4 questions per facet.
#[derive(Debug, Clone, PartialEq)]
pub struct Ipip120 {
    questions: Vec<Question>,
    lang: String,
}

impl Ipip120 {
    /// Load the inventory for a specific language.
    ///
    /// Supported languages: "en" (English), "ru" (Russian)
    pub fn new(lang: &str) -> Result<Self, Error> {
        let json_data = match lang {
            "en" => include_str!("../data/en.json"),
            "ru" => include_str!("../data/ru.json"),
            _ => return Err(Error::UnsupportedLanguage(lang.to_string())),
        };

        let raw_questions: Vec<RawQuestion> =
            serde_json::from_str(json_data).map_err(|e| Error::ParseError(e.to_string()))?;

        let questions = raw_questions
            .into_iter()
            .map(|q| {
                let domain = Domain::from_code(&q.domain)
                    .ok_or_else(|| Error::InvalidDomain(q.domain.clone()))?;

                Ok(Question {
                    id: q.id,
                    text: q.text,
                    domain,
                    facet_index: q.facet,
                    reversed: q.keyed == "minus",
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        if questions.len() != 120 {
            return Err(Error::InvalidQuestionCount(questions.len()));
        }

        Ok(Self {
            questions,
            lang: lang.to_string(),
        })
    }

    /// Load the English inventory.
    pub fn english() -> Self {
        Self::new("en").expect("English inventory should always be valid")
    }

    /// Load the Russian inventory.
    pub fn russian() -> Self {
        Self::new("ru").expect("Russian inventory should always be valid")
    }

    /// Get all questions in the inventory.
    pub fn questions(&self) -> &[Question] {
        &self.questions
    }

    /// Get a question by its ID.
    pub fn question_by_id(&self, id: &str) -> Option<&Question> {
        self.questions.iter().find(|q| q.id == id)
    }

    /// Get the language of this inventory.
    pub fn lang(&self) -> &str {
        &self.lang
    }

    /// Get the number of questions.
    pub fn len(&self) -> usize {
        self.questions.len()
    }

    /// Check if the inventory is empty.
    pub fn is_empty(&self) -> bool {
        self.questions.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_load_english() {
        let inventory = Ipip120::english();
        assert_eq!(inventory.len(), 120);
        assert_eq!(inventory.lang(), "en");
    }

    #[test]
    fn test_load_russian() {
        let inventory = Ipip120::russian();
        assert_eq!(inventory.len(), 120);
        assert_eq!(inventory.lang(), "ru");
    }

    #[test]
    fn test_question_distribution() {
        let inventory = Ipip120::english();

        // Count questions per domain
        for domain in Domain::all() {
            let count = inventory
                .questions()
                .iter()
                .filter(|q| q.domain == *domain)
                .count();
            assert_eq!(count, 24, "Domain {:?} should have 24 questions", domain);
        }

        // Count questions per facet (should be 4 each)
        for domain in Domain::all() {
            for facet_idx in 1..=6 {
                let count = inventory
                    .questions()
                    .iter()
                    .filter(|q| q.domain == *domain && q.facet_index == facet_idx)
                    .count();
                assert_eq!(
                    count, 4,
                    "Domain {:?} facet {} should have 4 questions",
                    domain, facet_idx
                );
            }
        }
    }

    #[test]
    fn test_reversed_questions_exist() {
        let inventory = Ipip120::english();
        let reversed_count = inventory.questions().iter().filter(|q| q.reversed).count();
        assert!(
            reversed_count > 0,
            "There should be some reversed questions"
        );
        assert!(reversed_count < 120, "Not all questions should be reversed");
    }
}
