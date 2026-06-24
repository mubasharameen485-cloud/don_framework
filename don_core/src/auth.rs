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
use sqlx::{Postgres, QueryBuilder, Row};

use crate::server::AppState;

pub fn generate_routes() -> Router<AppState> {
    Router::new()
        .route("/auth/signup", post(dynamic_signup_handler))
        .route("/auth/login", post(fixed_login_handler))
}

#[derive(Deserialize, Debug)]
pub struct DynamicPayload {
    #[serde(flatten)]
    pub fields: std::collections::HashMap<String, Value>,
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
}


#[derive(Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub role: String, 
    pub exp: usize,
}

// ==========================================
// DYNAMIC SIGNUP HANDLER 
// ==========================================
async fn dynamic_signup_handler(
    State(state): State<AppState>,
    Json(mut payload): Json<DynamicPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    let email = payload.fields.get("email").and_then(|v| v.as_str()).map(|s| s.to_string());
    let password = payload.fields.get("password").and_then(|v| v.as_str()).map(|s| s.to_string());

    if email.is_none() || password.is_none() {
        return Err((StatusCode::BAD_REQUEST, "Email and password are required!".to_string()));
    }

    let email = email.unwrap();
    let password = password.unwrap();

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt).unwrap().to_string();

    payload.fields.insert("password".to_string(), Value::String(password_hash));

    let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new("INSERT INTO users (");
    let mut separated = query_builder.separated(", ");
    for key in payload.fields.keys() { separated.push(key); }
    separated.push_unseparated(") VALUES (");
    
    let mut separated_values = query_builder.separated(", ");
    for value in payload.fields.values() {
        match value {
            Value::String(s) => { separated_values.push_bind(s.clone()); },
            Value::Number(n) => {
                if let Some(i) = n.as_i64() { separated_values.push_bind(i); }
                else if let Some(f) = n.as_f64() { separated_values.push_bind(f); }
            },
            Value::Bool(b) => { separated_values.push_bind(*b); },
            _ => { separated_values.push_bind(value.to_string()); }
        }
    }
    separated_values.push_unseparated(")");

    match query_builder.build().execute(&state.db).await {
        Ok(_) => Ok(Json(AuthResponse { success: true, message: "Account created!".to_string(), token: None })),
        Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
    }
}

// ==========================================
// FIXED LOGIN HANDLER (Superuser Logic Added)
// ==========================================
async fn fixed_login_handler(
    State(state): State<AppState>,
    Json(payload): Json<LoginPayload>,
) -> Result<Json<AuthResponse>, (StatusCode, String)> {
    
    
    let super_email = env::var("SUPERUSER_EMAIL").unwrap_or_default();
    let super_pass = env::var("SUPERUSER_PASSWORD").unwrap_or_default();

    let (role, final_email) = if payload.email == super_email && payload.password == super_pass {
        // Agar Superuser hai, toh database check mat karo!
        info!("DonFramework: Superuser (Admin) logged in!");
        ("admin".to_string(), super_email)
    } else {
        // 2. NORMAL USER CHECK (Database se)
        let row = sqlx::query("SELECT password FROM users WHERE email = $1")
            .bind(&payload.email)
            .fetch_optional(&state.db)
            .await
            .map_err(|_| (StatusCode::INTERNAL_SERVER_ERROR, "DB error".to_string()))?
            .ok_or((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()))?;

        let stored_hash: String = row.try_get("password").unwrap();
        let parsed_hash = PasswordHash::new(&stored_hash).unwrap();
        
        if !Argon2::default().verify_password(payload.password.as_bytes(), &parsed_hash).is_ok() {
            return Err((StatusCode::UNAUTHORIZED, "Invalid email or password".to_string()));
        }
        
        info!("DonFramework: Normal user logged in!");
        ("user".to_string(), payload.email.clone())
    };

    // 3. JWT TOKEN GENERATION 
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET missing");
    let expiration = Utc::now().checked_add_signed(Duration::hours(24)).unwrap().timestamp() as usize;

    let claims = Claims {
        sub: final_email,
        role, // "admin" or "user"
        exp: expiration,
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_bytes())).unwrap();

    Ok(Json(AuthResponse {
        success: true,
        message: "Login successful!".to_string(),
        token: Some(token),
    }))
}