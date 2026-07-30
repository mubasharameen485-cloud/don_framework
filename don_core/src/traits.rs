// don_core/src/traits.rs

/// The DonHooks Trait for CRUD Operations
#[allow(async_fn_in_trait)]
pub trait DonHooks {
    async fn before_save(&mut self) -> Result<(), String> { Ok(()) }
    async fn before_update(&mut self) -> Result<(), String> { Ok(()) }
}

/// ==========================================
/// NAYA JADOO: The DonAuthHooks Trait for Authentication
/// ==========================================
#[allow(async_fn_in_trait)]
pub trait DonAuthHooks {
    // Signup hone se pehle chalega (Data validation aur modification ke liye)
    async fn before_signup(&mut self) -> Result<(), String> { 
        Ok(()) 
    }

    // Login hone se pehle chalega (Rate limiting, blocking hackers ke liye)
    // Ismein 'self' nahi hota kyunke user abhi login nahi hua, humein sirf uska username/email milta hai.
    async fn before_login(_primary_key: &str) -> Result<(), String> { 
        Ok(()) 
    }
}