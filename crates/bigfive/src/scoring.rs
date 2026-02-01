//! Scoring logic for the Big Five personality test.

use std::collections::HashMap;

use crate::Error;
use crate::inventory::Ipip120;
#[cfg(test)]
use crate::types::Facet;
use crate::types::{Answer, Domain, DomainScore, FacetScore, PersonalityProfile, ScoreLevel};

/// Calculate the personality profile from answers.
///
/// # Arguments
/// * `inventory` - The question inventory used
/// * `answers` - Vector of answers (must have exactly 120 answers)
///
/// # Returns
/// A `PersonalityProfile` with scores for all domains and facets.
pub fn calculate(inventory: &Ipip120, answers: &[Answer]) -> Result<PersonalityProfile, Error> {
    if answers.len() != 120 {
        return Err(Error::InvalidAnswerCount(answers.len()));
    }

    // Create a map of question_id -> answer for quick lookup
    let answer_map: HashMap<&str, u8> = answers
        .iter()
        .map(|a| (a.question_id.as_str(), a.value))
        .collect();

    // Validate all answers have values 1-5
    for answer in answers {
        if answer.value < 1 || answer.value > 5 {
            return Err(Error::InvalidAnswerValue(answer.value));
        }
    }

    // Calculate scores for each facet
    // Key: (Domain, facet_index) -> Vec<scores>
    let mut facet_scores_raw: HashMap<(Domain, u8), Vec<u8>> = HashMap::new();

    for question in inventory.questions() {
        let answer_value = answer_map
            .get(question.id.as_str())
            .ok_or_else(|| Error::MissingAnswer(question.id.clone()))?;

        // Apply reverse scoring if needed
        let score = if question.reversed {
            6 - answer_value // 1->5, 2->4, 3->3, 4->2, 5->1
        } else {
            *answer_value
        };

        facet_scores_raw
            .entry((question.domain, question.facet_index))
            .or_default()
            .push(score);
    }

    // Build domain scores
    let mut domains = Vec::new();

    for domain in Domain::all() {
        let mut facets = Vec::new();
        let mut domain_total: u16 = 0;

        for facet in domain.facets() {
            let facet_index = facet.index();
            let scores = facet_scores_raw
                .get(&(*domain, facet_index))
                .ok_or(Error::MissingFacetData(*domain, facet_index))?;

            if scores.len() != 4 {
                return Err(Error::InvalidFacetQuestionCount(
                    *domain,
                    facet_index,
                    scores.len(),
                ));
            }

            let raw: u8 = scores.iter().map(|&s| s as u16).sum::<u16>() as u8;
            domain_total += raw as u16;

            facets.push(FacetScore {
                facet: *facet,
                raw,
                level: facet_level(raw),
            });
        }

        domains.push(DomainScore {
            domain: *domain,
            raw: domain_total as u8,
            level: domain_level(domain_total as u8),
            facets,
        });
    }

    Ok(PersonalityProfile { domains })
}

/// Determine the level for a facet score (range 4-20).
fn facet_level(raw: u8) -> ScoreLevel {
    // Divide into roughly thirds:
    // Low: 4-9 (6 values)
    // Neutral: 10-14 (5 values)
    // High: 15-20 (6 values)
    match raw {
        4..=9 => ScoreLevel::Low,
        10..=14 => ScoreLevel::Neutral,
        15..=20 => ScoreLevel::High,
        _ => ScoreLevel::Neutral, // Should not happen with valid data
    }
}

/// Determine the level for a domain score (range 24-120).
fn domain_level(raw: u8) -> ScoreLevel {
    // Divide into roughly thirds:
    // Range is 24-120, span is 96
    // Low: 24-55 (32 values)
    // Neutral: 56-87 (32 values)
    // High: 88-120 (33 values)
    match raw {
        24..=55 => ScoreLevel::Low,
        56..=87 => ScoreLevel::Neutral,
        88..=120 => ScoreLevel::High,
        _ => ScoreLevel::Neutral, // Should not happen with valid data
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inventory::Ipip120;

    fn create_uniform_answers(inventory: &Ipip120, value: u8) -> Vec<Answer> {
        inventory
            .questions()
            .iter()
            .map(|q| Answer {
                question_id: q.id.clone(),
                value,
            })
            .collect()
    }

    #[test]
    fn test_all_ones_scoring() {
        let inventory = Ipip120::english();
        let answers = create_uniform_answers(&inventory, 1);
        let profile = calculate(&inventory, &answers).unwrap();

        // With all 1s, reversed questions score 5, normal questions score 1
        // This should give varied scores
        assert_eq!(profile.domains.len(), 5);

        for domain_score in &profile.domains {
            assert_eq!(domain_score.facets.len(), 6);
        }
    }

    #[test]
    fn test_all_threes_scoring() {
        let inventory = Ipip120::english();
        let answers = create_uniform_answers(&inventory, 3);
        let profile = calculate(&inventory, &answers).unwrap();

        // With all 3s (neutral), reversed or not, score is always 3
        // Facet score: 4 * 3 = 12 (Neutral)
        // Domain score: 6 * 12 = 72 (Neutral)
        for domain_score in &profile.domains {
            assert_eq!(domain_score.raw, 72);
            assert_eq!(domain_score.level, ScoreLevel::Neutral);

            for facet_score in &domain_score.facets {
                assert_eq!(facet_score.raw, 12);
                assert_eq!(facet_score.level, ScoreLevel::Neutral);
            }
        }
    }

    #[test]
    fn test_all_fives_scoring() {
        let inventory = Ipip120::english();
        let answers = create_uniform_answers(&inventory, 5);
        let profile = calculate(&inventory, &answers).unwrap();

        // With all 5s, reversed questions score 1, normal questions score 5
        assert_eq!(profile.domains.len(), 5);
    }

    #[test]
    fn test_reverse_scoring() {
        let inventory = Ipip120::english();

        // Find a reversed question
        let reversed_q = inventory
            .questions()
            .iter()
            .find(|q| q.reversed)
            .expect("Should have reversed questions");

        // Find a non-reversed question in the same facet
        let normal_q = inventory.questions().iter().find(|q| {
            !q.reversed && q.domain == reversed_q.domain && q.facet_index == reversed_q.facet_index
        });

        if let Some(normal_q) = normal_q {
            // Create answers where reversed = 1, normal = 5
            // Both should contribute 5 to the score
            let mut answers: Vec<Answer> = inventory
                .questions()
                .iter()
                .map(|q| Answer {
                    question_id: q.id.clone(),
                    value: 3, // Default neutral
                })
                .collect();

            // Set specific answers
            for answer in &mut answers {
                if answer.question_id == reversed_q.id {
                    answer.value = 1; // Will become 5 after reverse
                }
                if answer.question_id == normal_q.id {
                    answer.value = 5; // Stays 5
                }
            }

            let profile = calculate(&inventory, &answers).unwrap();

            // Both questions should have contributed 5 to their facet
            let facet =
                Facet::from_domain_and_index(reversed_q.domain, reversed_q.facet_index).unwrap();
            let facet_score = profile.facet_score(facet).unwrap();

            // 2 questions with score 5 + 2 questions with score 3 = 16
            assert_eq!(facet_score.raw, 16);
        }
    }

    #[test]
    fn test_invalid_answer_count() {
        let inventory = Ipip120::english();
        let answers = vec![Answer {
            question_id: "test".to_string(),
            value: 3,
        }];

        let result = calculate(&inventory, &answers);
        assert!(matches!(result, Err(Error::InvalidAnswerCount(1))));
    }

    #[test]
    fn test_invalid_answer_value() {
        let inventory = Ipip120::english();
        let mut answers = create_uniform_answers(&inventory, 3);
        answers[0].value = 6; // Invalid

        let result = calculate(&inventory, &answers);
        assert!(matches!(result, Err(Error::InvalidAnswerValue(6))));
    }

    #[test]
    fn test_percentage_calculations() {
        let facet_score = FacetScore {
            facet: Facet::Anxiety,
            raw: 12,
            level: ScoreLevel::Neutral,
        };
        assert!((facet_score.percentage() - 50.0).abs() < 0.01);

        let facet_min = FacetScore {
            facet: Facet::Anxiety,
            raw: 4,
            level: ScoreLevel::Low,
        };
        assert!((facet_min.percentage() - 0.0).abs() < 0.01);

        let facet_max = FacetScore {
            facet: Facet::Anxiety,
            raw: 20,
            level: ScoreLevel::High,
        };
        assert!((facet_max.percentage() - 100.0).abs() < 0.01);
    }
}
