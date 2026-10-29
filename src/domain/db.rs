use async_trait::async_trait;

#[async_trait]
pub trait DBRepository {
    async fn init_db(&self) -> Result<(), String>;
}
