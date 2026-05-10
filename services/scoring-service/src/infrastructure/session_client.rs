/// HTTP client for the session-service.
pub struct SessionClient {
    base_url: String,
    client: reqwest::Client,
}

impl SessionClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: reqwest::Client::new(),
        }
    }

    /// GET /sessions/:code/config
    ///
    /// Returns score config for the session.  Falls back to defaults if the
    /// session-service returns 404 (config not yet set) or any other error.
    pub async fn get_score_config(&self, session_code: &str) -> anyhow::Result<ScoreConfigDto> {
        let url = format!("{}/sessions/{}/config", self.base_url, session_code);
        let resp = self.client.get(&url).send().await?;
        if resp.status() == reqwest::StatusCode::NOT_FOUND {
            return Ok(ScoreConfigDto::default());
        }
        if !resp.status().is_success() {
            tracing::warn!(
                "session-service returned {} for score config, using defaults",
                resp.status()
            );
            return Ok(ScoreConfigDto::default());
        }
        Ok(resp.json().await?)
    }
}

#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct ScoreConfigDto {
    #[serde(default = "default_tier1")]
    pub tier1_max_seconds: i32,
    #[serde(default = "default_tier2")]
    pub tier2_max_seconds: i32,
    #[serde(default = "default_t1m")]
    pub tier1_multiplier: f64,
    #[serde(default = "default_t2m")]
    pub tier2_multiplier: f64,
    #[serde(default = "default_t3m")]
    pub tier3_multiplier: f64,
    #[serde(default = "default_pm")]
    pub perfect_match_multiplier: f64,
    #[serde(default = "default_cu")]
    pub catchup_multiplier: f64,
    #[serde(default = "default_ct")]
    pub catchup_threshold_percent: i32,
    #[serde(default = "default_bp")]
    pub base_points: i32,
}

impl Default for ScoreConfigDto {
    fn default() -> Self {
        Self {
            tier1_max_seconds: default_tier1(),
            tier2_max_seconds: default_tier2(),
            tier1_multiplier: default_t1m(),
            tier2_multiplier: default_t2m(),
            tier3_multiplier: default_t3m(),
            perfect_match_multiplier: default_pm(),
            catchup_multiplier: default_cu(),
            catchup_threshold_percent: default_ct(),
            base_points: default_bp(),
        }
    }
}

fn default_tier1() -> i32 { 10 }
fn default_tier2() -> i32 { 20 }
fn default_t1m() -> f64 { 3.0 }
fn default_t2m() -> f64 { 2.0 }
fn default_t3m() -> f64 { 1.0 }
fn default_pm() -> f64 { 2.0 }
fn default_cu() -> f64 { 1.5 }
fn default_ct() -> i32 { 50 }
fn default_bp() -> i32 { 100 }
