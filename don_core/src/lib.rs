// don_core/src/lib.rs

pub mod server;
pub mod guard;
pub mod traits;
pub mod relations;
pub mod upload;
pub mod websocket;
pub mod validation;

pub use server::{DonServer, AppState};
pub use guard::DonAdmin;
pub use traits::{DonHooks, DonAuthHooks};
pub use relations::{has_many_route, has_one_route, many_to_many_route};

// MACRO KE LIYE LIBRARIES EXPORT KAR RAHE HAIN
pub use axum; 
pub use sqlx; 
pub use jsonwebtoken;
pub use argon2;
pub use chrono;
pub use serde_json;
pub use serde;

use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct QueryParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}