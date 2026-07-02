// example_app/src/main.rs

use don_core::{DonServer, axum::Router, DonAdmin, DonHooks}; // NAYA: DonHooks import kiya
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


impl DonHooks for Product {
    async fn before_save(&mut self) -> Result<(), String> {
        // 1. Validation Logic
        if self.price <= 0 {
            return Err("Validation Error: Price must be greater than 0!".to_string());
        }
        if self.name.trim().is_empty() {
            return Err("Validation Error: Product name cannot be empty!".to_string());
        }

        
        self.name = self.name.trim().to_uppercase();

        Ok(()) 
    }
}

async fn admin_dashboard_handler(_admin: DonAdmin) -> &'static str {
    "Welcome to the Don Framework! You have Superuser (Admin) Access. 🚀"
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("App Starting...");

    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes())
        .route("/admin/dashboard", don_core::axum::routing::get(admin_dashboard_handler));

    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crash ho gaya!");
}