use std::collections::HashMap;
use uuid::Uuid;

use crate::infrastructure::engine_client::PlayerAnswerDto;
use crate::infrastructure::session_client::ScoreConfigDto;

/// Result of scoring a single player's answer in a round.
#[derive(Debug, Clone)]
pub struct RoundScoreResult {
    pub player_id: Uuid,
    pub player_name: String,
    pub base_points: i32,
    pub time_multiplier: f64,
    pub final_points: i32,
    pub is_correct: bool,
}

/// Calculate per-player scores for a single closed round.
///
/// - `existing_totals` — current aggregate totals before this round (for catch-up detection)
/// - `lucky_boosts`    — pending Lucky Boost multipliers per player (applied on correct answers, then reset)
pub fn calculate_round_scores(
    answers: &[PlayerAnswerDto],
    config: &ScoreConfigDto,
    existing_totals: &HashMap<Uuid, i32>,
    lucky_boosts: &HashMap<Uuid, f64>,
) -> Vec<RoundScoreResult> {
    let base_points = config.base_points;

    // First pass: flat scoring (no time multiplier).
    let mut results: Vec<RoundScoreResult> = answers
        .iter()
        .map(|a| {
            let time_multiplier = 1.0;

            let raw_score = if a.is_correct {
                base_points
            } else {
                0
            };

            RoundScoreResult {
                player_id: a.player_id,
                player_name: a.player_name.clone(),
                base_points: if a.is_correct { base_points } else { 0 },
                time_multiplier,
                final_points: raw_score,
                is_correct: a.is_correct,
            }
        })
        .collect();

    // Check if this is an ich_oder_du round (all answers have same type)
    let is_ich_oder_du = answers.first().map(|a| a.question_type == "ich_oder_du").unwrap_or(false);

    // Lucky Boost (not for ich_oder_du).
    if !is_ich_oder_du {
        for result in &mut results {
            if result.is_correct {
                if let Some(&boost) = lucky_boosts.get(&result.player_id) {
                    if boost > 1.0 {
                        result.final_points = (result.final_points as f64 * boost) as i32;
                    }
                }
            }
        }
    }

    results
}
