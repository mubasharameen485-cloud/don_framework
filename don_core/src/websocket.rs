// don_core/src/websocket.rs

use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    response::Response,
};
use tracing::{info, error};
use crate::server::AppState;

/// ==========================================
/// UNIVERSAL WEBSOCKET HANDLER
/// ==========================================
pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| handle_socket(socket, state))
}

async fn handle_socket(mut socket: WebSocket, state: AppState) {
    
    let mut rx = state.tx.subscribe();
    info!("DonFramework: new setup okk!");

    loop {
        tokio::select! {
            
            msg_result = socket.recv() => {
                let msg = match msg_result {
                    Some(Ok(m)) => m,
                    Some(Err(e)) => { error!("WebSocket Error: {}", e); break; }
                    None => { info!("DonFramework: User disconnect ho gaya."); break; }
                };

                if let Message::Text(text) = msg {
                    info!("DonFramework: Live message aaya -> {}", text);
                    
                    
                    let _ = state.tx.send(text);
                }
            }

            
            channel_result = rx.recv() => {
                if let Ok(broadcast_msg) = channel_result {
                    if let Err(e) = socket.send(Message::Text(broadcast_msg)).await {
                        error!("Message bhejne mein masla: {}", e);
                        break;
                    }
                }
            }
        }
    }
}