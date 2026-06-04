use std::collections::HashMap;
use std::sync::Arc;

use rand::seq::SliceRandom;
use rf_cache::RedisPubSub;
use rf_orm::DatabaseManager;
use serde::Deserialize;
use uuid::Uuid;

use crate::application::score_calculator;
use crate::domain::scoring::entity::{PlayerScore, RoundScore};
use crate::infrastructure::{
    engine_client::EngineClient,
    persistence::score_repository::ScoreRepository,
    session_client::SessionClient,
};

// ---------------------------------------------------------------------------
// Game event types received from engine-service
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
enum GameEvent {
    QuestionStarted {
        round_id: Uuid,
        question_id: Uuid,
        question_type: String,
        question_text: String,
        correct_answer: String,
        started_at: String,
        round_number: i32,
    },
    RoundClosed {
        round_id: Uuid,
        correct_answer: String,
        closed_at: String,
    },
    IchOderDuStarted {
        round_id: Uuid,
        ich_oder_du_text: String,
    },
    CoupleAnswered {
        round_id: Uuid,
        couple_answer: String,
    },
    GameEnded {
        session_code: String,
    },
}

// ---------------------------------------------------------------------------
// Events published to `wedding_quest:session:<code>`
// ---------------------------------------------------------------------------

#[derive(Debug, serde::Serialize)]
struct ScoresUpdatedEvent<'a> {
    #[serde(rename = "type")]
    event_type: &'a str,
    session_code: &'a str,
    scores: Vec<PlayerScoreEntry>,
}

#[derive(Debug, serde::Serialize)]
struct PlayerScoreEntry {
    player_id: Uuid,
    player_name: String,
    total_score: i32,
    last_round_score: i32,
    rank: usize,
}

#[derive(Debug, serde::Serialize)]
struct LuckyBoostEvent<'a> {
    #[serde(rename = "type")]
    event_type: &'a str,
    session_code: &'a str,
    player_id: Uuid,
    player_name: String,
    multiplier: f64,
}

// ---------------------------------------------------------------------------
// Main listener loop
// ---------------------------------------------------------------------------

pub async fn run(
    sub_pubsub: Arc<RedisPubSub>,
    db: Arc<DatabaseManager>,
    engine_client: Arc<EngineClient>,
    session_client: Arc<SessionClient>,
    pub_pubsub: Arc<RedisPubSub>,
) {
    let mut rx = match sub_pubsub.psubscribe("wedding_quest:game:*").await {
        Ok(rx) => rx,
        Err(e) => {
            tracing::error!("Failed to psubscribe to wedding_quest:game:*: {e}");
            return;
        }
    };

    tracing::info!("Event listener subscribed to wedding_quest:game:*");

    while let Some(msg) = rx.recv().await {
        let session_code = match msg.channel.strip_prefix("wedding_quest:game:") {
            Some(code) if !code.is_empty() => code.to_string(),
            _ => {
                tracing::warn!("Unexpected channel format: {}", msg.channel);
                continue;
            }
        };

        match serde_json::from_str::<GameEvent>(&msg.payload) {
            Ok(GameEvent::RoundClosed { round_id, correct_answer, .. }) => {
                tracing::debug!("RoundClosed for session={session_code} round={round_id}");
                if let Err(e) = handle_round_closed(
                    &session_code,
                    round_id,
                    &correct_answer,
                    &db,
                    &engine_client,
                    &session_client,
                    &pub_pubsub,
                )
                .await
                {
                    tracing::error!(
                        "Error handling RoundClosed for {session_code}/{round_id}: {e}"
                    );
                }
            }
            Ok(GameEvent::GameEnded { .. }) => {
                tracing::info!("GameEnded for session={session_code}");
            }
            Ok(_) => {
                tracing::debug!("Ignored event on channel {}", msg.channel);
            }
            Err(e) => {
                tracing::warn!(
                    "Failed to deserialise game event on {}: {e}  payload={}",
                    msg.channel,
                    msg.payload
                );
            }
        }
    }

    tracing::warn!("Event listener PubSub stream ended");
}

// ---------------------------------------------------------------------------
// Core scoring handler
// ---------------------------------------------------------------------------

async fn handle_round_closed(
    session_code: &str,
    round_id: Uuid,
    event_correct_answer: &str,
    db: &Arc<DatabaseManager>,
    engine_client: &Arc<EngineClient>,
    session_client: &Arc<SessionClient>,
    pub_pubsub: &Arc<RedisPubSub>,
) -> anyhow::Result<()> {
    let mut answers = engine_client
        .get_round_answers(session_code, round_id)
        .await?;

    if answers.is_empty() {
        tracing::info!("No answers for round {round_id}, skipping score calculation");
        return Ok(());
    }

    // For ich_oder_du: recalculate is_correct based on the event's correct_answer
    // (which is set by close_round based on couple agreement)
    let is_ich_oder_du = answers.first().map(|a| a.question_type == "ich_oder_du").unwrap_or(false);
    if is_ich_oder_du {
        for answer in &mut answers {
            if event_correct_answer.is_empty() {
                // Couple disagreed - nobody gets points
                answer.is_correct = false;
            } else {
                // Couple agreed - check if guest picked the same
                answer.is_correct = answer.answer.to_lowercase() == event_correct_answer.to_lowercase();
            }
        }
    }

    let config = session_client.get_score_config(session_code).await?;

    let repo = ScoreRepository::new(db.connection().clone());

    let existing = repo.find_by_session(session_code).await?;
    let existing_totals: HashMap<Uuid, i32> = existing
        .iter()
        .map(|ps| (ps.player_id, ps.total_score))
        .collect();

    // Collect pending Lucky Boost multipliers, then reset them to 1.0.
    let lucky_boosts: HashMap<Uuid, f64> = existing
        .iter()
        .filter(|ps| ps.lucky_boost_multiplier > 1.0)
        .map(|ps| (ps.player_id, ps.lucky_boost_multiplier))
        .collect();

    let results =
        score_calculator::calculate_round_scores(&answers, &config, &existing_totals, &lucky_boosts);

    for result in &results {
        let round_score = RoundScore::new(
            round_id,
            session_code.to_string(),
            result.player_id,
            result.player_name.clone(),
            result.base_points,
            result.time_multiplier,
            result.final_points,
            result.is_correct,
        );
        repo.insert_round_score(&round_score).await?;

        let mut player_score = repo
            .find_player(session_code, result.player_id)
            .await?
            .unwrap_or_else(|| {
                PlayerScore::new(
                    session_code.to_string(),
                    result.player_id,
                    result.player_name.clone(),
                )
            });

        player_score.apply_round_score(result.final_points);
        // Reset Lucky Boost after it was applied this round.
        if lucky_boosts.contains_key(&result.player_id) {
            player_score.lucky_boost_multiplier = 1.0;
        }
        repo.upsert_player_score(&player_score).await?;
    }

    // Reload leaderboard and publish ScoresUpdated.
    let leaderboard = repo.find_by_session(session_code).await?;

    // Assign ranks with ties: same score = same rank
    let score_entries: Vec<PlayerScoreEntry> = leaderboard
        .iter()
        .enumerate()
        .map(|(i, ps)| {
            let rank = if i == 0 {
                1
            } else if ps.total_score == leaderboard[i - 1].total_score {
                // Same score as previous → same rank
                // Find the rank of the first player with this score
                let mut r = i + 1;
                for j in (0..i).rev() {
                    if leaderboard[j].total_score == ps.total_score { r = j + 1; } else { break; }
                }
                r
            } else {
                i + 1
            };
            PlayerScoreEntry {
                player_id: ps.player_id,
                player_name: ps.player_name.clone(),
                total_score: ps.total_score,
                last_round_score: ps.last_round_score,
                rank,
            }
        })
        .collect();

    let event = ScoresUpdatedEvent {
        event_type: "ScoresUpdated",
        session_code,
        scores: score_entries,
    };

    let channel = format!("wedding_quest:session:{session_code}");
    pub_pubsub
        .publish(&channel, &serde_json::to_string(&event)?)
        .await?;

    tracing::info!(
        "ScoresUpdated published for session={session_code} round={round_id} players={}",
        leaderboard.len()
    );

    // Assign a Lucky Boost to the last-place GUEST player (if ≥ 2 players).
    // Skip Lucky Boost for ich_oder_du rounds and exclude couple members.
    // Couple members are identified by time_taken_seconds == 0 in their answers.
    let couple_player_ids: std::collections::HashSet<Uuid> = answers.iter()
        .filter(|a| a.time_taken_seconds == 0.0)
        .map(|a| a.player_id)
        .collect();

    // Find minimum couple score (to compare against guests)
    let min_couple_score = leaderboard.iter()
        .filter(|ps| couple_player_ids.contains(&ps.player_id))
        .map(|ps| ps.total_score)
        .min()
        .unwrap_or(i32::MAX);

    if !is_ich_oder_du && leaderboard.len() >= 2 {
        // Find the actual last-place player (lowest score).
        // Rules:
        // - Any player (guest or couple member) with the lowest score gets lucky boost
        // - All equal → no lucky boost
        let max_score = leaderboard.first().map(|ps| ps.total_score).unwrap_or(0);
        let min_score = leaderboard.last().map(|ps| ps.total_score).unwrap_or(0);

        // Only assign if exactly ONE player is in last place (alone)
        let last_place = if max_score > min_score {
            // Count how many players share the lowest score
            let last_count = leaderboard.iter().filter(|ps| ps.total_score == min_score).count();
            if last_count == 1 {
                leaderboard.last()
            } else {
                None // Multiple players tied at last place → no boost
            }
        } else {
            None
        };

        if let Some(last_place) = last_place {
                // Calculate dynamic multiplier:
                // Target: after boost, player lands in the middle but never exceeds leader
                let base_points = 100i32;
                let gap_to_leader = (max_score - last_place.total_score).max(0) as f64;

                // Target: land about 40-70% of the way to the leader (middle of pack)
                let target_extra = gap_to_leader * (0.4 + rand::random::<f64>() * 0.3);

                // Convert to multiplier: (base + extra) / base
                // e.g. gap=300, target_extra=180 → multiplier = (100+180)/100 = 2.8
                let capped_boost_points = target_extra.min(gap_to_leader - (base_points as f64 * 0.3));

                // Convert to multiplier (applied to base_points of next correct answer)
                let multiplier = if base_points > 0 && capped_boost_points > 0.0 {
                    ((capped_boost_points + base_points as f64) / base_points as f64).max(1.5).min(5.0)
                } else {
                    1.5
                };
                // Round to 1 decimal
                let multiplier = (multiplier * 10.0).round() / 10.0;

                if let Err(e) = repo
                    .set_lucky_boost(session_code, last_place.player_id, multiplier)
                    .await
                {
                    tracing::warn!("Failed to set Lucky Boost: {e}");
                } else {
                    let boost_event = LuckyBoostEvent {
                        event_type: "LuckyBoost",
                        session_code,
                        player_id: last_place.player_id,
                        player_name: last_place.player_name.clone(),
                        multiplier,
                    };
                    pub_pubsub
                        .publish(&channel, &serde_json::to_string(&boost_event)?)
                        .await?;

                    tracing::info!(
                        "LuckyBoost x{multiplier} assigned to {} in session={session_code}",
                        last_place.player_name
                    );
                }
        }
    }

    Ok(())
}
