use rand::{distributions::Alphanumeric, Rng};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GameCode(String);

impl GameCode {
    pub fn generate() -> Self {
        let code: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(6)
            .map(char::from)
            .collect::<String>()
            .to_uppercase();
        Self(code)
    }

    pub fn from_string(s: String) -> Result<Self, String> {
        let s = s.to_uppercase();
        if s.len() == 6 && s.chars().all(|c| c.is_ascii_alphanumeric()) {
            Ok(Self(s))
        } else {
            Err(format!("Invalid game code: {s}"))
        }
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for GameCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Lobby,
    Active,
    Paused,
    Ended,
}

impl SessionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            SessionStatus::Lobby => "lobby",
            SessionStatus::Active => "active",
            SessionStatus::Paused => "paused",
            SessionStatus::Ended => "ended",
        }
    }
}

impl TryFrom<String> for SessionStatus {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.as_str() {
            "lobby" => Ok(SessionStatus::Lobby),
            "active" => Ok(SessionStatus::Active),
            "paused" => Ok(SessionStatus::Paused),
            "ended" => Ok(SessionStatus::Ended),
            other => Err(format!("Unknown session status: {other}")),
        }
    }
}
