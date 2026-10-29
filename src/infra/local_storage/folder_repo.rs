use std::collections::HashMap;

use async_trait::async_trait;
use sqlx::Row;

use crate::{
    domain::{
        folder_model::FolderModel,
        folder_repository::FolderRepository,
        request_model::{AuthConfig, RequestSavedModel},
    },
    infra::local_storage::sqlite_repo::SqliteRepository,
};

#[async_trait]
impl FolderRepository for SqliteRepository {
    async fn save_folder(&self, name: &str, parent_id: Option<i64>) -> Result<i64, String> {
        let result = sqlx::query("INSERT INTO folders (name, parent_id) VALUES (?, ?)")
            .bind(name)
            .bind(parent_id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }

    async fn delete_folder(&self, id: i64) -> Result<(), String> {
        sqlx::query("DELETE FROM folders WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn fetch_workspace_tree(&self) -> Result<Vec<FolderModel>, String> {
        let folder_rows = sqlx::query("SELECT id, parent_id, name FROM folders")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let request_rows = sqlx::query("SELECT * FROM requests")
            .fetch_all(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let mut requests_by_folder: HashMap<Option<i64>, Vec<RequestSavedModel>> = HashMap::new();
        for row in request_rows {
            let folder_id: Option<i64> = row.get("folder_id");
            let req = RequestSavedModel {
                id: Some(row.get("id")),
                folder_id,
                name: row.get("name"),
                method: row.get("method"),
                url: row.get("url"),
                headers: serde_json::from_str(row.get("headers")).unwrap_or_default(),
                params: serde_json::from_str(row.get("params")).unwrap_or_default(),
                auth: serde_json::from_str(row.get("auth")).unwrap_or(AuthConfig::None),
                body: row.get("body"),
            };
            requests_by_folder.entry(folder_id).or_default().push(req);
        }

        struct RawFolder {
            id: i64,
            parent_id: Option<i64>,
            name: String,
        }

        let raw_folders: Vec<RawFolder> = folder_rows
            .into_iter()
            .map(|r| RawFolder {
                id: r.get("id"),
                parent_id: r.get("parent_id"),
                name: r.get("name"),
            })
            .collect();

        fn build_node(
            parent_id: Option<i64>,
            raw_folders: &[RawFolder],
            requests_by_folder: &mut HashMap<Option<i64>, Vec<RequestSavedModel>>,
        ) -> Vec<FolderModel> {
            let mut nodes = Vec::new();

            for f in raw_folders.iter().filter(|f| f.parent_id == parent_id) {
                let subfolders = build_node(Some(f.id), raw_folders, requests_by_folder);
                let requests = requests_by_folder.remove(&Some(f.id)).unwrap_or_default();

                nodes.push(FolderModel {
                    id: Some(f.id),
                    parent_id: f.parent_id,
                    name: f.name.clone(),
                    subfolders,
                    requests,
                });
            }

            nodes
        }

        Ok(build_node(None, &raw_folders, &mut requests_by_folder))
    }
}
