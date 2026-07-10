// example_app/src/main.rs

use don_core::{DonServer, axum::Router};
use don_macros::{DonAuth, DonSocket}; // NAYA: DonSocket import kiya

#[derive(DonAuth)]
pub struct User { pub email: String }

// ==========================================
// JADOO: 1-LINE WEBSOCKET CHAT ENGINE!
// ==========================================
#[derive(DonSocket)]
pub struct LiveChat; // User ne sirf ek khali struct banaya!

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Real-Time WebSockets...");

    let api_routes = Router::new()
        // NAYA: WebSocket route attach kar diya
        .nest("/api/chat", LiveChat::get_ws_routes());

    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}