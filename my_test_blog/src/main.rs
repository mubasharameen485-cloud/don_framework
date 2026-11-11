use don_core::{DonServer, DonHooks, axum::Router};
use don_core::upload::get_upload_routes;
use don_macros::{DonAuth, DonModel};
use serde::{Deserialize, Serialize};

// 1. Authentication Model (Zero-boilerplate Signup & Login)
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth)]
#[don_auth_key = "username"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
}

// 2. Active Record CRUD Model (Auto-generates 5 RESTful routes + Pagination)
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: i32,
}

// Optional: Lifecycle hooks for custom validation
impl DonHooks for Product {
    async fn before_save(&mut self) -> Result<(), String> {
        self.name = self.name.trim().to_uppercase();
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("🦀 Welcome to your new Don Framework App!");

    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes())
        .nest("/api/upload", get_upload_routes());

    DonServer::new()
        .port(8080)
        .auth_key("username")
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}
