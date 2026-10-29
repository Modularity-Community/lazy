use async_trait::async_trait;

use crate::domain::folder_model::FolderModel;

#[async_trait]
pub trait FolderRepository {
    async fn save_folder(&self, name: &str, parent_id: Option<i64>) -> Result<i64, String>;
    async fn delete_folder(&self, id: i64) -> Result<(), String>;

    async fn fetch_workspace_tree(&self) -> Result<Vec<FolderModel>, String>;
}
