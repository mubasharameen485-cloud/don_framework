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

// ==========================================
// 2. DON MODEL MACRO (FULL CRUD API + DATABASE)
// ==========================================
// don_macros/src/lib.rs (DonModel Macro part)

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
    let select_all_query = format!("SELECT * FROM {}", table_name);
    let select_by_id_query = format!("SELECT * FROM {} WHERE id = $1", table_name);
    let delete_query = format!("DELETE FROM {} WHERE id = $1", table_name);
    
    let update_sets = insert_fields.iter().enumerate().map(|(i, f)| format!("{} = ${}", f, i + 1)).collect::<Vec<_>>().join(", ");
    let update_query = format!("UPDATE {} SET {} WHERE id = ${} RETURNING *", table_name, update_sets, insert_fields.len() + 1);

    let binds = insert_fields.iter().map(|f| quote! { .bind(self.#f.clone()) });
    let update_binds = binds.clone();

    let expanded = quote! {
        impl #struct_name {
            // ==========================================
            // DATABASE METHODS (With Hooks Injected!)
            // ==========================================
            
            
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

            pub async fn find_all(pool: &don_core::sqlx::PgPool) -> Result<Vec<Self>, String> {
                don_core::sqlx::query_as::<_, Self>(#select_all_query)
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
                let result = don_core::sqlx::query(#delete_query)
                    .bind(id)
                    .execute(pool)
                    .await
                    .map_err(|e| e.to_string())?;
                Ok(result.rows_affected())
            }

            // ==========================================
            // AXUM API HANDLERS
            // ==========================================
            pub async fn api_create(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
                // NAYA: mut payload liya hai taake hook isay change kar sake
                don_core::axum::Json(mut payload): don_core::axum::Json<Self>,
            ) -> Result<don_core::axum::Json<Self>, (don_core::axum::http::StatusCode, String)> {
                match payload.save(&state.db).await {
                    Ok(saved) => Ok(don_core::axum::Json(saved)),
                   
                    Err(e) => Err((don_core::axum::http::StatusCode::BAD_REQUEST, e)),
                }
            }

            pub async fn api_get_all(
                don_core::axum::extract::State(state): don_core::axum::extract::State<don_core::server::AppState>,
            ) -> Result<don_core::axum::Json<Vec<Self>>, (don_core::axum::http::StatusCode, String)> {
                match Self::find_all(&state.db).await {
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