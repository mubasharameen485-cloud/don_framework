

# 🦀 Don Framework

**The Django-like, blazing-fast, and developer-friendly web framework for Rust.**

Building web APIs in Rust is incredibly fast and safe, but it often requires writing a lot of boilerplate code (setting up Axum routers, configuring SQLx pools, hashing passwords with Argon2, generating JWTs, etc.). 

**Don Framework** solves this. It acts as a powerful wrapper over `axum` and `sqlx`. By simply adding macros like `#[derive(DonAuth)]` and `#[derive(DonModel)]` to your structs, the framework automatically generates your database queries, API routes, and security guards!



## Features
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
Setup your PostgreSQL Database:
```


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
sqlx migrate run
```

## 2. Authentication Made Easy

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
Test the Auth API

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
###  Under the Hood: How `DonAuth` Works

You might be wondering: *"Why is there only an `email` field in the `User` struct? Where is the password and ID? And what exactly is `DonServer` doing?"*

Here is the magic explained:

1. **`#[derive(DonAuth)]`:** This is a Rust Procedural Macro. When the compiler sees this attribute, it automatically generates the `/auth/signup` and `/auth/login` Axum handlers and attaches them to your `User` struct. You don't have to write any routing logic.
2. **Where is the Password?** We intentionally omit the `password` field from the struct for security and abstraction. The framework's internal payload parser catches the password directly from the JSON request, hashes it using **Argon2**, and stores it in the database. You never have to handle raw passwords in your application code.
3. **Where is the ID?** The `id` is handled entirely by PostgreSQL (`SERIAL PRIMARY KEY`). The framework abstracts this away so you don't have to manage auto-incrementing integers.
4. **Dynamic Metadata:** If you add extra fields to your JSON request (like `age` or `city`), the framework catches them and safely stores them in a Postgres `JSONB` column called `metadata`.
5. **`DonServer`:** This is a powerful wrapper. Instead of manually loading `.env` files, setting up `sqlx::PgPool` connections, and binding `tokio::net::TcpListener`, `DonServer` encapsulates all of this setup. You just call `.start()`, and the framework handles the heavy lifting!


##  3. Route Protection & Admin Guards

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
## Database Setup
```
sqlx database drop -y
sqlx database create
sqlx migrate add all_relations_tables
```
past this new .sql file:
```
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    metadata JSONB DEFAULT '{}'
);

-- 1-to-1 Relation (User has 1 Profile)
CREATE TABLE profiles (
    id SERIAL PRIMARY KEY,
    user_id INT UNIQUE NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    bio TEXT NOT NULL
);

-- 1-to-N Relation (User has many Products)
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    user_id INT NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    price INT NOT NULL
);

-- N-to-N Relation (Products have many Tags)
CREATE TABLE tags (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL
);

CREATE TABLE product_tags (
    product_id INT REFERENCES products(id) ON DELETE CASCADE,
    tag_id INT REFERENCES tags(id) ON DELETE CASCADE,
    PRIMARY KEY (product_id, tag_id)
);
```
migration:
```
sqlx migrate run
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

## 🔐 8. Flexible Role-Based Access Control (RBAC)

In a real-world application, you don't just have an "Admin". You have Managers, Editors, Finance teams, etc. Don Framework provides a highly flexible IAM (Identity and Access Management) system.

By using the `#[derive(DonGuard)]` macro, you can generate custom middleware extractors for any role in just 2 lines of code!

### The Code (`src/main.rs`)

Here is a complete, runnable example showing how to create custom roles and protect specific routes:

```rust
use don_core::{DonServer, axum::Router};
use don_macros::{DonAuth, DonGuard}; 

// 1. Auth Model (Handles Signup/Login)
#[derive(DonAuth)]
pub struct User { 
    pub email: String 
}

// ==========================================
// 2. DEFINE CUSTOM ROLE GUARDS
// ==========================================

// Creates a Guard that only allows users with role="manager"
#[derive(DonGuard)]
#[don_role = "manager"]
pub struct ManagerGuard;

// Creates a Guard that only allows users with role="editor"
#[derive(DonGuard)]
#[don_role = "editor"]
pub struct EditorGuard;

// ==========================================
// 3. PROTECTED ROUTES
// ==========================================

// Only Managers can access this route
async fn manager_dashboard(_guard: ManagerGuard) -> &'static str {
    "Welcome Manager! You have access to the financial reports. 📊"
}

// Only Editors can access this route
async fn editor_dashboard(_guard: EditorGuard) -> &'static str {
    "Welcome Editor! You can write and edit articles. 📝"
}

// ==========================================
// 4. START THE SERVER
// ==========================================
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Custom RBAC...");

    // Attach protected routes
    let custom_routes = Router::new()
        .route("/manager/dashboard", don_core::axum::routing::get(manager_dashboard))
        .route("/editor/dashboard", don_core::axum::routing::get(editor_dashboard));

    DonServer::new()
        .port(8080)
        .with_routes(User::get_auth_routes())
        .with_routes(custom_routes)
        .start()
        .await
        .expect("Server crashed!");
}
```
# Test the RBAC System
Run your server (cargo run) and open a new terminal.
## 1. Create a Manager User:
Notice how we pass "role": "manager" in the dynamic JSON payload.
```curl -X POST http://localhost:8080/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"email": "manager@test.com", "password": "123", "role": "manager"}'
```
## 2. Login as Manager (Get the Token):
```
curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "manager@test.com", "password": "123"}'

```
(Copy the JWT token from the response. Ensure no spaces are copied!)
## 3. Success Test (Manager accessing Manager Route):
```
curl -X GET http://localhost:8080/manager/dashboard \
     -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
Output: Welcome Manager! You have access to the financial reports. 
## 4. Hacker Test (Manager accessing Editor Route):
```
curl -X GET http://localhost:8080/editor/dashboard \
     -H "Authorization: Bearer YOUR_TOKEN_HERE"
```
Output: Access Denied: Route requires 'editor' role! Your role is 'manager'. 
if there is any isseu in this copy paste token so please run this command to check everythign is ok:
## Step 1:
```
curl -X POST http://localhost:8080/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"email": "manager99@test.com", "password": "123", "role": "manager"}'
```
## Step 2: Login and auto save token:
```
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"email": "manager99@test.com", "password": "123"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
```
## Step 3: call the dashboard :
```
curl -X GET http://localhost:8080/manager/dashboard \
     -H "Authorization: Bearer $TOKEN"
```
Welcome Manager! You have access to the financial reports.


##  9. 1-Line File & Image Uploads

Handling multipart form data, generating unique filenames, and serving static files (like images) to the browser can take hundreds of lines of code in Rust.

**Don Framework** reduces this to exactly **1 line of code**. It automatically handles file streams, saves them to an `uploads/` directory with unique UUIDs, and serves them statically so your frontend can access them instantly.

### The Code (`src/main.rs`)

Here is a complete, runnable example showing how to enable file uploads in your application:

```rust
use don_core::{DonServer, axum::Router};
use don_core::upload::get_upload_routes;

#[tokio::main]
async fn main() {
    // Load environment variables (DATABASE_URL is required to start the server)
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with File Uploads...");

    // 1. Define your API routes
    let api_routes = Router::new()
        // ✨ THE MAGIC: 1-Line File Upload API!
        .nest("/api/upload", get_upload_routes());

    // 2. Start the Server
    // The framework will automatically serve the uploaded files at http://localhost:8080/uploads/...
    DonServer::new()
        .port(8080)
        .with_routes(api_routes)
        .start()
        .await
        .expect("Server crashed!");
}
```
## Test the File Upload API
Run your server (cargo run) and open a new terminal.

## 1. Create a dummy test file:
```
echo "Hello Don Framework, this is my test file!" > test_image.txt
```
## 2. Upload the file using cURL (Multipart Form Data):
```
curl -X POST http://localhost:8080/api/upload \
     -F "file=@test_image.txt"
```
Output:
```
{
  "message": "Files uploaded successfully!",
  "success": true,
  "urls": [
    "/uploads/38e9e58f-d63a-41e6-8454-ec8083fc31cd.txt"
  ]
}
```

##  11. 100% Dynamic Authentication (Schema-less JSONB)

Most frameworks force you to use `email` or `username` for authentication. **Don Framework** gives you ultimate flexibility. You can use ANY field (e.g., `phone_number`, `cnic`, `school_id`) as your primary login key!

Furthermore, you don't need to define every single user attribute in your database schema. Any extra fields sent during signup are automatically caught and stored in a PostgreSQL `JSONB` column called `metadata`.

### 1. The Database Migration
Your `users` table only needs the primary auth key (e.g., `school`), the `password`, and the `metadata` column.
firstly setup:
```
sqlx database drop -y
sqlx database create
sqlx migrate add flexible_auth_table
```

```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    school VARCHAR(255) UNIQUE NOT NULL, -- Your Custom Auth Key
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user',
    metadata JSONB DEFAULT '{}'          -- All extra fields go here!
);

```
also migrate
```
sqlx migrate run
```
2. The Code (src/main.rs)
Simply tell the DonServer which key to use for authentication via .auth_key()
```rust


use don_core::DonServer;
use don_macros::DonAuth;
// The struct acts as an anchor for the macro. 
// The actual auth key is defined in the server builder below.

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
        //THE MAGIC: Tell the framework to use 'school' for login!
        .auth_key("school") 
        .with_routes(User::get_auth_routes())
        .start()
        .await
        .expect("Server crashed!");
}
```







## Test the Dynamic Auth API
## 1. Signup (With arbitrary extra fields):
Notice how we send username, email, age, and city. The framework extracts school and password, and safely dumps the rest into the metadata JSONB column!

```
curl -X POST http://localhost:8080/auth/signup \
     -H "Content-Type: application/json" \
     -d '{
           "school": "Harvard", 
           "password": "secure123", 
           "username": "cool_dev", 
           "email": "dev@test.com", 
           "age": 22, 
           "city": "Lahore"
         }'
```
## 2. Login (Using the custom Auth Key):
You now log in using school instead of email! The API returns your JWT token along with all your stored metadata.
```
curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"school": "Harvard", "password": "secure123"}'
```

### How to use:
in this you only enter word you want to set a primary field in this===

```
.auth_key("school")  
```
as:
```
.auth_key("email")
----------------
.auth_key("age")
----------------
.auth_key("color")
etc
```
```
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Custom Auth Key...");

    DonServer::new()
        .port(8080)
        //at this line=====
//---------------------------
        .auth_key("school")

//--------------------------
        .with_routes(User::get_auth_routes())
        .start()
        .await
        .expect("Server crashed!");
}
```
```
and also change in database as it:
at this line:
```
school VARCHAR(255) UNIQUE NOT NULL, -- Your Custom Auth Key
```
at:
```
school VARCHAR(255) UNIQUE NOT NULL, 
-------------------------------------------------
email VARCHAR(255) UNIQUE NOT NULL, 
---------------------------------------------
color VARCHAR(255) UNIQUE NOT NULL, 
etc
```

### 🧠 Under the Hood: The Magic of `.auth_key()` and JSONB

**1. What is the purpose of `struct User { pub email: String }`?**
Currently, this struct acts merely as an **"Anchor"** for the `#[derive(DonAuth)]` macro. The macro doesn't actually read the fields inside it! It simply uses the struct's name to generate and attach the `/auth/signup` and `/auth/login` routes. In future versions, we might remove the need for this struct entirely, but for now, it serves as the attachment point.

**2. How does `.auth_key("school")` work?**
When you call `DonServer::new().auth_key("school")`, the framework saves the string `"school"` into the server's **Global State (`AppState`)** (in RAM) right when the server starts.

**3. The JSONB Metadata Magic:**
When a user sends a Signup JSON payload like this:
`{"username": "cool", "email": "a@a.com", "school": "donlee", "password": "123", "city": "Lahore"}`

Here is exactly what the framework does behind the scenes:
1. It asks the `AppState`: *"What is the primary auth key?"* It gets the answer: `"school"`.
2. It extracts `"school": "donlee"` and `"password": "123"` from the JSON payload.
3. It securely hashes the password using **Argon2**.
4. It packs all the remaining JSON fields (`username`, `email`, `city`) into a single box and labels it `metadata`.
5. Finally, it executes the SQL query: `INSERT INTO users (school, password, metadata) VALUES (...)`.

This is exactly why your database migration only needs the `school`, `password`, and `metadata` columns. The framework handles the rest dynamically!






##  How It Works (Under the Hood)
The Don Framework is built on the principles of **Procedural Macros (Meta-Programming)** and the **Active Record Pattern**.

Instead of manually writing repetitive SQL queries, CRUD handlers, and route definitions for every database table, Don Framework leverages Rust's `proc-macro` system to analyze your structs at compile time. It automatically generates the required SQL operations, Axum route handlers, and database bindings, significantly reducing boilerplate while preserving Rust's type safety and performance.

###  Dynamic Authentication Metadata

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






