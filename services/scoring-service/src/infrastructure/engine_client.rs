use uuid::Uuid;

/// HTTP client for the engine-service.
pub struct EngineClient {
    base_url: String,
    client: reqwest::Client,
}

impl EngineClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// GET /games/:code/rounds/:round_id/answers
    pub async fn get_round_answers(
        &self,
        session_code: &str,
        round_id: Uuid,
    ) -> anyhow::Result<Vec<PlayerAnswerDto>> {
        let url = format!(
            "{}/games/{}/rounds/{}/answers",
            self.base_url, session_code, round_id
        );
        let resp = self.client.get(&url).send().await?;
        if !resp.status().is_success() {
            anyhow::bail!(
                "engine-service returned {} for round answers",
                resp.status()
            );
        }
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct PlayerAnswerDto {
    pub player_id: Uuid,
    pub player_name: String,
    /// The raw answer string — kept for future audit/replay use.
    #[allow(dead_code)]
    pub answer: String,
    pub is_correct: bool,
    pub time_taken_seconds: f64,
}
