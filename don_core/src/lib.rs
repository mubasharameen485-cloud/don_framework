// don_core/src/lib.rs

pub mod server;
pub mod auth;
pub mod guard;
pub mod traits;
pub mod relations;
pub mod upload;
pub mod websocket; // NAYA: WebSocket module add kiya

pub use server::{DonServer, AppState};
pub use guard::DonAdmin;
pub use traits::DonHooks;
pub use relations::{has_many_route, has_one_route, many_to_many_route}; 
pub use axum; 
pub use sqlx; 
pub use jsonwebtoken;

use serde::Deserialize;

#[derive(Deserialize, Default, Debug)]
pub struct QueryParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}