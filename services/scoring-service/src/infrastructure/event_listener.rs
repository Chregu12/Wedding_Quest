use std::collections::HashMap;
use std::sync::Arc;

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
// ScoresUpdated event published to `wedding_quest:session:<code>`
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
        // Extract session_code from channel: "wedding_quest:game:ABC123"
        let session_code = match msg.channel.strip_prefix("wedding_quest:game:") {
            Some(code) if !code.is_empty() => code.to_string(),
            _ => {
                tracing::warn!("Unexpected channel format: {}", msg.channel);
                continue;
            }
        };

        match serde_json::from_str::<GameEvent>(&msg.payload) {
            Ok(GameEvent::RoundClosed { round_id, .. }) => {
                tracing::debug!("RoundClosed for session={session_code} round={round_id}");
                if let Err(e) = handle_round_closed(
                    &session_code,
                    round_id,
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
                // QuestionStarted, IchOderDuStarted, CoupleAnswered — ignored for now
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
    db: &Arc<DatabaseManager>,
    engine_client: &Arc<EngineClient>,
    session_client: &Arc<SessionClient>,
    pub_pubsub: &Arc<RedisPubSub>,
) -> anyhow::Result<()> {
    // 1. Fetch player answers from engine-service.
    let answers = engine_client
        .get_round_answers(session_code, round_id)
        .await?;

    if answers.is_empty() {
        tracing::info!("No answers for round {round_id}, skipping score calculation");
        return Ok(());
    }

    // 2. Fetch score config from session-service (defaults if unavailable).
    let config = session_client.get_score_config(session_code).await?;

    let repo = ScoreRepository::new(db.connection().clone());

    // 3. Load current aggregate totals for catch-up calculation.
    let existing = repo.find_by_session(session_code).await?;
    let existing_totals: HashMap<Uuid, i32> = existing
        .iter()
        .map(|ps| (ps.player_id, ps.total_score))
        .collect();

    // 4. Calculate scores.
    let results = score_calculator::calculate_round_scores(&answers, &config, &existing_totals);

    // 5. Persist round scores and update aggregate player scores.
    for result in &results {
        // Insert round_score row.
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

        // Load or create the player aggregate.
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
        repo.upsert_player_score(&player_score).await?;
    }

    // 6. Reload leaderboard and publish ScoresUpdated.
    let leaderboard = repo.find_by_session(session_code).await?;

    let score_entries: Vec<PlayerScoreEntry> = leaderboard
        .iter()
        .enumerate()
        .map(|(i, ps)| PlayerScoreEntry {
            player_id: ps.player_id,
            player_name: ps.player_name.clone(),
            total_score: ps.total_score,
            last_round_score: ps.last_round_score,
            rank: i + 1,
        })
        .collect();

    let event = ScoresUpdatedEvent {
        event_type: "ScoresUpdated",
        session_code,
        scores: score_entries,
    };

    let payload = serde_json::to_string(&event)?;
    let channel = format!("wedding_quest:session:{session_code}");
    pub_pubsub.publish(&channel, &payload).await?;

    tracing::info!(
        "ScoresUpdated published for session={session_code} round={round_id} players={}",
        leaderboard.len()
    );

    Ok(())
}
