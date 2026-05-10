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

    // First pass: raw time-multiplied score.
    let mut results: Vec<RoundScoreResult> = answers
        .iter()
        .map(|a| {
            let time_multiplier = if a.time_taken_seconds <= config.tier1_max_seconds as f64 {
                config.tier1_multiplier
            } else if a.time_taken_seconds <= config.tier2_max_seconds as f64 {
                config.tier2_multiplier
            } else {
                config.tier3_multiplier
            };

            let raw_score = if a.is_correct {
                (base_points as f64 * time_multiplier) as i32
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

    // Second pass: catch-up bonus.
    let max_total = existing_totals.values().copied().max().unwrap_or(0);
    if max_total > 0 {
        let threshold =
            (max_total as f64 * config.catchup_threshold_percent as f64 / 100.0) as i32;
        for result in &mut results {
            let player_total = existing_totals.get(&result.player_id).copied().unwrap_or(0);
            if player_total < threshold {
                result.final_points =
                    (result.final_points as f64 * config.catchup_multiplier) as i32;
            }
        }
    }

    // Third pass: Lucky Boost — applied only when the answer was correct.
    for result in &mut results {
        if result.is_correct {
            if let Some(&boost) = lucky_boosts.get(&result.player_id) {
                if boost > 1.0 {
                    result.final_points = (result.final_points as f64 * boost) as i32;
                }
            }
        }
    }

    results
}
