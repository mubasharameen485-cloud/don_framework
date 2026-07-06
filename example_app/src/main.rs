// example_app/src/main.rs

use don_core::{DonServer, axum::Router};
// NAYA: DonGuard import kiya
use don_macros::{DonAuth, DonGuard}; 

#[derive(DonAuth)]
pub struct User { pub email: String }

// ==========================================
// JADOO: CUSTOM ROLE GUARDS (IAM)
// ==========================================

// 1. Manager Guard banaya
#[derive(DonGuard)]
#[don_role = "manager"]
pub struct ManagerGuard;

// 2. Editor Guard banaya
#[derive(DonGuard)]
#[don_role = "editor"]
pub struct EditorGuard;

// ==========================================
// PROTECTED ROUTES
// ==========================================

// Yeh route sirf Manager khol sakta hai
async fn manager_dashboard(_guard: ManagerGuard) -> &'static str {
    "Welcome Manager! You have access to the financial reports. 📊"
}

// Yeh route sirf Editor khol sakta hai
async fn editor_dashboard(_guard: EditorGuard) -> &'static str {
    "Welcome Editor! You can write and edit articles. 📝"
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Custom RBAC...");

    let custom_routes = Router::new()
        .route("/manager/dashboard", don_core::axum::routing::get(manager_dashboard))
        .route("/editor/dashboard", don_core::axum::routing::get(editor_dashboard));

    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(custom_routes)
        .start()
        .await
        .expect("Server crashed!");
}