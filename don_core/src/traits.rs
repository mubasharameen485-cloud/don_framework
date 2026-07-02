// don_core/src/traits.rs


#[allow(async_fn_in_trait)]
pub trait DonHooks {
   
    async fn before_save(&mut self) -> Result<(), String> {
        Ok(()) 
    }

    
    async fn before_update(&mut self) -> Result<(), String> {
        Ok(())
    }
}