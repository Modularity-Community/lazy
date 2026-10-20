use async_trait::async_trait;

use super::request_model::RequestSavedModel;

#[async_trait]
pub trait RequestSavedRepository: Send + Sync {
    async fn save(&self, req: &RequestSavedModel) -> Result<i64, String>;
    async fn fetch(&self) -> Result<Vec<RequestSavedModel>, String>;
    async fn delete(&self) -> Result<i64, String>;
    async fn update(&self) -> Result<i64, String>;
    async fn fetch_one(&self) -> Result<RequestSavedModel, String>;
}
