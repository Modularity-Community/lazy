use serde::{Deserialize, Serialize};

use crate::domain::{folder_model::FolderModel, request_model::RequestSavedModel};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceTree {
    pub folders: Vec<FolderModel>,
    pub requests: Vec<RequestSavedModel>,
}
