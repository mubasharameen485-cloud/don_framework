// don_core/src/upload.rs

use axum::{
    extract::Multipart,
    http::StatusCode,
    routing::post,
    Json, Router,
};
use serde_json::{json, Value};
use tokio::fs;
use uuid::Uuid;
use tracing::{info, error};
use crate::server::AppState;

/// ==========================================
/// UPLOAD ROUTER GENERATOR
/// ==========================================
/// Returns a pre-configured router for handling file uploads.
pub fn get_upload_routes() -> Router<AppState> {
    Router::new().route("/", post(upload_handler))
}

/// ==========================================
/// THE MULTIPART UPLOAD HANDLER
/// ==========================================
async fn upload_handler(mut multipart: Multipart) -> Result<Json<Value>, (StatusCode, String)> {
    info!("DonFramework: Receiving file upload...");

    
    if let Err(e) = fs::create_dir_all("uploads").await {
        error!("Failed to create uploads directory: {}", e);
        return Err((StatusCode::INTERNAL_SERVER_ERROR, "Server file system error".to_string()));
    }

    let mut uploaded_urls = Vec::new();

    
    while let Some(field) = multipart.next_field().await.map_err(|e| {
        error!("Multipart error: {}", e);
        (StatusCode::BAD_REQUEST, "Failed to read upload stream".to_string())
    })? {
        
        let file_name = field.file_name().unwrap_or("unknown_file").to_string();
        if file_name == "unknown_file" {
            continue; // Skip empty fields
        }

        
        let extension = file_name.split('.').last().unwrap_or("bin");
        let unique_name = format!("{}.{}", Uuid::new_v4(), extension);
        let save_path = format!("uploads/{}", unique_name);

        
        let data = field.bytes().await.map_err(|_| {
            (StatusCode::BAD_REQUEST, "Failed to read file bytes".to_string())
        })?;

        if let Err(e) = fs::write(&save_path, &data).await {
            error!("Failed to save file to disk: {}", e);
            return Err((StatusCode::INTERNAL_SERVER_ERROR, "Failed to save file".to_string()));
        }

        info!("DonFramework: File saved successfully at {}", save_path);
        
        
        uploaded_urls.push(format!("/uploads/{}", unique_name));
    }

    // 5. Return the URLs to the user
    Ok(Json(json!({
        "success": true,
        "message": "Files uploaded successfully!",
        "urls": uploaded_urls
    })))
}