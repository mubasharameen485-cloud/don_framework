

use don_core::{
    DonServer, AppState,DonAuthHooks, DonAdmin, 
    axum::{Router, extract::{State, Path}, Json, routing::{get, put}}
};
use don_macros::DonAuth;
use serde::{Deserialize, Serialize};

// ==========================================
// 1. STRICT AUTH MODEL
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth)]
#[don_auth_key = "username"] 
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
    pub is_suspended: bool, 
}

//DonAuthHppls is empty if you like than add any logic and function
impl DonAuthHooks for User {}
// ==========================================
// 2. ADMIN LOGIC (CUSTOM HANDLERS)
// ==========================================

// A. Get All Users (Admin Only)
async fn get_all_users(
    _admin: DonAdmin, // 1. Guard: only admin allowed
    State(state): State<AppState>, // 2. Database access
) -> Result<Json<Vec<User>>, String> {
    
    // Custom SQL Query to fetch all users
    let users = don_core::sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(users))
}

// B. Suspend a User (Admin Only)
async fn suspend_user(
    _admin: DonAdmin, // Guard
    State(state): State<AppState>, // Database access
    Path(user_id): Path<i32>, // URL  User ID (e.g., /admin/suspend/5)
) -> Result<Json<don_core::serde_json::Value>, String> {
    
    // Custom SQL Query to update user status
    don_core::sqlx::query("UPDATE users SET is_suspended = TRUE WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(don_core::serde_json::json!({
        "success": true,
        "message": format!("User ID {} has been suspended successfully!", user_id)
    })))
}

// ==========================================
// 3. START THE SERVER
// ==========================================
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Admin Logic...");

    // Admin Routes Setup
    let admin_routes = Router::new()
        .route("/admin/users", get(get_all_users))
        .route("/admin/suspend/:id", put(suspend_user)); // PUT request for updating

    DonServer::new()
        .port(8080)
        .auth_key("username")
        .with_routes(User::get_auth_routes())
        .with_routes(admin_routes) // Inject Admin routes
        .start()
        .await
        .expect("Server crashed!");
}