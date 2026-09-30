use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// ==========================================
// 1. DON AUTH MACRO (Strict Auth + Validation)
// ==========================================
#[proc_macro_derive(DonAuth, attributes(don_auth_key, don_validate))]
pub fn don_auth_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let table_name = format!("{}s", struct_name.to_string().to_lowercase());

    let mut auth_key = "email".to_string();
    let mut should_validate = false; 

    for attr in &ast.attrs {
        if attr.path().is_ident("don_auth_key") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        auth_key = lit_str.value();
                    }
                }
            }
        }
        if attr.path().is_ident("don_validate") {
            should_validate = true;
        }
    }

    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(ref fields), .. }) = ast.data {
        fields.named.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>()
    } else {
        panic!("DonAuth only works with named structs!");
    };

    if !fields.iter().any(|f| f.to_string() == "password") {
        panic!("DonAuth requires a 'password' field in your struct!");
    }

    let has_role = fields.iter().any(|f| f.to_string() == "role");



//  Check if struct has 'is_suspended'
    let has_suspended = fields.iter().any(|f| f.to_string() == "is_suspended");
    let suspended_check = if has_suspended {
        quote! {
            if user.is_suspended {
                return Err((don_core::axum::http::StatusCode::FORBIDDEN, "Your account is suspended! Please contact the administrator.".to_string()));
            }
        }
    } else {
        quote! {}
    };













    let role_assignment = if has_role {
        quote! { user.role.clone() }
    } else {
        quote! { "user".to_string() }
    };

    let insert_fields: Vec<_> = fields.iter().filter(|f| f.to_string() != "id").collect();
    let bind_marks = (1..=insert_fields.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
    let insert_columns = insert_fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
    
    let insert_query = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING *", table_name, insert_columns, bind_marks);
    let select_query = format!("SELECT * FROM {} WHERE {} = $1", table_name, auth_key);

    let binds = insert_fields.iter().map(|f| quote! { .bind(payload.#f.clone()) });

    let validation_code = if should_validate {
        quote! {
            if let Err(e) = validator::Validate::validate(&payload) {
                return Err((don_core::axum::http::StatusCode::BAD_REQUEST, format!("Validation Failed: {}", e)));
            }
        }
    } else {
        quote! {} 
    };

    let expanded = quote! {
        impl #struct_name {
            pub async fn api_signup(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::Json(mut payload): don_core::axum::Json<Self>,
            ) -> Result<don_core::axum::Json<don_core::serde_json::Value>, (don_core::axum::http::StatusCode, String)> {
                
                #validation_code

                if let Err(e) = don_core::traits::DonAuthHooks::before_signup(&mut payload).await {
                    return Err((don_core::axum::http::StatusCode::BAD_REQUEST, e));
                }

                let salt = don_core::argon2::password_hash::SaltString::generate(&mut don_core::argon2::password_hash::rand_core::OsRng);
                let argon2 = don_core::argon2::Argon2::default();
                let hash = don_core::argon2::PasswordHasher::hash_password(
                    &argon2,
                    payload.password.as_bytes(),
                    &salt
                ).map_err(|_| (don_core::axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Hashing failed".to_string()))?.to_string();
                
                payload.password = hash;

                let result = don_core::sqlx::query_as::<_, Self>(#insert_query)
                    #(#binds)*
                    .fetch_one(&state.db)
                    .await;

                match result {
                    Ok(_) => Ok(don_core::axum::Json(don_core::serde_json::json!({
                        "success": true,
                        "message": "Account created successfully!"
                    }))),
                    Err(e) => {
                        if e.to_string().contains("duplicate key") {
                            Err((don_core::axum::http::StatusCode::CONFLICT, format!("{} already exists!", #auth_key)))
                        } else {
                            Err((don_core::axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
                        }
                    }
                }
            }

            pub async fn api_login(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::Json(payload): don_core::axum::Json<don_core::serde_json::Value>,
            ) -> Result<don_core::axum::Json<don_core::serde_json::Value>, (don_core::axum::http::StatusCode, String)> {
                
                let auth_val = payload.get(#auth_key).and_then(|v| v.as_str()).unwrap_or_default();
                let password_val = payload.get("password").and_then(|v| v.as_str()).unwrap_or_default();

                if auth_val.is_empty() || password_val.is_empty() {
                    return Err((don_core::axum::http::StatusCode::BAD_REQUEST, format!("{} and password are required", #auth_key)));
                }

                if let Err(e) = <Self as don_core::traits::DonAuthHooks>::before_login(auth_val).await {
                    return Err((don_core::axum::http::StatusCode::FORBIDDEN, e));
                }

                let super_id = std::env::var("SUPERUSER_ID").unwrap_or_default();
                let super_pass = std::env::var("SUPERUSER_PASSWORD").unwrap_or_default();

                let (role, final_id) = if auth_val == super_id && password_val == super_pass {
                    ("admin".to_string(), super_id.to_string())
                } else {
                    let user = don_core::sqlx::query_as::<_, Self>(#select_query)
                        .bind(auth_val)
                        .fetch_optional(&state.db)
                        .await
                        .map_err(|e| (don_core::axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?
                        .ok_or((don_core::axum::http::StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()))?;

                    let parsed_hash = don_core::argon2::password_hash::PasswordHash::new(&user.password)
                        .map_err(|_| (don_core::axum::http::StatusCode::INTERNAL_SERVER_ERROR, "Invalid hash".to_string()))?;

                    if !don_core::argon2::PasswordVerifier::verify_password(&don_core::argon2::Argon2::default(), password_val.as_bytes(), &parsed_hash).is_ok() {
                        return Err((don_core::axum::http::StatusCode::UNAUTHORIZED, "Invalid credentials".to_string()));
                    }
                    #suspended_check

                    (#role_assignment, auth_val.to_string())
                };

                let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET missing");
                let expiration = don_core::chrono::Utc::now().checked_add_signed(don_core::chrono::Duration::hours(24)).unwrap().timestamp() as usize;

                #[derive(don_core::serde::Serialize)]
                struct Claims { sub: String, role: String, exp: usize }

                let claims = Claims { sub: final_id, role, exp: expiration };
                let token = don_core::jsonwebtoken::encode(
                    &don_core::jsonwebtoken::Header::default(),
                    &claims,
                    &don_core::jsonwebtoken::EncodingKey::from_secret(secret.as_bytes())
                ).unwrap();

                Ok(don_core::axum::Json(don_core::serde_json::json!({
                    "success": true,
                    "message": "Login successful!",
                    "token": token
                })))
            }

            pub fn get_auth_routes() -> don_core::axum::Router<don_core::server::AppState> {
                don_core::axum::Router::new()
                    .route("/auth/signup", don_core::axum::routing::post(Self::api_signup))
                    .route("/auth/login", don_core::axum::routing::post(Self::api_login))
            }
        }
    };

    TokenStream::from(expanded)
}

// don_macros/src/lib.rs (Sirf DonModel wala hissa replace karo)

// ==========================================
// 2. DON MODEL MACRO (CRUD + Pagination + Hooks + VALIDATION)
// ==========================================
#[proc_macro_derive(DonModel, attributes(don_validate))]
pub fn don_model_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    let table_name = format!("{}s", struct_name.to_string().to_lowercase());

    //  Check if user wants Validation
    let mut should_validate = false;
    for attr in &ast.attrs {
        if attr.path().is_ident("don_validate") {
            should_validate = true;
        }
    }

    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(ref fields), .. }) = ast.data {
        fields.named.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>()
    } else {
        panic!("DonModel only works with named structs!");
    };

    let insert_fields: Vec<_> = fields.iter().filter(|f| f.to_string() != "id").collect();
    let bind_marks = (1..=insert_fields.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
    let insert_columns = insert_fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
    
    let insert_query = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING *", table_name, insert_columns, bind_marks);
    let select_all_query = format!("SELECT * FROM {} ORDER BY id DESC LIMIT $1 OFFSET $2", table_name);
    let select_by_id_query = format!("SELECT * FROM {} WHERE id = $1", table_name);
    let delete_query = format!("DELETE FROM {} WHERE id = $1", table_name);
    
    let update_sets = insert_fields.iter().enumerate().map(|(i, f)| format!("{} = ${}", f, i + 1)).collect::<Vec<_>>().join(", ");
    let update_query = format!("UPDATE {} SET {} WHERE id = ${} RETURNING *", table_name, update_sets, insert_fields.len() + 1);

    let binds = insert_fields.iter().map(|f| quote! { .bind(self.#f.clone()) });
    let update_binds = binds.clone();

    
    let validation_code = if should_validate {
        quote! {
            if let Err(e) = validator::Validate::validate(&payload) {
                return Err((don_core::axum::http::StatusCode::BAD_REQUEST, format!("Validation Failed: {}", e)));
            }
        }
    } else {
        quote! {} 
    };

    let expanded = quote! {
        impl #struct_name {
            pub async fn save(&mut self, pool: &don_core::sqlx::PgPool) -> Result<Self, String> {
                don_core::DonHooks::before_save(self).await?;
                let result = don_core::sqlx::query_as::<_, Self>(#insert_query)
                    #(#binds)*
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(result)
            }

            pub async fn update(&mut self, pool: &don_core::sqlx::PgPool, id: i32) -> Result<Self, String> {
                don_core::DonHooks::before_update(self).await?;
                let result = don_core::sqlx::query_as::<_, Self>(#update_query)
                    #(#update_binds)*
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(result)
            }

            pub async fn find_all(pool: &don_core::sqlx::PgPool, page: i64, limit: i64) -> Result<Vec<Self>, String> {
                let offset = (page - 1) * limit;
                don_core::sqlx::query_as::<_, Self>(#select_all_query)
                    .bind(limit)
                    .bind(offset)
                    .fetch_all(pool)
                    .await
                    .map_err(|e| e.to_string())
            }

            pub async fn find_by_id(pool: &don_core::sqlx::PgPool, id: i32) -> Result<Self, String> {
                don_core::sqlx::query_as::<_, Self>(#select_by_id_query)
                    .bind(id)
                    .fetch_one(pool)
                    .await
                    .map_err(|e| e.to_string())
            }

            pub async fn delete(pool: &don_core::sqlx::PgPool, id: i32) -> Result<u64, String> {
                let result = don_core::sqlx::query(#delete_query).bind(id).execute(pool).await.map_err(|e| e.to_string())?;
                Ok(result.rows_affected())
            }

            pub async fn api_create(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::Json(mut payload): don_core::axum::Json<Self>,
            ) -> Result<don_core::axum::Json<Self>, (don_core::axum::http::StatusCode, String)> {
                #validation_code // JADOO: Save hone se pehle validate hoga!
                match payload.save(&state.db).await {
                    Ok(saved) => Ok(don_core::axum::Json(saved)),
                    Err(e) => Err((don_core::axum::http::StatusCode::BAD_REQUEST, e)),
                }
            }

            pub async fn api_get_all(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::extract::Query(params): don_core::axum::extract::Query<don_core::QueryParams>,
            ) -> Result<don_core::axum::Json<Vec<Self>>, (don_core::axum::http::StatusCode, String)> {
                let page = params.page.unwrap_or(1);
                let limit = params.limit.unwrap_or(10);
                match Self::find_all(&state.db, page, limit).await {
                    Ok(records) => Ok(don_core::axum::Json(records)),
                    Err(e) => Err((don_core::axum::http::StatusCode::INTERNAL_SERVER_ERROR, e)),
                }
            }

            pub async fn api_get_one(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::extract::Path(id): don_core::axum::extract::Path<i32>,
            ) -> Result<don_core::axum::Json<Self>, (don_core::axum::http::StatusCode, String)> {
                match Self::find_by_id(&state.db, id).await {
                    Ok(record) => Ok(don_core::axum::Json(record)),
                    Err(e) => Err((don_core::axum::http::StatusCode::NOT_FOUND, e)),
                }
            }

            pub async fn api_update(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::extract::Path(id): don_core::axum::extract::Path<i32>,
                don_core::axum::Json(mut payload): don_core::axum::Json<Self>,
            ) -> Result<don_core::axum::Json<Self>, (don_core::axum::http::StatusCode, String)> {
                #validation_code // JADOO: Update hone se pehle bhi validate hoga!
                match payload.update(&state.db, id).await {
                    Ok(updated) => Ok(don_core::axum::Json(updated)),
                    Err(e) => Err((don_core::axum::http::StatusCode::BAD_REQUEST, e)),
                }
            }

            pub async fn api_delete(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::extract::Path(id): don_core::axum::extract::Path<i32>,
            ) -> Result<don_core::axum::Json<don_core::serde_json::Value>, (don_core::axum::http::StatusCode, String)> {
                match Self::delete(&state.db, id).await {
                    Ok(_) => Ok(don_core::axum::Json(don_core::serde_json::json!({"message": "Deleted successfully"}))),
                    Err(e) => Err((don_core::axum::http::StatusCode::INTERNAL_SERVER_ERROR, e)),
                }
            }

            pub fn get_api_routes() -> don_core::axum::Router<don_core::server::AppState> {
                don_core::axum::Router::new()
                    .route("/", don_core::axum::routing::get(Self::api_get_all).post(Self::api_create))
                    .route("/:id", don_core::axum::routing::get(Self::api_get_one).put(Self::api_update).delete(Self::api_delete))
            }
        }
    };

    TokenStream::from(expanded)
}
// ==========================================
// 3. DON GUARD MACRO (RBAC)
// ==========================================
#[proc_macro_derive(DonGuard, attributes(don_role))]
pub fn don_guard_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let mut role_name = String::new();
    for attr in &ast.attrs {
        if attr.path().is_ident("don_role") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        role_name = lit_str.value();
                    }
                }
            }
        }
    }

    if role_name.is_empty() {
        panic!("DonGuard requires a #[don_role = \"...\"] attribute!");
    }

    let expanded = quote! {
        #[don_core::axum::async_trait]
        impl<S> don_core::axum::extract::FromRequestParts<S> for #struct_name
        where
            S: Send + Sync,
        {
            type Rejection = (don_core::axum::http::StatusCode, String);

            async fn from_request_parts(
                parts: &mut don_core::axum::http::request::Parts,
                _state: &S,
            ) -> Result<Self, Self::Rejection> {
                
                let auth_header = parts.headers.get("authorization").and_then(|h| h.to_str().ok());
                let auth_header = match auth_header {
                    Some(header) => header,
                    None => return Err((don_core::axum::http::StatusCode::UNAUTHORIZED, "Missing Token! Please login.".to_string())),
                };

                if !auth_header.starts_with("Bearer ") {
                    return Err((don_core::axum::http::StatusCode::UNAUTHORIZED, "Invalid Token Format!".to_string()));
                }

                let token = &auth_header[7..];
                let secret = std::env::var("JWT_SECRET").expect("JWT_SECRET missing in .env");

                let decoded = don_core::jsonwebtoken::decode::<don_core::guard::Claims>(
                    token,
                    &don_core::jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
                    &don_core::jsonwebtoken::Validation::default(),
                ).map_err(|_| (don_core::axum::http::StatusCode::UNAUTHORIZED, "Invalid or Expired Token!".to_string()))?;

                if decoded.claims.role != #role_name {
                    return Err((
                        don_core::axum::http::StatusCode::FORBIDDEN, 
                        format!("Access Denied: Route requires '{}' role! Your role is '{}'.", #role_name, decoded.claims.role)
                    ));
                }

                Ok(#struct_name)
            }
        }
    };

    TokenStream::from(expanded)
}

// ==========================================
// 4. DON SOCKET MACRO (WebSockets)
// ==========================================
#[proc_macro_derive(DonSocket)]
pub fn don_socket_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let expanded = quote! {
        impl #struct_name {
            pub fn get_ws_routes() -> don_core::axum::Router<don_core::server::AppState> {
                don_core::axum::Router::new()
                    .route("/ws", don_core::axum::routing::get(don_core::websocket::ws_handler))
            }
        }
    };

    TokenStream::from(expanded)
}