// example_app/src/main.rs

use don_core::{DonServer, axum::Router, DonHooks};
// Teeno generators import kiye
use don_core::{has_many_route, has_one_route, many_to_many_route}; 
use don_macros::{DonAuth, DonModel};
use serde::{Deserialize, Serialize};

#[derive(DonAuth)]
pub struct User { pub email: String }

// 1-to-1 Model
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Profile { pub id: i32, pub user_id: i32, pub bio: String }
impl DonHooks for Profile {}

// 1-to-N Model
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product { pub id: i32, pub user_id: i32, pub name: String, pub price: i32 }
impl DonHooks for Product {}

// N-to-N Model
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Tag { pub id: i32, pub name: String }
impl DonHooks for Tag {}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with All Relations...");

    let api_routes = Router::new()
        // Standard CRUD Routes
        .nest("/api/profiles", Profile::get_api_routes())
        .nest("/api/products", Product::get_api_routes())
        .nest("/api/tags", Tag::get_api_routes())
        
        // ==========================================
        // 1-LINE RELATIONSHIP ROUTES!
        // ==========================================
        
        // 1. ONE-TO-ONE (User has 1 Profile) -> Returns Object {}
        .merge(has_one_route::<Profile>("/api/users/:id/profile", "profiles", "user_id"))
        
        // 2. ONE-TO-MANY (User has many Products) -> Returns Array []
        .merge(has_many_route::<Product>("/api/users/:id/products", "products", "user_id"))
        
        // 3. MANY-TO-MANY (Product has many Tags) -> Returns Array []
        .merge(many_to_many_route::<Tag>("/api/products/:id/tags", "tags", "product_tags", "product_id", "tag_id"));

    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}