// example_app/src/main.rs

use don_core::{DonServer, DonHooks, axum::Router};
use don_macros::{DonAuth, DonModel};
use serde::{Deserialize, Serialize};

#[derive(DonAuth)]
pub struct User {
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, 
    pub name: String,
    pub price: i32,
}

impl DonHooks for Product {
    async fn before_save(&mut self) -> Result<(), String> {
        if self.price <= 0 {
            return Err("Validation Error: Price must be greater than 0!".to_string());
        }
        self.name = self.name.trim().to_uppercase();
        Ok(()) 
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("App Starting with Pagination Magic...");

    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes());

    DonServer::new()
        .port(8080)
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}