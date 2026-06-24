// example_app/src/main.rs

use don_core::{DonServer, axum::Router, DonAdmin}; // DonAdmin import kiya
use don_macros::{DonAuth, DonModel};
use serde::{Deserialize, Serialize};

#[derive(DonAuth)]
pub struct User {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, 
    pub name: String,
    pub price: i32,
}

// ==========================================
// USER KA APNA ADMIN DASHBOARD
// ==========================================
// JADOO: User ne sirf `_admin: DonAdmin` likha hai. 
// Ab dunya ki koi taqat bina Superuser token ke is function ko nahi chala sakti!
async fn my_super_secret_dashboard(_admin: DonAdmin) -> &'static str {
    "Welcome to the Don Framework Admin Dashboard! You are a Superuser."
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // User ne apne routes banaye
    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes())
        // Admin route attach kiya
        .route("/admin/dashboard", don_core::axum::routing::get(my_super_secret_dashboard));

    DonServer::new()
        .port(3000)
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes)
        .start()
        .await
        .expect("Failed to start Don Server");
}