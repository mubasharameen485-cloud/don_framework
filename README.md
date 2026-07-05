

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

cargo new my_don_app
cd my_don_app

```
Add the required dependencies to your Cargo.toml:
###cargo.toml
```toml
[dependencies]
tokio = { version = "1.36", features = ["full"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
dotenvy = "0.15"


# The Don Framework
don_core = "0.1.1"
don_macros = "0.1.0"
```
Create a .env file in the root of your project:

###.env
```env
DATABASE_URL=postgres://postgres:password@localhost:5432/don_app_db
JWT_SECRET=your_super_secret_jwt_key_12345
SUPERUSER_EMAIL=admin@don.com
SUPERUSER_PASSWORD=supersecret
```
```
Setup your PostgreSQL Database:

sqlx database create
sqlx migrate add init_users
```
```
In the generated .sql migration file, add the following table:
### init_users.sql
```.sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    metadata JSONB DEFAULT '{}'
);
```
```bash
sqlx database create
sqlx migrate add init_users
```
```bash
sqlx migrate run
```

🔐 2. Authentication Made Easy

With Don Framework, you don't need to write complex Axum handlers for
authentication. Just define your User struct!
```
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
```
Run your server:
```
**cargo run**
```
```
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

```
---

```

## 🛡️ 3. Route Protection & Admin Guards

Don Framework provides a built-in, zero-configuration security guard (`DonAdmin`) to protect your sensitive routes. Only users with the `admin` role (like the Superuser defined in your `.env`) can access these endpoints.

### Protecting Any Custom Route

You don't need to write complex middleware. Simply add `_admin: DonAdmin` as a parameter to your Axum handler. The framework will automatically intercept the request, verify the JWT token, check the user's role, and block unauthorized access!
```
```
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
```
```
🧪 Test the Protected Route

1. Try accessing without a token (Hacker attempt):

curl -X GET http://localhost:8080/admin/dashboard

Output: Missing Token! Please login. 🛑

2. Login as the Superuser (Defined in your .env):

curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "admin@don.com", "password": "supersecret"}'
```
(Copy the token string from the JSON response).

3. Access the route with the Token: Replace YOUR_TOKEN_HERE with the actual
token you copied.

curl -X GET http://localhost:8080/admin/dashboard \
     -H "Authorization: Bearer YOUR_TOKEN_HERE"

Output: Welcome to the Secure Dashboard! You have Admin access. 


---


## 📦 4. Active Record ORM (Full CRUD API)

Tired of writing repetitive SQL queries and API handlers for every database table? Don Framework introduces the `#[derive(DonModel)]` macro. 

By simply attaching this macro to your struct, the framework automatically generates **5 RESTful API routes** (Create, Read All, Read One, Update, Delete) and their underlying PostgreSQL queries!

### Step 1: Create the Database Table

First, create a migration for your new model (e.g., `Product`):
```bash
sqlx migrate add create_products
```

Add the SQL code to the generated migration file:
```sql
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price INT NOT NULL
);
```
Run the migration:
```bash
sqlx migrate run
```
Step 2: Define Your Model and Mount Routes

Update your src/main.rs to include the new Product model:
```rust
use don_core::{DonServer, axum::Router};
use don_macros::{DonAuth, DonModel};
use serde::{Deserialize, Serialize};

#[derive(DonAuth)]
pub struct User {
    pub email: String,
}

// 1. Define your Database Model
// The `DonModel` macro generates SQL queries and Axum handlers automatically!
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, // ID is required. Pass 0 when creating, DB will auto-increment.
    pub name: String,
    pub price: i32,
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // 2. Mount the auto-generated CRUD routes under a specific path
    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes());

    // 3. Start the Server
    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes) // Inject the CRUD routes
        .start()
        .await
        .expect("Server crashed!");
}
```
🧪 Test the CRUD API

Run your server (cargo run), open a new terminal, and test the auto-generated
endpoints!

#### 1. CREATE (POST): Add a new product
```bash
curl -X POST http://localhost:8080/api/products \
     -H "Content-Type: application/json" \
     -d '{"id": 0, "name": "MacBook Pro", "price": 2000}'
```

#### 2. READ ALL (GET): Fetch all products
```bash
curl -X GET http://localhost:8080/api/products
```

#### 3. READ ONE (GET): Fetch a single product by ID
```bash
curl -X GET http://localhost:8080/api/products/1
```

#### 4. UPDATE (PUT): Update an existing product
```bash
curl -X PUT http://localhost:8080/api/products/1 \
     -H "Content-Type: application/json" \
     -d '{"id": 1, "name": "MacBook Pro M3 Max", "price": 3500}'
```
#### 5. DELETE (DELETE): Remove a product
```bash
curl -X DELETE http://localhost:8080/api/products/1



```
---
---
## 🪝 5. Lifecycle Hooks & Custom Validation

Auto-generated CRUD is great, but what if you need to validate data or hash a password before saving it to the database? 

Don Framework solves the "macro magic boundary" problem by providing the `DonHooks` trait. You can easily intercept data before it is saved or updated!

### Implementing Hooks

Simply implement the `DonHooks` trait for your model. In this example, we validate that the price is greater than 0, and we automatically convert the product name to UPPERCASE before saving it to the database.

```rust
use don_core::{DonServer, DonHooks, axum::Router};
use don_macros::DonModel;
use serde::{Deserialize, Serialize};

// ==========================================
// 1. Define your Database Model (Auto-generates CRUD)
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, 
    pub name: String,
    pub price: i32,
}

// ==========================================
// 2. Implement Lifecycle Hooks (Custom Validation & Modification)
// ==========================================
impl DonHooks for Product {
    async fn before_save(&mut self) -> Result<(), String> {
        
        // Custom Validation: Reject negative or zero prices
        if self.price <= 0 {
            return Err("Validation Error: Price must be greater than 0!".to_string());
        }

        // Data Modification: Auto-capitalize the product name before saving
        self.name = self.name.trim().to_uppercase();

        Ok(()) // If everything is fine, proceed to save in the database
    }
}

// ==========================================
// 3. Main Function (Start the Server)
// ==========================================
#[tokio::main]
async fn main() {
    // Load environment variables (.env)
    dotenvy::dotenv().ok();
    println!("App Starting with Hooks...");

    // Define the API routes for the Product model
    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes());

    // Start the Don Server
    DonServer::new()
        .port(8080)
        .with_routes(api_routes) // Pass the defined routes here
        .start()
        .await
        .expect("Server crashed!");
}
```
## Test the Hooks
1. Test Validation Failure (Negative Price):
```
curl -X POST http://localhost:8080/api/products \
     -H "Content-Type: application/json" \
     -d '{"id": 0, "name": "MacBook Pro", "price": -500}'
Output: Validation Error: Price must be greater than 0!  (Database is never touched).
```

2. Test Data Modification (Valid Data):
```
curl -X POST http://localhost:8080/api/products \
     -H "Content-Type: application/json" \
     -d '{"id": 0, "name": "gaming mouse", "price": 50}'
Output: {"id":1,"name":"GAMING MOUSE","price":50} ✅ (Name automatically capitalized!).
```
## 📄 6. Zero-Config Pagination & Query Params

Handling pagination (Limits, Offsets, Query Params) in standard APIs requires writing repetitive boilerplate for every single route. 

**Don Framework does this automatically.** When you use the `#[derive(DonModel)]` macro, the generated `GET /` route is instantly equipped with pagination capabilities. If the user doesn't provide query parameters, it defaults to `page=1` and `limit=10`.

### The Code (`src/main.rs`)

You don't need to write a single line of extra code to enable pagination. Just define your model and start the server! Here is a complete, runnable example:

```rust
use don_core::{DonServer, DonHooks, axum::Router};
use don_macros::DonModel;
use serde::{Deserialize, Serialize};

// ==========================================
// 1. Define your Database Model
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, 
    pub name: String,
    pub price: i32,
}

// ==========================================
// 2. Optional: Lifecycle Hooks
// ==========================================
impl DonHooks for Product {
    async fn before_save(&mut self) -> Result<(), String> {
        self.name = self.name.trim().to_uppercase();
        Ok(()) 
    }
}

// ==========================================
// 3. Start the Server
// ==========================================
#[tokio::main]
async fn main() {
    // Load environment variables (.env)
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Pagination...");

    // Mount the auto-generated CRUD routes
    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes());

    // Start the Don Server
    DonServer::new()
        .port(8080)
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}
```
### Test the Pagination API
Run your server (cargo run) and open a new terminal to test the auto-generated pagination.
## 1. Add some dummy data (Run this 3-4 times with different names):
```
curl -X POST http://localhost:8080/api/products \
     -H "Content-Type: application/json" \
     -d '{"id": 0, "name": "Product A", "price": 100}'
```
## 2. Test Default Pagination (No params provided):

Fetches the latest 10 records (Default: page=1, limit=10).
```
curl -X GET http://localhost:8080/api/products
```
## 3. Test Custom Pagination (The Magic):
Fetch only 2 records from Page 1:
```
curl -X GET "http://localhost:8080/api/products?page=1&limit=2"
```
Fetch the next 2 records from Page 2:
```
curl -X GET "http://localhost:8080/api/products?page=2&limit=2"
```
## 🔗 7. 1-Line Database Relationships

Handling relationships like **One-to-One**, **One-to-Many**, and **Many-to-Many** usually requires writing complex, error-prone SQL `JOIN` queries and custom API handlers. 

Don Framework abstracts this away completely! You can generate fully-functional relationship endpoints with just **1 line of code** using `has_one_route`, `has_many_route`, and `many_to_many_route`.

### The Code (`src/main.rs`)

Here is a complete, runnable example showing how to link Users, Profiles, Products, and Tags without writing a single line of SQL:

```rust
use don_core::{DonServer, axum::Router, DonHooks};
use don_core::{has_many_route, has_one_route, many_to_many_route}; 
use don_macros::{DonAuth, DonModel};
use serde::{Deserialize, Serialize};

#[derive(DonAuth)]
pub struct User { pub email: String }

#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Profile { pub id: i32, pub user_id: i32, pub bio: String }
impl DonHooks for Profile {}

#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product { pub id: i32, pub user_id: i32, pub name: String, pub price: i32 }
impl DonHooks for Product {}

#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Tag { pub id: i32, pub name: String }
impl DonHooks for Tag {}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with 1-Line Relations...");

    let api_routes = Router::new()
        .nest("/api/profiles", Profile::get_api_routes())
        .nest("/api/products", Product::get_api_routes())
        .nest("/api/tags", Tag::get_api_routes())
        
        // ✨ THE MAGIC: 1-Line Relationship Routes!
        
        // 1. ONE-TO-ONE (Get User's Profile -> Returns Object {})
        .merge(has_one_route::<Profile>("/api/users/:id/profile", "profiles", "user_id"))
        
        // 2. ONE-TO-MANY (Get User's Products -> Returns Array [])
        .merge(has_many_route::<Product>("/api/users/:id/products", "products", "user_id"))
        
        // 3. MANY-TO-MANY (Get Product's Tags -> Returns Array [])
        .merge(many_to_many_route::<Tag>("/api/products/:id/tags", "tags", "product_tags", "product_id", "tag_id"));

    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}
```
 ## Test the Relationships API
Run your server (cargo run) and open a new terminal to test the relationships.
## 1. Create Dummy Data:
```
# Create User (ID 1)
curl -X POST http://localhost:8080/auth/signup -H "Content-Type: application/json" -d '{"email": "ali@test.com", "password": "123"}'

# Create Profile for User 1
curl -X POST http://localhost:8080/api/profiles -H "Content-Type: application/json" -d '{"id": 0, "user_id": 1, "bio": "I am a Rust Developer"}'

# Create Product for User 1
curl -X POST http://localhost:8080/api/products -H "Content-Type: application/json" -d '{"id": 0, "user_id": 1, "name": "MacBook", "price": 2000}'

# Create a Tag (ID 1)
curl -X POST http://localhost:8080/api/tags -H "Content-Type: application/json" -d '{"id": 0, "name": "Electronics"}'
```
(Note: For Many-to-Many, manually link product_id=1 and tag_id=1 in your database's product_tags table).

## 2. Test ONE-TO-ONE (Get User's Profile):
   ```
curl -X GET http://localhost:8080/api/users/1/profile
```
## 4. Test ONE-TO-MANY (Get User's Products):
```
curl -X GET http://localhost:8080/api/users/1/products
```
## 5. Test MANY-TO-MANY (Get Product's Tags):
```
curl -X GET http://localhost:8080/api/products/1/tags
```

##  How It Works (Under the Hood)

The Don Framework is built on the principles of **Procedural Macros (Meta-Programming)** and the **Active Record Pattern**.

Instead of manually writing repetitive SQL queries, CRUD handlers, and route definitions for every database table, Don Framework leverages Rust's `proc-macro` system to analyze your structs at compile time. It automatically generates the required SQL operations, Axum route handlers, and database bindings, significantly reducing boilerplate while preserving Rust's type safety and performance.

### 🔐 Dynamic Authentication Metadata

For authentication, Don Framework supports a **schema-less dynamic payload** approach.

During user registration, if the client sends additional fields such as `age`, `gender`, `city`, or any other custom attributes, the framework automatically separates these unknown fields from the typed Rust struct and stores them inside a PostgreSQL `JSONB` metadata column.

This approach combines the safety and performance of strongly typed Rust models with the flexibility of NoSQL-style dynamic data—without requiring developers to write custom SQL or serialization logic.

---

##  About the Author

Hi, I'm **M. Mubashar Ameen**, a Full-Stack and Backend Systems Engineer specializing in the **MERN Stack**, **Next.js**, **Rust (gRPC, distributed systems)**, and **Web3**.

My journey with Rust began because of its exceptional performance, memory safety, and reliability. However, I quickly realized that building even simple REST APIs often involved a significant amount of repetitive boilerplate.

Coming from Python's Django ecosystem, I wanted to bring the same "plug-and-play" developer experience to Rust.

After exploring Rust's procedural macro system and experimenting with compile-time code generation, I created the initial version of **Don Framework**. Throughout the development process, **Google AI Studio** was used extensively as a brainstorming and learning companion while designing the architecture and refining ideas.

Don Framework is an ongoing project, and the long-term vision is to make Rust backend development faster, cleaner, and more enjoyable for developers of all experience levels.

###  Connect

* **LinkedIn:** https://www.linkedin.com/in/m-mubashar-ameen-637359397/
* **Email:**mubfreelance332@gmail.com






