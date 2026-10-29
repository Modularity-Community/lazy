use crate::domain::request_model::RequestSavedModel;
use async_trait::async_trait;

#[async_trait]
pub trait RequestSavedRepository {
    async fn save_request(&self, req: &RequestSavedModel) -> Result<i64, String>;
    async fn update_request(&self, req: &RequestSavedModel) -> Result<(), String>;
    async fn delete_request(&self, id: i64) -> Result<(), String>;
    async fn fetch_request_by_id(&self, id: i64) -> Result<RequestSavedModel, String>;
}
