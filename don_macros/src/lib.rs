// don_macros/src/lib.rs

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

// ==========================================
// 1. DON AUTH MACRO
// ==========================================
#[proc_macro_derive(DonAuth)]
pub fn don_auth_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let expanded = quote! {
        impl #struct_name {
            pub fn get_auth_routes() -> don_core::axum::Router<don_core::server::AppState> {
                don_core::auth::generate_routes()
            }
            pub fn framework_info() -> &'static str {
                concat!("DonFramework: Auth generated for ", stringify!(#struct_name))
            }
        }
    };
    TokenStream::from(expanded)
}
// don_macros/src/lib.rs (Sirf DonModel Macro ka hissa update karo)

#[proc_macro_derive(DonModel)]
pub fn don_model_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;
    
    let table_name = format!("{}s", struct_name.to_string().to_lowercase());

    let fields = if let syn::Data::Struct(syn::DataStruct { fields: syn::Fields::Named(ref fields), .. }) = ast.data {
        fields.named.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>()
    } else {
        panic!("DonModel only works with named structs!");
    };

    let insert_fields: Vec<_> = fields.iter().filter(|f| f.to_string() != "id").collect();
    
    let bind_marks = (1..=insert_fields.len()).map(|i| format!("${}", i)).collect::<Vec<_>>().join(", ");
    let insert_columns = insert_fields.iter().map(|f| f.to_string()).collect::<Vec<_>>().join(", ");
    
    let insert_query = format!("INSERT INTO {} ({}) VALUES ({}) RETURNING *", table_name, insert_columns, bind_marks);
    
    // NAYA JADOO: Pagination Query (ORDER BY id DESC LIMIT $1 OFFSET $2)
    let select_all_query = format!("SELECT * FROM {} ORDER BY id DESC LIMIT $1 OFFSET $2", table_name);
    
    let select_by_id_query = format!("SELECT * FROM {} WHERE id = $1", table_name);
    let delete_query = format!("DELETE FROM {} WHERE id = $1", table_name);
    
    let update_sets = insert_fields.iter().enumerate().map(|(i, f)| format!("{} = ${}", f, i + 1)).collect::<Vec<_>>().join(", ");
    let update_query = format!("UPDATE {} SET {} WHERE id = ${} RETURNING *", table_name, update_sets, insert_fields.len() + 1);

    let binds = insert_fields.iter().map(|f| quote! { .bind(self.#f.clone()) });
    let update_binds = binds.clone();

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

            // NAYA JADOO: Database function ab page aur limit accept karega
            pub async fn find_all(pool: &don_core::sqlx::PgPool, page: i64, limit: i64) -> Result<Vec<Self>, String> {
                let offset = (page - 1) * limit; // Math for pagination
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

            // ==========================================
            // AXUM API HANDLERS
            // ==========================================
            pub async fn api_create(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::Json(mut payload): don_core::axum::Json<Self>,
            ) -> Result<don_core::axum::Json<Self>, (don_core::axum::http::StatusCode, String)> {
                match payload.save(&state.db).await {
                    Ok(saved) => Ok(don_core::axum::Json(saved)),
                    Err(e) => Err((don_core::axum::http::StatusCode::BAD_REQUEST, e)),
                }
            }

            // NAYA JADOO: API Handler ab URL se Query Params pakrega
            pub async fn api_get_all(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::extract::Query(params): don_core::axum::extract::Query<don_core::QueryParams>,
            ) -> Result<don_core::axum::Json<Vec<Self>>, (don_core::axum::http::StatusCode, String)> {
                
                // Agar user ne page/limit nahi diya, toh default (Page 1, Limit 10) laga do
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
                match payload.update(&state.db, id).await {
                    Ok(updated) => Ok(don_core::axum::Json(updated)),
                    Err(e) => Err((don_core::axum::http::StatusCode::BAD_REQUEST, e)),
                }
            }

            pub async fn api_delete(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                don_core::axum::extract::Path(id): don_core::axum::extract::Path<i32>,
            ) -> Result<don_core::axum::Json<serde_json::Value>, (don_core::axum::http::StatusCode, String)> {
                match Self::delete(&state.db, id).await {
                    Ok(_) => Ok(don_core::axum::Json(serde_json::json!({"message": "Deleted successfully"}))),
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

// don_macros/src/lib.rs (Aakhir mein add karo)

// ==========================================
// 3. DON GUARD MACRO (FLEXIBLE RBAC / IAM)
// ==========================================
#[proc_macro_derive(DonGuard, attributes(don_role))]
pub fn don_guard_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    // 1. User ne jo role diya hai (e.g., #[don_role = "manager"]) usay parhna
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
        panic!("DonGuard requires a #[don_role = \"...\"] attribute! Example: #[don_role = \"manager\"]");
    }

    // 2. Axum ka Middleware (Extractor) Generate karna
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
                
                // A. Token Nikalna
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

                // B. Token Decode karna
                let decoded = don_core::jsonwebtoken::decode::<don_core::auth::Claims>(
                    token,
                    &don_core::jsonwebtoken::DecodingKey::from_secret(secret.as_bytes()),
                    &don_core::jsonwebtoken::Validation::default(),
                ).map_err(|_| (don_core::axum::http::StatusCode::UNAUTHORIZED, "Invalid or Expired Token!".to_string()))?;

                // C. JADOO: Role Check karna!
                if decoded.claims.role != #role_name {
                    return Err((
                        don_core::axum::http::StatusCode::FORBIDDEN, 
                        format!("Access Denied: Route requires '{}' role! Your role is '{}'.", #role_name, decoded.claims.role)
                    ));
                }

                // D. Agar sab theek hai toh Rasta Khol do!
                Ok(#struct_name)
            }
        }
    };

    TokenStream::from(expanded)
}

// don_macros/src/lib.rs (Aakhir mein paste karo)

// ==========================================
// 4. DON SOCKET MACRO (REAL-TIME MAGIC)
// ==========================================
#[proc_macro_derive(DonSocket)]
pub fn don_socket_derive(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let struct_name = &ast.ident;

    let expanded = quote! {
        impl #struct_name {
            /// Automatically generated function to return the WebSocket Router
            pub fn get_ws_routes() -> don_core::axum::Router<don_core::server::AppState> {
                don_core::axum::Router::new()
                    // Yeh route automatically don_core ke websocket handler se connect ho jayega
                    .route("/ws", don_core::axum::routing::get(don_core::websocket::ws_handler))
            }
        }
    };

    TokenStream::from(expanded)
}