

# 🦀 Don Framework

**The Django-like, blazing-fast, and developer-friendly web framework for Rust.**

Building web ,,,,,APIs in Rust is incredibly fast and safe, but it often requires writing a lot of boilerplate code (setting up Axum routers, configuring SQLx pools, hashing passwords with Argon2, generating JWTs, etc.). 

**Don Framework** solves this. It acts as a powerful wrapper over `axum` and `sqlx`. By simply adding macros like `#[derive(DonAuth)]` and `#[derive(DonModel)]` to your structs, the framework automatically generates your database queries, API routes, and security guards!



## Features
- **Zero Boilerplate:** Write a struct, get a full API.
- **Auto-Auth:** Instant `/auth/signup` and `/auth/login` routes with Argon2 and JWT.
- **Dynamic Metadata:** Pass any extra JSON fields during signup, and they are safely stored in a Postgres `JSONB` column.
- **Active Record ORM:** Full CRUD API generation for any struct.
- **Admin Guards:** Protect any route with a simple `DonAdmin` extractor.


# 1. Quick Setup

Create a new Rust firstly project:
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
    username VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    age INT NOT NULL,
    city VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL
);
```

```bash
sqlx migrate run
```

# 2. Authentication Made Easy
### 12. Implementing Validation & Auth Hooks

With Don Framework, you don't need to write complex Axum handlers for
authentication. Just define your User struct!

Let's put Declarative Validation (`#[validate]`) and Lifecycle Hooks (`DonAuthHooks`) together in a real-world scenario. 

In this example, we will:
1. Use `#[validate]` to ensure the username is at least 3 characters and the user is 18+.
2. Use `before_signup` to auto-capitalize the user's city.
3. Use `before_login` to block a specific malicious username from attempting to log in.

#### The Code (`src/main.rs`)


```rust
use don_core::{DonServer, axum::Router};
use don_core::traits::DonAuthHooks;
use validator::Validate; 
use don_macros::DonAuth;
use serde::{Deserialize, Serialize};

// ==========================================
// 1. STRICT AUTH MODEL WITH VALIDATION
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth, Validate)]
#[don_auth_key = "username"] 
#[don_validate] // Tells the framework to run validation before signup
pub struct User {
    pub id: i32,
    
    #[validate(length(min = 3, message = "Username must be at least 3 characters!"))]
    pub username: String,
    
    #[validate(length(min = 6, message = "Password must be at least 6 characters!"))]
    pub password: String,
    
    #[validate(range(min = 18, message = "You must be 18+ to signup!"))]
    pub age: i32,
    
    pub city: String,
    pub role: String,
}

// ==========================================
// 2. AUTH LIFECYCLE HOOKS
// ==========================================
impl DonAuthHooks for User {
    
    // Runs BEFORE the user is saved to the database
    async fn before_signup(&mut self) -> Result<(), String> {
        // Data Modification: Auto-format the city name to uppercase
        self.city = self.city.trim().to_uppercase();
        Ok(())
    }

    // Runs BEFORE the login query is executed
    async fn before_login(primary_key: &str) -> Result<(), String> {
        // Security Check: Block a specific username (e.g., a known hacker or banned user)
        if primary_key == "banned_hacker" {
            return Err("Security Alert: Your account has been suspended!".to_string());
        }
        Ok(())
    }
}

// ==========================================
// 3. START THE SERVER
// ==========================================
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Validation & Hooks...");

    DonServer::new()
        .port(8080)
        .auth_key("username") // Set primary login key to 'username'
        .with_routes(User::get_auth_routes())
        .start()
        .await
        .expect("Server crashed!");
}

```
run the server:
```
cargo run
```
#### Test the Validation & Hooks API
Run your server (cargo run) and open a new terminal to run these tests.
##### 1. Test Declarative Validation Failure (Age < 18):
The framework will reject this request before it even reaches the before_signup hook.

```
curl -X POST http://localhost:8080/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"id": 0, "username": "ali123", "password": "secure123", "age": 15, "city": "Lahore", "role": "user"}'
```
Output: Validation Failed: age: You must be 18+ to signup! 
##### 2. Test Successful Signup & Data Modification:
Watch how the city "karachi" is automatically converted to "KARACHI" in the database.
```
curl -X POST http://localhost:8080/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"id": 0, "username": "good_user", "password": "secure123", "age": 25, "city": "karachi", "role": "user"}'
```
Output: {"message":"Account created successfully!","success":true}
##### 3. Test Login Blocker Hook (Hacker Attempt):
Try to log in with the banned username. The before_login hook will intercept and block it.
```
curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username": "banned_hacker", "password": "anypassword"}'
```
Output: Security Alert: Your account has been suspended!
##### 4. Test Successful Login:
```
curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username": "good_user", "password": "secure123"}'
```
Output: Returns the JWT token successfully!

---


#####  Under the Hood: How `DonAuth` Works

You might be wondering: *"Why is there only an `email` field in the `User` struct? Where is the password and ID? And what exactly is `DonServer` doing?"*

Here is the magic explained:

1. **`#[derive(DonAuth)]`:** This is a Rust Procedural Macro. When the compiler sees this attribute, it automatically generates the `/auth/signup` and `/auth/login` Axum handlers and attaches them to your `User` struct. You don't have to write any routing logic.
2. **Where is the Password?** We intentionally omit the `password` field from the struct for security and abstraction. The framework's internal payload parser catches the password directly from the JSON request, hashes it using **Argon2**, and stores it in the database. You never have to handle raw passwords in your application code.
3. **Where is the ID?** The `id` is handled entirely by PostgreSQL (`SERIAL PRIMARY KEY`). The framework abstracts this away so you don't have to manage auto-incrementing integers.
5. **`DonServer`:** This is a powerful wrapper. Instead of manually loading `.env` files, setting up `sqlx::PgPool` connections, and binding `tokio::net::TcpListener`, `DonServer` encapsulates all of this setup. You just call `.start()`, and the framework handles the heavy lifting!

------------------
### Understanding the Magic (Deep Dive into Don Framework)
When you look at a Don Framework model, it looks incredibly simple. But there is a lot of powerful Rust engineering happening behind the scenes. Let's break down exactly what each line and macro does.
##### 1. The Power of #[derive(...)]
```
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth, Validate)]
```
In Rust, #[derive(...)] is a procedural macro that automatically writes code for your struct at compile-time. Here is what each trait does:
##### Debug & Clone:
Standard Rust traits. Debug allows you to print the struct in the terminal for debugging, and Clone allows you to create copies of it in memory.
##### Serialize & Deserialize:
Provided by the serde crate. This allows your Rust struct to automatically convert to JSON (when sending responses) and parse from JSON (when receiving API requests).
##### don_core::sqlx::FromRow:
This tells SQLx how to map a PostgreSQL database row directly into your Rust struct. No manual mapping is required!
##### DonAuth:
Our custom framework macro. It reads your struct and automatically generates the /auth/signup and /auth/login Axum handlers, complete with Argon2 password hashing and JWT generation.
##### Validate:
Provided by the validator crate. It enables declarative validation, allowing you to add rules like #[validate(length(min = 3))] directly on your fields.
#### 2. #[don_auth_key = "username"]
By default, most frameworks force you to use email for authentication. The Don Framework is flexible.
When you attach #[don_auth_key = "username"] (or "phone", "cnic", etc.) to your struct, you are telling the DonAuth macro: "Do not look for an email. Use this specific field as the primary login ID." The framework will dynamically adjust the SQL queries and JSON payloads to expect this key during login and signup.
##### Note:
see point 2 full detailed on next .
##### 3. #[don_validate]
This is a custom flag for the DonAuth macro. When you add #[don_validate] above your struct, you are instructing the framework's auto-generated signup handler to pause and run the validation rules before touching the database. If any field fails the validation (e.g., age is less than 18), it instantly aborts the process and returns a 400 Bad Request with the exact error messages.
##### 4. DonAuthHooks (The Lifecycle Interceptor)
Sometimes, declarative validation (like min/max length) isn't enough. You might need complex business logic. That's where DonAuthHooks comes in.
By implementing this trait, you get access to two powerful lifecycle events:
##### before_signup(&mut self):
Runs right before the user is saved to the database. You can use this to run complex validations (e.g., checking if an email domain is allowed) or to mutate data (e.g., auto-capitalizing a city name).
##### before_login(primary_key: &str): 
Runs right before the login query executes. This is highly useful for security. For example, you can check a Redis cache to see if this user has failed 3 login attempts and block them temporarily to prevent Brute-Force attacks.
#### 5. DonGuard (Role-Based Access Control)
```
#[derive(DonGuard)]
#[don_role = "manager"]
pub struct ManagerGuard;
```
Security in standard APIs requires writing repetitive middleware to check JWT tokens and user roles. DonGuard automates this.
When you define an empty struct and attach this macro, it generates an Axum Extractor (Middleware). When you add _guard: ManagerGuard as a parameter to any API route, the framework will intercept the HTTP request, decode the JWT token, verify that the user's role exactly matches "manager", and block unauthorized users with a 403 Forbidden response.

-----------------------------------------------------------------------------------

#### Declarative Validation vs. Lifecycle Hooks
In Don Framework, you have two powerful ways to control your data: Declarative Validation (#[validate(...)]) and Lifecycle Hooks (DonAuthHooks). Understanding the difference between them is key to writing clean, enterprise-grade code.
##### 1. Declarative Validation (#[validate(...)])
This is provided by the validator crate. It is used for simple, read-only checks. It ensures the data looks correct before it even reaches your business logic.
##### length(min = X, max = Y):
Used for String fields to check character count.
##### range(min = X, max = Y):
Used for numeric fields (like i32) to check value limits.
##### message = "...": 
The custom error message sent back to the user if the check fails.
What else can it do?
Besides length and range, the validator supports many other rules:
##### #[validate(email)]:
Ensures the string is a valid email format (e.g., test@test.com).
##### #[validate(url)]:
Ensures the string is a valid web link.
##### #[validate(regex(path = "CUSTOM_REGEX"))]:
Checks the string against a custom Regular Expression pattern.
##### #[validate(must_match = "other_field")]:
Ensures two fields match (perfect for "Confirm Password" fields).
#### 2. Lifecycle Hooks (DonAuthHooks)
While #[validate] is great for checking data, it cannot change the data, and it cannot run complex async tasks (like checking a database). That is where DonAuthHooks comes in.
##### What does impl mean?
In Rust, impl stands for "implement". Because Rust does not have Classes (like Python or Java), impl is the keyword used to attach functions, logic, or Traits to a struct.
##### How to use before_signup:
##### Notice the signature:
async fn before_signup(&mut self). The &mut self means you have a mutable reference to the entire struct.
##### Data Mutation:
Because it is mutable, you can alter the data before it saves. You access a field using self.field_name and change it like this: self.city = self.city.trim().to_uppercase();.
##### Complex Logic:
You can write if-else statements, make external API calls, or check the database. If something is wrong, simply return an error: return Err("Custom Error".to_string());.
------------------------------------------------------
#### IN CODE:
##### How to use self in before_signup
Inside the before_signup hook, you have mutable access to your entire struct using &mut self. This allows you to modify, format, or calculate fields before they are saved to the database.
The syntax is simply: self.field_name = your_logic;
Examples of Data Modification:
```
impl DonAuthHooks for User {
    async fn before_signup(&mut self) -> Result<(), String> {
        
        // 1. CITY: Always capitalize the first letter (e.g., "lahore" -> "Lahore")
        if let Some(first_char) = self.city.chars().next() {
            self.city = format!("{}{}", first_char.to_uppercase(), &self.city[1..]);
        }

        // 2. ROLE: Force the role to always be lowercase and remove extra spaces
        self.role = self.role.trim().to_lowercase();

        // 3. USERNAME: Ensure the username has no spaces
        self.username = self.username.replace(" ", "_");

        // 4. AGE LOGIC: If age is exactly 18, auto-assign a specific role
        if self.age == 18 {
            self.role = "new_adult".to_string();
        }

        // 5. PASSWORD: Trim accidental spaces before the framework hashes it
        self.password = self.password.trim().to_string();

        Ok(())
    }
}
```
If you do not need any custom data modification, you can simply leave the function empty. The framework will just proceed to save the data:

```
impl DonAuthHooks for User {
    async fn before_signup(&mut self) -> Result<(), String> {
        Ok(()) // Do nothing, just proceed
    }
}
```
 
#### How to use before_login:
##### Notice the signature:
async fn before_login(_primary_key: &str). Here, you don't have self because the user hasn't logged in yet! You only have their login ID (e.g., username or email). You can use this to block hackers. For example, if a user has failed to login 5 times, you can check their _primary_key against a Redis cache and return an error to block them.

##### IN CODE:
##### How to use primary_key in before_login
In the before_login hook, you do not have access to self. Why? Because the user has not been authenticated yet! The framework only passes the primary_key (which is the email, username, or whatever you set in .auth_key()).
You can use this primary_key to run security checks before the framework even touches the database.
The syntax is simply: if primary_key == logic { return Err(...) }
Examples of Pre-Login Security Logic:
```
impl DonAuthHooks for User {
    async fn before_login(primary_key: &str) -> Result<(), String> {
        
        // 1. Block a specific banned user
        if primary_key == "banned_hacker" {
            return Err("Security Alert: Your account has been suspended!".to_string());
        }

        // 2. Block temporary or fake email domains
        if primary_key.ends_with("@tempmail.com") {
            return Err("Security Alert: Temporary emails are not allowed!".to_string());
        }

        // 3. Prevent extremely long inputs (Basic DoS Protection)
        if primary_key.len() > 100 {
            return Err("Invalid login ID format.".to_string());
        }

        // 4. Custom Database/Redis Check (Pseudo-code)
        // if check_redis_for_brute_force(primary_key).await {
        //     return Err("Too many failed attempts. Try again in 5 minutes.".to_string());
        // }

        Ok(())
    }
}
```
If you do not need any custom data modification, you can simply leave the function empty. The framework will just proceed to save the data:

```
impl DonAuthHooks for User {
  async fn before_login(primary_key: &str) -> Result<(), String> {
        Ok(()) // Do nothing, just proceed
    }
}
```


##### 3. The Core Difference
#[validate] is a Bouncer at the door. It only checks IDs (data format). It cannot change your clothes (mutate data).
DonAuthHooks is the Manager inside the club. It can change things, run complex background checks, and make the final decision.
##### 4. How to Bypass Hooks
Because the Don Framework relies heavily on security, it forces you to implement DonAuthHooks (or DonHooks for CRUD) on your models.
However, if your app is simple and you only want to use #[validate] without any custom data mutation or login blocking, you can simply provide an empty implementation. This satisfies the Rust compiler without adding extra logic:

```
// I only want declarative validation, no custom hooks needed!
impl DonAuthHooks for User {}
```
By doing this, the framework will still run your #[validate] rules, but it will skip the custom hook logic and proceed directly to the database operations.











# 3. Route Protection & Admin Guards

Don Framework provides a built-in, zero-configuration security guard (`DonAdmin`) to protect your sensitive routes. Only users with the `admin` role (like the Superuser defined in your `.env`) can access these endpoints.

### Protecting Any Custom Route

You don't need to write complex middleware. Simply add `_admin: DonAdmin` as a parameter to your Axum handler. The framework will automatically intercept the request, verify the JWT token, check the user's role, and block unauthorized access!

##### STEP 1:
##### setup.env
```
DATABASE_URL=postgres://postgres:password@localhost:5432/don_db
JWT_SECRET=your_super_secret_jwt_key_12345

# Define your Admin/Superuser credentials here
SUPERUSER_ID=admin_boss
SUPERUSER_PASSWORD=supersecret
```

##### Database setup:
delete ohers files in migration than  run this in terminal:
```
sqlx database drop -y
sqlx database create
sqlx migrate add admin_logic_tables

```
pase this to new .sql file
```
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) DEFAULT 'user',
    is_suspended BOOLEAN DEFAULT FALSE 
);
```
run migration
```
sqlx migrate run
```
##### Step 2 Update your `src/main.rs`:

```rust


use don_core::{
    DonServer, AppState,DonAuthHooks, DonAdmin, 
    axum::{Router, extract::{State, Path}, Json, routing::{get, put}}
};
use don_macros::DonAuth;
use serde::{Deserialize, Serialize};

// ==========================================
// 1. STRICT AUTH MODEL
// ==========================================
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth)]
#[don_auth_key = "username"] 
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
    pub is_suspended: bool, 
}

//DonAuthHppls is empty if you like than add any logic and function
impl DonAuthHooks for User {}
// ==========================================
// 2. ADMIN LOGIC (CUSTOM HANDLERS)
// ==========================================

// A. Get All Users (Admin Only)
async fn get_all_users(
    _admin: DonAdmin, // 1. Guard: only admin allowed
    State(state): State<AppState>, // 2. Database access
) -> Result<Json<Vec<User>>, String> {
    
    // Custom SQL Query to fetch all users
    let users = don_core::sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY id ASC")
        .fetch_all(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(users))
}

// B. Suspend a User (Admin Only)
async fn suspend_user(
    _admin: DonAdmin, // Guard
    State(state): State<AppState>, // Database access
    Path(user_id): Path<i32>, // URL  User ID (e.g., /admin/suspend/5)
) -> Result<Json<don_core::serde_json::Value>, String> {
    
    // Custom SQL Query to update user status
    don_core::sqlx::query("UPDATE users SET is_suspended = TRUE WHERE id = $1")
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(don_core::serde_json::json!({
        "success": true,
        "message": format!("User ID {} has been suspended successfully!", user_id)
    })))
}

// ==========================================
// 3. START THE SERVER
// ==========================================
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Admin Logic...");

    // Admin Routes Setup
    let admin_routes = Router::new()
        .route("/admin/users", get(get_all_users))
        .route("/admin/suspend/:id", put(suspend_user)); // PUT request for updating

    DonServer::new()
        .port(8080)
        .auth_key("username")
        .with_routes(User::get_auth_routes())
        .with_routes(admin_routes) // Inject Admin routes
        .start()
        .await
        .expect("Server crashed!");
}
```
than run :
```
cargo run
```
##### Step 3: The Final Test (Terminal cURL Commands)
##### 1. Create a Dummy User (ID 1):
```
curl -X POST http://localhost:8080/auth/signup \
     -H "Content-Type: application/json" \
     -d '{"id": 0, "username": "bad_guy", "password": "123", "role": "user", "is_suspended": false}'
```
##### 2. Login as SUPERUSER (Admin) to get the Token:
```
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username": "admin_boss", "password": "supersecret"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
```
##### 3. Admin Logic Test 1: Get All Users
```
curl -X GET http://localhost:8080/admin/users \
     -H "Authorization: Bearer $TOKEN"
```
Output: Tumhein return JSON Array [...] where all the user shown.
#### 4. Admin Logic Test 2: Suspend the User (ID 1)
```
curl -X PUT http://localhost:8080/admin/suspend/1 \
     -H "Authorization: Bearer $TOKEN"
```
Output: {"message":"User ID 1 has been suspended successfully!","success":true}


---------------------------------------------------
### Others:
if you not like to set extra logic in code you simple setup this only admin setup:
main.rs
```
use don_core::{DonServer, axum::Router, DonAdmin,DonAuthHooks};
use don_macros::DonAuth;
use serde::{Deserialize, Serialize};


#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth)]
#[don_auth_key = "username"] // Login will be done via 'username'
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub role: String,
}
impl DonAuthHooks for User {}
// 2. Create a Protected Route
// Adding `_admin: DonAdmin` makes this route 100% secure!
async fn secure_dashboard(_admin: DonAdmin) -> &'static str {
    "Welcome to the Secure Dashboard! You have Superuser (Admin) access. "
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework...");

    // 3. Define your custom routes
    let custom_routes = Router::new()
        .route("/admin/dashboard", don_core::axum::routing::get(secure_dashboard));

    // 4. Start the Server
    DonServer::new()
        .port(8080)
        .auth_key("username") // Tell the framework to use 'username' for login
        .with_routes(User::get_auth_routes())
        .with_routes(custom_routes) // Inject the protected route
        .start()
        .await
        .expect("Server crashed!");
}
```

# RBAC (Role-Based Access Control)
 setup migration
```

sqlx database drop -y
sqlx database create

sqlx migrate add company_rbac_tables
```
paste in new sql migration file
.sql
```
-- Users table with Role and Salary
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(255) UNIQUE NOT NULL,
    password VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL, -- manager, editor, finance, user
    salary INT DEFAULT 0
);

-- Articles table for the Editor to manage
CREATE TABLE articles (
    id SERIAL PRIMARY KEY,
    title VARCHAR(255) NOT NULL,
    content TEXT NOT NULL,
    is_published BOOLEAN DEFAULT FALSE
);
```
run migration
```
sqlx migrate run
```
than paste code in main.rs file
```
// example_app/src/main.rs

use don_core::{
    DonServer, AppState, 
    axum::{Router, extract::{State, Path}, Json, routing::{get, put, post}}
};
use don_core::traits::{DonAuthHooks, DonHooks};
use validator::Validate;
use don_macros::{DonAuth, DonGuard, DonModel};
use serde::{Deserialize, Serialize};

// ==========================================
// 1. MODELS & VALIDATION
// ==========================================

#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonAuth, Validate)]
#[don_auth_key = "username"] 
#[don_validate]
pub struct User {
    pub id: i32,
    #[validate(length(min = 3, message = "Username must be at least 3 characters"))]
    pub username: String,
    #[validate(length(min = 4, message = "Password must be at least 4 characters"))]
    pub password: String,
    pub role: String,
    pub salary: i32,
}

// Auth Hooks (Signup se pehle check karna)
impl DonAuthHooks for User {
    async fn before_signup(&mut self) -> Result<(), String> {
        // Validation:only these 4 allowed
        let valid_roles = ["manager", "editor", "finance", "user"];
        if !valid_roles.contains(&self.role.as_str()) {
            return Err("Invalid Role! Must be manager, editor, finance, or user.".to_string());
        }
        Ok(())
    }
    async fn before_login(primary_key: &str) -> Result<(), String> {
        if primary_key == "hacker" {
            return Err("Security Alert: You are banned!".to_string());
        }
        Ok(())
    }
}

// Article Model (For Editor)
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Article {
    pub id: i32,
    pub title: String,
    pub content: String,
    pub is_published: bool,
}
impl DonHooks for Article {}

// ==========================================
// 2. DEFINE RBAC GUARDS (1-Line Magic)
// ==========================================

#[derive(DonGuard)]
#[don_role = "manager"]
pub struct ManagerGuard;

#[derive(DonGuard)]
#[don_role = "editor"]
pub struct EditorGuard;

#[derive(DonGuard)]
#[don_role = "finance"]
pub struct FinanceGuard;

// ==========================================
// 3. ROLE-SPECIFIC LOGIC & DASHBOARDS
// ==========================================

// A. MANAGER LOGIC: Can see total company salary expense
async fn manager_dashboard(
    _guard: ManagerGuard, // Protected!
    State(state): State<AppState>,
) -> Result<Json<don_core::serde_json::Value>, String> {
    
    // SQL Aggregation Query
    let row = don_core::sqlx::query!("SELECT SUM(salary) as total_salary FROM users")
        .fetch_one(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    let total = row.total_salary.unwrap_or(0);

    Ok(Json(don_core::serde_json::json!({
        "message": "Welcome Manager! Here is the company report.",
        "total_salary_expense": total
    })))
}

// B. EDITOR LOGIC: Can publish an article
async fn editor_publish_article(
    _guard: EditorGuard, // Protected!
    State(state): State<AppState>,
    Path(article_id): Path<i32>,
) -> Result<Json<don_core::serde_json::Value>, String> {
    
    don_core::sqlx::query("UPDATE articles SET is_published = TRUE WHERE id = $1")
        .bind(article_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(don_core::serde_json::json!({
        "success": true,
        "message": format!("Article {} has been published to the public!", article_id)
    })))
}

// C. FINANCE LOGIC: Can update a user's salary
#[derive(Deserialize)]
struct SalaryPayload { salary: i32 }

async fn finance_update_salary(
    _guard: FinanceGuard, // Protected!
    State(state): State<AppState>,
    Path(user_id): Path<i32>,
    Json(payload): Json<SalaryPayload>,
) -> Result<Json<don_core::serde_json::Value>, String> {
    
    don_core::sqlx::query("UPDATE users SET salary = $1 WHERE id = $2")
        .bind(payload.salary)
        .bind(user_id)
        .execute(&state.db)
        .await
        .map_err(|e| e.to_string())?;

    Ok(Json(don_core::serde_json::json!({
        "success": true,
        "message": format!("Salary for User {} updated to ${}", user_id, payload.salary)
    })))
}

// ==========================================
// 4. START THE SERVER
// ==========================================
#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();
    println!("Starting Don Framework with Advanced RBAC...");

    let rbac_routes = Router::new()
        // Manager Route
        .route("/api/manager/report", get(manager_dashboard))
        // Editor Route
        .route("/api/editor/publish/:id", put(editor_publish_article))
        // Finance Route
        .route("/api/finance/salary/:id", put(finance_update_salary))
        // Standard CRUD for Articles
        .nest("/api/articles", Article::get_api_routes());

    DonServer::new()
        .port(8080)
        .auth_key("username")
        .with_routes(User::get_auth_routes())
        .with_routes(rbac_routes)
        .start()
        .await
        .expect("Server crashed!");
}
```
than
```
cargo run
```
than test it:

#### The Ultimate RBAC Test (Terminal Commands)

##### 1. Create Users (Manager, Editor, Finance):
```
# Create Manager
curl -X POST http://localhost:8080/auth/signup -H "Content-Type: application/json" -d '{"id":0, "username": "boss_man", "password": "1243", "role": "manager", "salary": 10000}'

# Create Editor
curl -X POST http://localhost:8080/auth/signup -H "Content-Type: application/json" -d '{"id":0, "username": "writer_pro", "password": "1243", "role": "editor", "salary": 5000}'

# Create Finance
curl -X POST http://localhost:8080/auth/signup -H "Content-Type: application/json" -d '{"id":0, "username": "money_guy", "password": "1243", "role": "finance", "salary": 8000}'
```
##### 2. Login and Save Tokens
```
MANAGER_TOKEN=$(curl -s -X POST http://localhost:8080/auth/login -H "Content-Type: application/json" -d '{"username": "boss_man", "password": "1243"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

EDITOR_TOKEN=$(curl -s -X POST http://localhost:8080/auth/login -H "Content-Type: application/json" -d '{"username": "writer_pro", "password": "1243"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)

FINANCE_TOKEN=$(curl -s -X POST http://localhost:8080/auth/login -H "Content-Type: application/json" -d '{"username": "money_guy", "password": "1243"}' | grep -o '"token":"[^"]*"' | cut -d'"' -f4)
```

##### 3. Test 1: Manager Logic (Get Total Salary Expense)
```
curl -X GET http://localhost:8080/api/manager/report -H "Authorization: Bearer $MANAGER_TOKEN"
```
Output: {"message":"Welcome Manager! Here is the company report.","total_salary_expense":23000}
##### 4. Test 2: Hacker Attempt (Editor trying to view Manager Report)
```
curl -X GET http://localhost:8080/api/manager/report -H "Authorization: Bearer $EDITOR_TOKEN"
```
Output: Access Denied: Route requires 'manager' role! Your role is 'editor'.  (Blocked!)
##### 5. Test 3: Editor Logic (Create & Publish Article)
```

curl -X POST http://localhost:8080/api/articles -H "Content-Type: application/json" -d '{"id":0, "title": "Rust is Awesome", "content": "Learning Don Framework", "is_published": false}'


curl -X PUT http://localhost:8080/api/editor/publish/1 -H "Authorization: Bearer $EDITOR_TOKEN"
```
Output: {"message":"Article 1 has been published to the public!","success":true}
##### 6. Test 4: Finance Logic (Update Editor's Salary)
```

curl -X PUT http://localhost:8080/api/finance/salary/2 -H "Content-Type: application/json" -H "Authorization: Bearer $FINANCE_TOKEN" -d '{"salary": 15000}'
```

#### Important terms and methods to use

In real-world enterprise applications, having a single "Admin" is never enough. You usually have multiple roles like `manager`, `editor`, `finance`, or `moderator`. 

Writing custom middleware to decode JWTs and verify specific roles for every single endpoint can lead to massive boilerplate. **Don Framework** solves this elegantly using the `#[derive(DonGuard)]` macro.

##### Step 1: Define Your Custom Guards

To create a security guard for a specific role, you simply define an empty struct and attach the `DonGuard` macro along with the `#[don_role = "..."]` attribute.

```rust
use don_macros::DonGuard;

// 1. Creates a Guard that ONLY allows users with the "manager" role
#[derive(DonGuard)]
#[don_role = "manager"]
pub struct ManagerGuard;

// 2. Creates a Guard that ONLY allows users with the "editor" role
#[derive(DonGuard)]
#[don_role = "editor"]
pub struct EditorGuard;

// 3. Creates a Guard that ONLY allows users with the "finance" role
#[derive(DonGuard)]
#[don_role = "finance"]
pub struct FinanceGuard;

```
##### Under the Hood:
When the compiler sees #[derive(DonGuard)], it automatically generates an Axum Extractor (Middleware) for that struct. It writes the complex logic to extract the Bearer Token from the HTTP headers, decode the JWT, check the role inside the payload, and instantly reject the request with a 403 Forbidden if the roles do not match.
##### Step 2: Protect Your Routes
Now that your guards are defined, how do you protect a route?
It is incredibly simple: Just add the Guard as a parameter to your route handler function!
```
// This route is now 100% protected. 
// If a user without the "manager" role tries to access it, the framework blocks them before the function even executes!
async fn manager_dashboard(_guard: ManagerGuard) -> &'static str {
    "Welcome Manager! Here is the highly confidential financial report. 📊"
}

// Only users with the "editor" role can access this.
async fn create_article(_guard: EditorGuard) -> &'static str {
    "Welcome Editor! You can now write and publish articles. 📝"
}

// Only users with the "finance" role can access this.
async fn update_salaries(_guard: FinanceGuard) -> &'static str {
    "Welcome Finance Team! You can now process the payroll. 💰"
}

```
##### Step 3: Mount the Routes
Finally, mount these handlers to your Axum router just like any standard route:
```
use don_core::{DonServer, axum::Router};

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    // The routes are protected at the handler level, so you just route them normally!
    let rbac_routes = Router::new()
        .route("/api/manager/dashboard", don_core::axum::routing::get(manager_dashboard))
        .route("/api/editor/article", don_core::axum::routing::post(create_article))
        .route("/api/finance/payroll", don_core::axum::routing::put(update_salaries));

    DonServer::new()
        .port(8080)
        .auth_key("username")
        .with_routes(rbac_routes)
        .start()
        .await
        .expect("Server crashed!");
}
```
##### Why is this awesome?
Zero Boilerplate: No need to write manual JWT decoding logic.
Highly Modular: You can create as many roles as your application needs.
Secure by Default: The request is intercepted and verified before your business logic runs.
##### impl
if you write other logic you can write this in 
```
impl DonHooks for Article {}
```

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
// src/main.rs

use don_core::{DonServer, axum::Router, DonHooks}; 
use don_macros::DonModel;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, 
    pub name: String,
    pub price: i32
    
}


impl DonHooks for Product {}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let api_routes = Router::new()
        .nest("/api/products", Product::get_api_routes());

    DonServer::new()
        .port(8080)
        .with_routes(api_routes) 
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


### Impotant Terms:
## 🧠 Understanding the Magic (Deep Dive)

If you are wondering how the Don Framework achieves so much with so little code, here is a detailed breakdown of the syntax and the underlying mechanics.

### 1. The Imports
```rust
use don_core::{DonServer, axum::Router, DonHooks};
use don_macros::DonModel;
```
### DonHooks:
This is a Rust trait provided by the core framework. It allows you to intercept data right before it is saved or updated in the database.
### DonModel:
This is the Procedural Macro (the "Magic Factory"). When you import this, you bring in the engine that reads your structs and writes hundreds of lines of SQLx and Axum code for you in the background.
## 2. The Model Definition
```
#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, 
    pub name: String,
    pub price: i32,
}
```
### don_core::sqlx::FromRow:
This tells the SQLx database driver how to take a row from the PostgreSQL database and convert it directly into this Rust struct.
### DonModel:
This triggers our custom macro. It looks at your struct name (Product) and assumes your database table is named products. It also looks at your fields (id, name, price) and generates the exact INSERT, SELECT, UPDATE, and DELETE SQL queries.
### Important Rule (Struct vs. SQL Table):
The fields in your Rust struct must exactly match the columns in your SQL table. If you want to add a new field (e.g., description), you simply update both:
Add pub description: String to your Rust struct.
Add description VARCHAR(255) to your PostgreSQL database migration.
The DonModel macro will automatically adapt and update all the underlying SQL queries for you!
example:
### sql
```
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    price INT NOT NULL,
    school VARCHAR(255) NOT NULL,
    age INT NOT NULL

);
```
### main.rs
```

#[derive(Debug, Clone, Serialize, Deserialize, don_core::sqlx::FromRow, DonModel)]
pub struct Product {
    pub id: i32, 
    pub name: String,
    pub price: i32,
    pub school:String,
    pub age :i32
}
```


## 3. Implementing Lifecycle Hooks
```
impl DonHooks for Product {}
```
Because the DonModel macro automatically calls before_save and before_update before interacting with the database, Rust requires you to implement the DonHooks trait for your struct.
### What goes inside the {}?
If you leave it empty (as shown above), the framework applies the default behavior (it just saves the data). However, you can open the brackets and write custom validation or modification logic:
```
impl DonHooks for Product {
    async fn before_save(&mut self) -> Result<(), String> {
        if self.price < 0 {
            return Err("Price cannot be negative!".to_string());
        }
        self.name = self.name.to_uppercase(); // Modify data before saving
        Ok(())
    }
}
```
### 4. Routing and Nesting
```
let api_routes = Router::new()
    .nest("/api/products", Product::get_api_routes());
```
### Product::get_api_routes():
This function is fixed and automatically generated by the DonModel macro. It bundles the 5 RESTful endpoints (Create, Read All, Read One, Update, Delete) into a single Axum Router.
### .nest():
This is an Axum feature that acts like a "folder" for your routes.
### Is "/api/products" fixed?
No! 
This is entirely up to you. You can change it to "/store/items" or "/v1/catalog". The .nest() function simply prefixes whatever path you choose to the auto-generated CRUD routes.
### 5. Starting the Server
```
DonServer::new()
    .port(8080)
    .with_routes(api_routes)
    .start()
    .await

```
### DonServer: 
This is the framework's wrapper around the Axum server, Tokio TCP listener, and SQLx Connection Pool. It hides all the complex boilerplate.
### .with_routes(api_routes):
This takes the routes you nested above and officially attaches them to the running server. You can chain .with_routes() multiple times to attach Auth routes, Admin routes, and CRUD routes seamlessly.



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






