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
        .route("/auth/signup", post(flexible_signup_handler))
        .route("/auth/login", post(fixed_login_handler))
}

// ==========================================
// DATA MODELS
// ==========================================
#[derive(Deserialize, Debug)]
pub struct SignupPayload {
    pub email: String,
    pub password: String,
   
    #[serde(flatten)]
    pub metadata: std::collections::HashMap<String, Value>,
}

#[derive(Deserialize, Debug)]
pub struct LoginPayload {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub success: bool,
    pub message: String,
    pub token: Option<String>,
    pub user_data: Option<Value>, 
}

#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String,
    pub exp: usize,
}

// ==========================================
// FLEXIBLE SIGNUP HANDLER (JSONB Approach)
// ==========================================
async fn flexible_signup_handler(
    State(state): State<AppState>,
    Json(payload): Json<SignupPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    info!("DonFramework: Signup request for {}", payload.email);

    // 1. Password Hash
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(payload.password.as_bytes(), &salt)
        .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "Hashing failed".to_string()))?
        .to_string();

   
    let metadata_json = serde_json::to_value(&payload.metadata).unwrap_or(serde_json::json!({}));

    
    let result = sqlx::query("INSERT INTO users (email, password_hash, metadata) VALUES ($1, $2, $3)")
        .bind(&payload.email)
        .bind(&password_hash)
        .bind(&metadata_json)
        .execute(&state.db)
        .await;

    match result {
        Ok(_) => {
            info!("DonFramework: User {} registered successfully!", payload.email);
            Ok(Json(AuthResponse {
                success: true,
                message: "Account created successfully!".to_string(),
                token: None,
                user_data: Some(metadata_json),
            }))
        }
        Err(e) => {
            if e.to_string().contains("duplicate key") {
                Err((StatusCode::CONFLICT, "Email already exists!".to_string()))
            } else {
                error!("Database error: {}", e);
                Err((StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()))
            }
        }
    }
}

// ==========================================
// FIXED LOGIN HANDLER
// ==========================================
async fn fixed_login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    info!("DonFramework: Login request for {}", payload.email);

    let super_email = env::var("SUPERUSER_EMAIL").unwrap_or_default();
    let super_pass = env::var("SUPERUSER_PASSWORD").unwrap_or_default();

    let (role, final_email, user_metadata) = if payload.email == super_email && payload.password == super_pass {
        info!("DonFramework: Superuser logged in!");
        ("admin".to_string(), super_email, None)
    } else {
        
        let row = sqlx::query("SELECT password_hash, metadata FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()))?;

        let stored_hash: String = row.try_get("password_hash").unwrap();
        let metadata: Option<Value> = row.try_get("metadata").unwrap_or(None);

        let parsed_hash = PasswordHash::new(&stored_hash).unwrap();
        if !Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_ok() {
            return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
        }
        
        info!("DonFramework: Normal user logged in!");
        ("user".to_string(), payload.email.clone(), metadata)
    };

    let secret = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let expiration = Utc::now().checked_add_signed(Duration::hours(24)).unwrap().timestamp() as usize;

    let claims = Claims {
        sub: final_email,
        role,
        exp: expiration,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap();

    Ok(Json(AuthResponse {
        success: true,
        message: "Login successful!".to_string(),
        token: Some(token),
        user_data: user_metadata, 
    }))
}