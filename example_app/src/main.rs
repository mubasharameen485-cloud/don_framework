// example_app/src/main.rs

use don_core::DonServer;
use don_macros::DonAuth;


#[derive(DonAuth)]
pub struct User {
    pub email: String,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Custom Auth Key...");

    DonServer::new()
        .port(8080)
        
        .auth_key("school") 
        .with_routes(User::get_auth_routes())
        .start()
        .await
        .expect("Server crashed!");
}