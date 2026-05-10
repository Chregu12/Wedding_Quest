use crate::domain::room::events::{ClientEvent, ServerEvent};
use rf_broadcast::RoomRegistry;
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        Path,
    },
    Extension,
    response::IntoResponse,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::sync::Arc;
use tokio::sync::mpsc;
use uuid::Uuid;

pub async fn ws_upgrade(
    ws: WebSocketUpgrade,
    Path(session_id): Path<Uuid>,
    Extension(registry): Extension<Arc<RoomRegistry>>,
) -> impl IntoResponse {
    ws.on_upgrade(move |socket| handle_socket(socket, session_id, registry))
}

async fn handle_socket(socket: WebSocket, session_id: Uuid, registry: Arc<RoomRegistry>) {
    let session_key = session_id.to_string();
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();

    let connection_id = registry.join(&session_key, tx);
    tracing::info!("Client {connection_id} connected to room {session_key}");

    // Bestätigung an den Client senden
    let connected = ServerEvent::Connected { session_code: session_key.clone() };
    let _ = ws_tx.send(Message::Text(connected.to_ws_text().into())).await;

    // Task: Channel → WebSocket
    let key_clone = session_key.clone();
    let mut send_task = tokio::spawn(async move {
        while let Some(msg) = rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
        tracing::debug!("Send task ended for room {key_clone}");
    });

    // Task: WebSocket → Client-Events verarbeiten
    let registry_clone = Arc::clone(&registry);
    let key_clone = session_key.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
                    if let Ok(event) = serde_json::from_str::<ClientEvent>(&text) {
                        match event {
                            ClientEvent::Pong => {
                                tracing::debug!("Pong from {connection_id}");
                            }
                        }
                    }
                }
                Message::Close(_) => {
                    tracing::info!("Client {connection_id} closed");
                    break;
                }
                _ => {}
            }
        }
        tracing::debug!("Recv task ended for room {key_clone}");
        drop(registry_clone);
    });

    tokio::select! {
        _ = &mut send_task => recv_task.abort(),
        _ = &mut recv_task => send_task.abort(),
    }

    registry.leave(&session_key, connection_id);
    tracing::info!(
        "Client {connection_id} left room {session_key}. Room size: {}",
        registry.room_size(&session_key)
    );
}
