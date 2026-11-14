// don_cli/src/main.rs

use clap::{Parser, Subcommand};
use colored::*;
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "don")]
#[command(
    about = "🦀 Don Framework CLI - Build blazing-fast, Django-like Rust APIs instantly",
    long_about = "Official CLI tool to scaffold, manage, and scale Don Framework web applications."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Scaffold a brand new Don Framework project
    New {
        /// Name of the project folder
        name: String,
    },
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::New { name } => {
            create_project(name);
        }
    }
}

fn create_project(project_name: &str) {
    let project_path = Path::new(project_name);

    if project_path.exists() {
        eprintln!(
            "{} Directory '{}' already exists! Choose another name.",
            " Error:".red().bold(),
            project_name
        );
        std::process::exit(1);
    }

    println!(
        "{} Creating a new Don Framework project: {}",
        "".bold(),
        project_name.cyan().bold()
    );

    let dirs = [
        format!("{}/src", project_name),
        format!("{}/migrations", project_name),
        format!("{}/uploads", project_name),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir).unwrap_or_else(|e| {
            eprintln!("Failed to create folder {}: {}", dir, e);
            std::process::exit(1);
        });
    }

    // 1. Write Cargo.toml
    let cargo_toml_content = format!(
r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1.36", features = ["full"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"
sqlx = {{ version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }}
dotenvy = "0.15"

# Don Framework Core & Macros
don_core = "0.3.1"
don_macros = "0.3.1"
"#,
        project_name
    );
    fs::write(format!("{}/Cargo.toml", project_name), cargo_toml_content).unwrap();

    // 2. Write .env file with clear instructions for the user's database password
    let env_content = format!(
r#"# Database Connection String
# REPLACE 'postgres' and 'your_password' with your actual PostgreSQL / pgAdmin credentials!
DATABASE_URL=postgres://postgres:your_password@localhost:5432/{}_db

# Security & Admin Config
JWT_SECRET=super_secret_don_key_12345678901234567890
SUPERUSER_ID=admin_boss
SUPERUSER_PASSWORD=supersecret
"#,
        project_name
    );
    fs::write(format!("{}/.env", project_name), env_content).unwrap();

    // 3. Write default migration (migrations/init.sql)
    let migration_content = r#"-- Initial migration for Don Framework
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user'
);

CREATE TABLE IF NOT EXISTS products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price INT NOT NULL
);
"#;
    fs::write(
        format!("{}/migrations/init.sql", project_name),
        migration_content,
    )
    .unwrap();

    // 4. Write boilerplate src/main.rs (Latest Strict Auth + Hooks + CRUD)
    let main_rs_content = r#"use don_core::{DonServer, DonHooks, axum::Router};
use don_core::traits::DonAuthHooks;
use don_core::upload::get_upload_routes;
use don_macros::{DonAuth, DonModel};
use serde::{Deserialize, Serialize};

// 1. Strict Authentication Model
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth)]
#[don_auth_key = "username"]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
}

// Auth Hooks (Pre-signup and Pre-login logic)
impl DonAuthHooks for User {
    async fn before_signup(&mut self) -> Result<(), String> {
        Ok(())
    }
    async fn before_login(_primary_key: &str) -> Result<(), String> {
        Ok(())
    }
}

// 2. Active Record CRUD Model
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32,
    pub name: String,
    pub price: i32,
}

// Lifecycle hooks for custom validation
impl DonHooks for Product {
    async fn before_save(&mut self) -> Result<(), String> {
        self.name = self.name.trim().to_uppercase();
        Ok(())
    }
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!(" Welcome to your new Don Framework App!");

    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes())
        .nest("/api/upload", get_upload_routes());

    DonServer::new()
        .port(8080)
        .auth_key("username")
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}
"#;
    fs::write(format!("{}/src/main.rs", project_name), main_rs_content).unwrap();

    // 5. Write .gitignore
    let gitignore_content = r#"/target
.env
uploads/*
!uploads/.gitkeep
"#;
    fs::write(format!("{}/.gitignore", project_name), gitignore_content).unwrap();
    fs::write(format!("{}/uploads/.gitkeep", project_name), "").unwrap();

    // 6. Success Output
    println!(
        "{} Created project directory: {}",
        "✔".green().bold(),
        project_name
    );
    println!("{} Generated Cargo.toml with dependencies", "✔".green().bold());
    println!("{} Created .env configuration (Remember to update your DB password!)", "✔".green().bold());
    println!("{} Created migrations/init.sql", "✔".green().bold());
    println!("{} Created boilerplate src/main.rs", "✔".green().bold());

    println!("\n{}", " Project scaffolded successfully!".green().bold());
    println!("\nNext steps to run your app:");
    println!("  cd {}", project_name.yellow().bold());
    println!("  sqlx database create   {}", "# Make sure Postgres is running".dimmed());
    println!("  cargo run\n");
}