// don_core/src/guard.rs

use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use std::env;
use crate::auth::Claims;


pub struct DonAdmin;

#[async_trait]
impl<S> FromRequestParts<S> for DonAdmin
where
    S: Send + Sync,
{
    type Rejection = (StatusCode, String);

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        
        let auth_header = parts.headers.get("authorization").and_then(|h| h.to_str().ok());
        let auth_header = match auth_header {
            Some(header) => header,
            None => return Err((StatusCode::UNAUTHORIZED, "Missing Token! Please login.".to_string())),
        };

        if !auth_header.starts_with("Bearer ") {
            return Err((StatusCode::UNAUTHORIZED, "Invalid Token Format!".to_string()));
        }

        let token = &auth_header[7..];
        let secret = env::var("JWT_SECRET").expect("JWT_SECRET missing in .env");

        
        let decoded = decode::<Claims>(
            token,
            &DecodingKey::from_secret(secret.as_bytes()),
            &Validation::default(),
        ).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid or Expired Token!".to_string()))?;

        
        if decoded.claims.role != "admin" {
            return Err((StatusCode::FORBIDDEN, "Access Denied: Superuser (Admin) only!".to_string()));
        }

        
        Ok(DonAdmin)
    }
}