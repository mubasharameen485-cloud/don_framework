// don_core/src/auth.rs

use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use std::env;
use tracing::{error, info};
use serde_json::Value;

use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{encode, EncodingKey, Header};
use sqlx::Row;

use crate::server::AppState;

pub fn generate_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(dynamic_signup_handler))
        .route("/auth/login", post(dynamic_login_handler))
}

#[derive(Deserialize, Debug)]
pub struct DynamicPayload {
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, Value>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub user_data: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

// ==========================================
// 100% DYNAMIC SIGNUP HANDLER (JSONB Approach)
// ==========================================
async fn dynamic_signup_handler(
    State(state): State<AppState>,
    Json(mut payload): Json<DynamicPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let auth_key = &state.auth_key; // e.g., "username", "phone", "email"

    // 1. Extract Primary Key and Password safely (No unwrap!)
    let primary_val = payload.fields.remove(auth_key)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or((StatusCode::BAD_REQUEST, format!("'{}' is required!", auth_key)))?;

    let password = payload.fields.remove("password")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or((StatusCode::BAD_REQUEST, "password is required!".to_string()))?;

    info!("DonFramework: Dynamic Signup request for {} = {}", auth_key, primary_val);

    // 2. Hash Password Safely
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hashing failed".to_string()))?
        .to_string();

    // 3. The rest of the fields become metadata (JSONB)
    let metadata_json = serde_json::to_value(&payload.fields)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Failed to parse metadata".to_string()))?;

    // 4. Dynamic SQL Query (Only inserting primary key, password, and metadata)
    let query = format!(
        "INSERT INTO users ({}, password, metadata) VALUES ($1, $2, $3)",
        auth_key
    );

    let result = sqlx::query(&query)
        .bind(&primary_val)
        .bind(&password_hash)
        .bind(&metadata_json)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => Ok(Json(AuthResponse { 
            success: true, 
            message: "Account created!".to_string(), 
            token: None, 
            user_data: Some(metadata_json) 
        })),
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                Err((StatusCode::CONFLICT, format!("{} already exists!", auth_key)))
            } else {
                error!("Database error: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))
            }
        }
    }
}

// ==========================================
// 100% DYNAMIC LOGIN HANDLER
// ==========================================
async fn dynamic_login_handler(
    State(state): State<AppState>,
    Json(mut payload): Json<DynamicPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let auth_key = &state.auth_key;

    let primary_val = payload.fields.remove(auth_key)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or((StatusCode::BAD_REQUEST, format!("'{}' is required!", auth_key)))?;

    let password = payload.fields.remove("password")
        .and_then(|v| v.as_str().map(|s| s.to_string()))
        .ok_or((StatusCode::BAD_REQUEST, "password is required!".to_string()))?;

    info!("DonFramework: Login request for {} = {}", auth_key, primary_val);

    let super_id = env::var("SUPERUSER_ID").unwrap_or_default();
    let super_pass = env::var("SUPERUSER_PASSWORD").unwrap_or_default();

    let (role, final_id, user_metadata) = if primary_val == super_id && password == super_pass {
        info!("DonFramework: Superuser logged in!");
        ("admin".to_string(), super_id, None)
    } else {
        // Safely query the database based on the dynamic auth_key
        let query = format!("SELECT password, metadata, role FROM users WHERE {} = $1", auth_key);
        
        let row = sqlx::query(&query)
            .bind(&primary_val)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

        let stored_hash: String = row.try_get("password")
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB column error".to_string()))?;
        
        let metadata: Option<Value> = row.try_get("metadata").unwrap_or(None);
        let db_role: String = row.try_get("role").unwrap_or_else(|_| "user".to_string());

        let parsed_hash = PasswordHash::new(&stored_hash)
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hash error".to_string()))?;
            
        if !Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok() {
            return Err((StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
        }
        
        (db_role, primary_val.clone(), metadata)
    };

    let secret = env::var("JWT_SECRET").map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "JWT_SECRET missing".to_string()))?;
    
    let expiration = Utc::now().checked_add_signed(Duration::hours(24))
        .ok_or((StatusCode::INTERNAL_SERVER_ERROR, "Time error".to_string()))?
        .timestamp() as usize;

    let claims = Claims { sub: final_id, role, exp: expiration };
    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes()))
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Token error".to_string()))?;

    Ok(Json(AuthResponse { 
        success: true, 
        message: "Login successful!".to_string(), 
        token: Some(token), 
        user_data: user_metadata 
    }))
}