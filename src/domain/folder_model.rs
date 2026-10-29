use serde::{Deserialize, Serialize};

use crate::domain::request_model::RequestSavedModel;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FolderModel {
    pub id: Option<i64>,
    pub parent_id: Option<i64>,
    pub name: String,
    pub subfolders: Vec<FolderModel>,
    pub requests: Vec<RequestSavedModel>,
}
