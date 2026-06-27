

# 🦀 Don Framework

**The Django-like, blazing-fast, and developer-friendly web framework for Rust.**

Building web APIs in Rust is incredibly fast and safe, but it often requires writing a lot of boilerplate code (setting up Axum routers, configuring SQLx pools, hashing passwords with Argon2, generating JWTs, etc.). 

**Don Framework** solves this. It acts as a powerful wrapper over `axum` and `sqlx`. By simply adding macros like `#[derive(DonAuth)]` and `#[derive(DonModel)]` to your structs, the framework automatically generates your database queries, API routes, and security guards!



## 🚀 Features
- **Zero Boilerplate:** Write a struct, get a full API.
- **Auto-Auth:** Instant `/auth/signup` and `/auth/login` routes with Argon2 and JWT.
- **Dynamic Metadata:** Pass any extra JSON fields during signup, and they are safely stored in a Postgres `JSONB` column.
- **Active Record ORM:** Full CRUD API generation for any struct.
- **Admin Guards:** Protect any route with a simple `DonAdmin` extractor.


## 🛠️ 1. Quick Setup

Create a new Rust project:
```bash
-------------------
cargo new my_don_app
cd my_don_app
-------------------
>Add the required dependencies to your Cargo.toml:
**cargo.toml**
[dependencies]
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
dotenvy = "0.15"

# The Don Framework
don_core = "0.1.1"
don_macros = "0.1.0"

Create a .env file in the root of your project:
**.env**
DATABASE_URL=postgres://postgres:password@localhost:5432/don_app_db
JWT_SECRET=your_super_secret_jwt_key_12345
SUPERUSER_EMAIL=admin@don.com
SUPERUSER_PASSWORD=supersecret

Setup your PostgreSQL Database:

sqlx database create
sqlx migrate add init_users

In the generated .sql migration file, add the following table:
**.sql**
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    metadata JSONB DEFAULT '{}'
);

Run the migration:

sqlx migrate run

🔐 2. Authentication Made Easy

With Don Framework, you don't need to write complex Axum handlers for
authentication. Just define your User struct!

In your src/main.rs:
**main.rs**
use don_core::DonServer;
use don_macros::DonAuth;

// 1. Define your Auth Model
// This automatically generates /auth/signup and /auth/login routes!
#[derive(DonAuth)]
pub struct User {
    pub email: String,
}

#[tokio::main]
async fn main() {
    // 2. Start the Server
    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes()) // Inject auto-generated auth routes
        .start()
        .await
        .expect("Server crashed!");
}

Run your server:

**cargo run**

🧪 Test the Auth API

1. Signup (With dynamic extra fields!) You can send any extra fields (like age,
city), and Don Framework will automatically save them in the metadata JSONB
column!

curl -X POST http://localhost:8080/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"email": "john@test.com", "password": "password123", "age": 30, "city": "New York"}'

2. Login (Get your JWT Token)

curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "john@test.com", "password": "password123"}'


---



## 🛡️ 3. Route Protection & Admin Guards

Don Framework provides a built-in, zero-configuration security guard (`DonAdmin`) to protect your sensitive routes. Only users with the `admin` role (like the Superuser defined in your `.env`) can access these endpoints.

### Protecting Any Custom Route

You don't need to write complex middleware. Simply add `_admin: DonAdmin` as a parameter to your Axum handler. The framework will automatically intercept the request, verify the JWT token, check the user's role, and block unauthorized access!

Update your `src/main.rs`:

```rust
use don_core::{DonServer, axum::Router, DonAdmin};
use don_macros::DonAuth;

#[derive(DonAuth)]
pub struct User {
    pub email: String,
}

// 1. Create a Protected Route
// Adding `_admin: DonAdmin` makes this route 100% secure!
async fn secure_dashboard(_admin: DonAdmin) -> &'static str {
    "Welcome to the Secure Dashboard! You have Admin access. "
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // 2. Define your custom routes
    let custom_routes = Router::new()
        .route("/admin/dashboard", don_core::axum::routing::get(secure_dashboard));

    // 3. Start the Server
    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(custom_routes) // Inject the protected route
        .start()
        .await
        .expect("Server crashed!");
}

🧪 Test the Protected Route

1. Try accessing without a token (Hacker attempt):

curl -X GET http://localhost:8080/admin/dashboard

Output: Missing Token! Please login. 🛑

2. Login as the Superuser (Defined in your .env):

curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "admin@don.com", "password": "supersecret"}'

(Copy the token string from the JSON response).

3. Access the route with the Token: Replace YOUR_TOKEN_HERE with the actual
token you copied.

curl -X GET http://localhost:8080/admin/dashboard \
     -H "Authorization: Bearer YOUR_TOKEN_HERE"

Output: Welcome to the Secure Dashboard! You have Admin access. 


---

