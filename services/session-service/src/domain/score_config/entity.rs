use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct ScoreConfig {
    pub session_id: Uuid,
    pub tier1_max_seconds: i32,
    pub tier2_max_seconds: i32,
    pub tier1_multiplier: f64,
    pub tier2_multiplier: f64,
    pub tier3_multiplier: f64,
    pub perfect_match_multiplier: f64,
    pub catchup_multiplier: f64,
    pub catchup_threshold_percent: i32,
    pub base_points: i32,
}

impl ScoreConfig {
    pub fn default_for_session(session_id: Uuid) -> Self {
        Self {
            session_id,
            tier1_max_seconds: 10,
            tier2_max_seconds: 20,
            tier1_multiplier: 3.0,
            tier2_multiplier: 2.0,
            tier3_multiplier: 1.0,
            perfect_match_multiplier: 2.0,
            catchup_multiplier: 1.5,
            catchup_threshold_percent: 50,
            base_points: 100,
        }
    }
}
