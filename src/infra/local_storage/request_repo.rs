use async_trait::async_trait;
use sqlx::Row;

use crate::{
    domain::{
        request_model::{AuthConfig, RequestSavedModel},
        request_repository::RequestSavedRepository,
    },
    infra::local_storage::sqlite_repo::SqliteRepository,
};

#[async_trait]
impl RequestSavedRepository for SqliteRepository {
    async fn save_request(&self, req: &RequestSavedModel) -> Result<i64, String> {
        let headers_json = serde_json::to_string(&req.headers).map_err(|e| e.to_string())?;
        let params_json = serde_json::to_string(&req.params).map_err(|e| e.to_string())?;
        let auth_json = serde_json::to_string(&req.auth).map_err(|e| e.to_string())?;

        let result = sqlx::query(
            r#"
            INSERT INTO requests (folder_id, name, method, url, headers, params, auth, body)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(req.folder_id)
        .bind(&req.name)
        .bind(&req.method)
        .bind(&req.url)
        .bind(headers_json)
        .bind(params_json)
        .bind(auth_json)
        .bind(&req.body)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(result.last_insert_rowid())
    }

    async fn update_request(&self, req: &RequestSavedModel) -> Result<(), String> {
        let id = req
            .id
            .ok_or_else(|| "Request ID required for update".to_string())?;
        let headers_json = serde_json::to_string(&req.headers).map_err(|e| e.to_string())?;
        let params_json = serde_json::to_string(&req.params).map_err(|e| e.to_string())?;
        let auth_json = serde_json::to_string(&req.auth).map_err(|e| e.to_string())?;

        sqlx::query(
            r#"
            UPDATE requests 
            SET folder_id = ?, name = ?, method = ?, url = ?, headers = ?, params = ?, auth = ?, body = ?, updated_at = CURRENT_TIMESTAMP
            WHERE id = ?
            "#,
        )
        .bind(req.folder_id)
        .bind(&req.name)
        .bind(&req.method)
        .bind(&req.url)
        .bind(headers_json)
        .bind(params_json)
        .bind(auth_json)
        .bind(&req.body)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn delete_request(&self, id: i64) -> Result<(), String> {
        sqlx::query("DELETE FROM requests WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    async fn fetch_request_by_id(&self, id: i64) -> Result<RequestSavedModel, String> {
        let row = sqlx::query("SELECT * FROM requests WHERE id = ?")
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| e.to_string())?;

        let headers_raw: String = row.get("headers");
        let params_raw: String = row.get("params");
        let auth_raw: String = row.get("auth");

        Ok(RequestSavedModel {
            id: Some(row.get("id")),
            folder_id: row.get("folder_id"),
            name: row.get("name"),
            method: row.get("method"),
            url: row.get("url"),
            headers: serde_json::from_str(&headers_raw).unwrap_or_default(),
            params: serde_json::from_str(&params_raw).unwrap_or_default(),
            auth: serde_json::from_str(&auth_raw).unwrap_or(AuthConfig::None),
            body: row.get("body"),
        })
    }
}
