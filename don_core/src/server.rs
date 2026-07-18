// don_core/src/server.rs

use axum::Router;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::net::TcpListener;
use tracing::info;
use tower_http::services::ServeDir;
use tokio::sync::broadcast;
// NAYA: CORS import kiya
use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub tx: broadcast::Sender<String>, 
    pub auth_key: String,
}

pub struct DonServer {
    port: u16,
    router: Router<AppState>,
    auth_key: String,
}

impl DonServer {
    pub fn new() -> Self {
        DonServer {
            port: 3000,
            router: Router::new(),
            auth_key: "email".to_string(),
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn auth_key(mut self, key: &str) -> Self {
        self.auth_key = key.to_string();
        self
    }

    pub fn with_routes(mut self, routes: Router<AppState>) -> Self {
        self.router = self.router.merge(routes);
        self
    }

    pub async fn start(self) -> Result<(), Box<dyn std::error::Error>> {
        dotenvy::dotenv().ok();
        tracing_subscriber::fmt().init();

        info!("Starting Don Framework Engine...");

        let db_url = env::var("DATABASE_URL").expect("DATABASE_URL is missing in .env");
        let pool = PgPoolOptions::new()
            .max_connections(5)
            .connect(&db_url)
            .await?;

        let (tx, _rx) = broadcast::channel::<String>(100);

        let state = AppState { 
            db: pool, 
            tx, 
            auth_key: self.auth_key.clone() 
        };

        // ==========================================
        // JADOO: CORS LAYER (Browser Errors Khatam!)
        // ==========================================
        let cors = CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any);

        let app = self.router
            .layer(cors) // CORS yahan laga diya!
            .nest_service("/uploads", ServeDir::new("uploads"))
            .with_state(state);

        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!("🚀 Don Server is running on http://{}", addr);
        info!("🔐 Authentication Primary Key set to: '{}'", self.auth_key);
        
        axum::serve(listener, app).await?;

        Ok(())
    }
}