use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PlayerName(String);

impl PlayerName {
    pub fn new(name: String) -> Result<Self, String> {
        let name = name.trim().to_string();
        if name.is_empty() {
            return Err("Player name cannot be empty".into());
        }
        if name.len() > 30 {
            return Err("Player name too long (max 30 chars)".into());
        }
        Ok(Self(name))
    }

    pub fn value(&self) -> &str {
        &self.0
    }
}
