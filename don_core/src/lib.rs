// don_core/src/lib.rs

pub mod server;
pub mod auth;
pub mod guard;
pub mod traits;

pub use server::{DonServer, AppState};
pub use guard::DonAdmin;
pub use traits::DonHooks;
pub use axum; 
pub use sqlx; 

use serde::Deserialize;

// ==========================================
// NAYA JADOO: Pagination Query Params
// ==========================================
// Yeh struct URL se ?page=2&limit=5 ko automatically pakar lega
#[derive(Deserialize, Default, Debug)]
pub struct QueryParams {
    pub page: Option<i64>,
    pub limit: Option<i64>,
}