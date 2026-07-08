// example_app/src/main.rs

use don_core::{DonServer, axum::Router, DonAdmin};
use don_core::upload::get_upload_routes; 
use don_macros::DonAuth;

#[derive(DonAuth)]
pub struct User { pub email: String }

async fn admin_dashboard(_admin: DonAdmin) -> &'static str {
    "Welcome to the Secure Dashboard! "
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with File Uploads...");

    let custom_routes = Router::new()
        .route("/admin/dashboard", don_core::axum::routing::get(admin_dashboard))
        // ==========================================
        // JADOO: 1-Line File Upload API!
        // ==========================================
        .nest("/api/upload", get_upload_routes());

    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(custom_routes)
        .start()
        .await
        .expect("Server crashed!");
}