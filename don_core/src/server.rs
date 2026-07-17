// don_core/src/server.rs

use axum::Router;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::net::TcpListener;
use tracing::info;
use tower_http::services::ServeDir;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub tx: broadcast::Sender<String>, 
    pub auth_key: String, // NAYA: Login ke liye konsi field use karni hai?
}

pub struct DonServer {
    port: u16,
    router: Router<AppState>,
    auth_key: String, // NAYA
}

impl DonServer {
    pub fn new() -> Self {
        DonServer {
            port: 3000,
            router: Router::new(),
            auth_key: "email".to_string(), // Default email rahega
        }
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    // JADOO: User yahan batayega ke login kis field se karna hai!
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

        // State mein auth_key save kar di
        let state = AppState { 
            db: pool, 
            tx, 
            auth_key: self.auth_key.clone() 
        };

        let app = self.router
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