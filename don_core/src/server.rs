// don_core/src/server.rs

use axum::Router;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::env;
use tokio::net::TcpListener;
use tracing::info;


#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
}


pub struct DonServer {
    port: u16,
    router: Router<AppState>,
}

impl DonServer {
    
    pub fn new() -> Self {
        DonServer {
            port: 3000, 
            router: Router::new(),
        }
    }

    
    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
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

        let state = AppState { db: pool };

        
        let app = self.router.with_state(state);

        
        let addr = format!("0.0.0.0:{}", self.port);
        let listener = TcpListener::bind(&addr).await?;
        
        info!(" Don Server is running on http://{}", addr);
        axum::serve(listener, app).await?;

        Ok(())
    }
}