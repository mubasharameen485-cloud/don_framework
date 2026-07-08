// don_core/src/relations.rs

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;
use sqlx::{postgres::PgRow, FromRow};
use crate::server::AppState;

/// ==========================================
/// 1. ONE-TO-MANY (1-to-N)
/// 
/// ==========================================
pub fn has_many_route<T>(
    route_path: &str,
    child_table: &str,
    foreign_key_column: &str,
) -> Router<AppState>
where
    T: for<'r> FromRow<'r, PgRow> + Serialize + Send + Sync + Unpin + 'static,
{
    let query = format!("SELECT * FROM {} WHERE {} = $1", child_table, foreign_key_column);

    Router::new().route(
        route_path,
        get(move |State(state): State<AppState>, Path(parent_id): Path<i32>| async move {
            match sqlx::query_as::<_, T>(&query).bind(parent_id).fetch_all(&state.db).await {
                Ok(records) => Ok(Json(records)),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            }
        }),
    )
}

/// ==========================================
/// 2. ONE-TO-ONE (1-to-1) & MANY-TO-ONE (N-to-1)
/// 
/// ==========================================
pub fn has_one_route<T>(
    route_path: &str,
    target_table: &str,
    foreign_key_column: &str,
) -> Router<AppState>
where
    T: for<'r> FromRow<'r, PgRow> + Serialize + Send + Sync + Unpin + 'static,
{
    // LIMIT 1 ensures we only get a single object, not an array
    let query = format!("SELECT * FROM {} WHERE {} = $1 LIMIT 1", target_table, foreign_key_column);

    Router::new().route(
        route_path,
        get(move |State(state): State<AppState>, Path(parent_id): Path<i32>| async move {
            match sqlx::query_as::<_, T>(&query).bind(parent_id).fetch_optional(&state.db).await {
                Ok(Some(record)) => Ok(Json(record)),
                Ok(None) => Err((StatusCode::NOT_FOUND, "Record not found".to_string())),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            }
        }),
    )
}

/// ==========================================
/// 3. MANY-TO-MANY (N-to-N)
/// Example: Products have many Tags, Tags belong to many Products. (Returns a List/Array)
/// ==========================================
pub fn many_to_many_route<T>(
    route_path: &str,
    target_table: &str,
    join_table: &str,
    parent_fk_in_join: &str,
    target_fk_in_join: &str,
) -> Router<AppState>
where
    T: for<'r> FromRow<'r, PgRow> + Serialize + Send + Sync + Unpin + 'static,
{
    let query = format!(
        "SELECT t.* FROM {} t JOIN {} j ON t.id = j.{} WHERE j.{} = $1",
        target_table, join_table, target_fk_in_join, parent_fk_in_join
    );

    Router::new().route(
        route_path,
        get(move |State(state): State<AppState>, Path(parent_id): Path<i32>| async move {
            match sqlx::query_as::<_, T>(&query).bind(parent_id).fetch_all(&state.db).await {
                Ok(records) => Ok(Json(records)),
                Err(e) => Err((StatusCode::INTERNAL_SERVER_ERROR, e.to_string())),
            }
        }),
    )
}