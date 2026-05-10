use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Envelope for all outgoing WebSocket messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WsMessage {
    #[serde(rename = "type")]
    pub event_type: String,
    pub payload: serde_json::Value,
}

impl WsMessage {
    pub fn new(event_type: impl Into<String>, payload: serde_json::Value) -> Self {
        Self {
            event_type: event_type.into(),
            payload,
        }
    }
}

/// All events the server broadcasts to clients in a room
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ServerEvent {
    /// Fired when a new player joins the lobby
    PlayerJoined {
        player_id: Uuid,
        display_name: String,
    },
    /// Fired when a player disconnects
    PlayerDisconnected {
        player_id: Uuid,
    },
    /// Fired when the host starts the game
    SessionStarted {
        session_id: Uuid,
    },
    /// Fired when the session ends
    SessionEnded {
        session_id: Uuid,
    },
    /// Generic game state change (used by engine-service events)
    GameStateChanged {
        payload: serde_json::Value,
    },
    /// Sent immediately after a client connects to confirm room membership
    Connected {
        session_code: String,
    },
    /// Periodic ping to keep the connection alive
    Ping,
}

impl ServerEvent {
    pub fn to_ws_text(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| r#"{"type":"ERROR"}"#.into())
    }
}

/// Events the client can send to the server (minimal for realtime-service)
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ClientEvent {
    Pong,
}
