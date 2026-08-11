// example_app/src/main.rs

use don_core::{
    DonServer, AppState, DonHooks,
    axum::{Router, extract::State, Json, routing::delete}
};
use don_macros::DonModel;
use validator::Validate;
use serde::{Deserialize, Serialize};

// ==========================================
// 1. THE MODEL (WITH DECLARATIVE VALIDATION)
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel, Validate)]
#[don_validate] // Framework ko bataya ke validation ON karni hai
pub struct Employee {
    pub id: i32, 
    
    #[validate(length(min = 3, message = "Name must be at least 3 characters!"))]
    pub name: String,
    
    #[validate(email(message = "Invalid email format!"))]
    pub email: String,
    
    #[validate(range(min = 18, max = 60, message = "Age must be between 18 and 60!"))]
    pub age: i32,
    
    pub salary: i32, // Iski validation hum Hooks mein karenge!
    pub department: String,
}

// ==========================================
// 2. LIFECYCLE HOOKS (CUSTOM BUSINESS LOGIC)
// ==========================================
impl DonHooks for Employee {
    async fn before_save(&mut self) -> Result<(), String> {
        // Custom Logic 1: Agar department "IT" hai, toh salary kam az kam 5000 honi chahiye
        if self.department == "IT" && self.salary < 5000 {
            return Err("IT Department employees must have a salary of at least 5000!".to_string());
        }

        // Custom Logic 2: Data Formatting
        self.name = self.name.trim().to_uppercase();
        self.department = self.department.trim().to_uppercase();

        Ok(())
    }

    async fn before_update(&mut self) -> Result<(), String> {
        // Update hone se pehle bhi same rules apply karo
        self.before_save().await
    }
}

// ==========================================
// 3. CUSTOM ROUTES (e.g., DELETE MANY)
// ==========================================
// Macro standard 5 routes banata hai. Agar "Delete All" chahiye toh developer aise likhega:
async fn delete_all_employees(State(state): State<AppState>) -> Result<Json<don_core::serde_json::Value>, String> {
    don_core::sqlx::query("DELETE FROM employees")
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;
        
    Ok(Json(don_core::serde_json::json!({"message": "All employees deleted!"})))
}

// ==========================================
// 4. START SERVER
// ==========================================
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Ultimate CRUD Validation...");

    let api_routes = Router::new()
        // Standard 5 CRUD Routes (POST, GET All, GET One, PUT, DELETE One)
        .nest("/api/employees", Employee::get_api_routes())
        // Custom Route (DELETE Many)
        .route("/api/employees/delete_all", delete(delete_all_employees));

    DonServer::new()
        .port(8080)
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}