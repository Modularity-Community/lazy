use async_trait::async_trait;

use crate::{
    domain::request_model::RequestSavedModel, infra::local_storage::sqlite_repo::SqliteRepository,
};

#[async_trait]
pub trait RequestSavedRepository {
    async fn save_request(&self, req: &RequestSavedModel) -> Result<i64, String>;
    async fn update_request(&self, req: &RequestSavedModel) -> Result<(), String>;
    async fn delete_request(&self, id: Option<i64>) -> Result<(), String>;
    async fn rename_request(&self, id: Option<i64>, name: &str) -> Result<(), sqlx::Error>;
}

#[async_trait]
impl RequestSavedRepository for SqliteRepository {
    async fn rename_request(&self, id: Option<i64>, name: &str) -> Result<(), sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE requests
            SET name = ?1
            WHERE id = ?2;
            "#,
        )
        .bind(name)
        .bind(id)
        .execute(&self.pool)
        .await?;

        if result.rows_affected() == 0 {
            tracing::error!("failed rename request");
            return Err(sqlx::Error::RowNotFound);
        }

        tracing::debug!("success renaming request: {:?}", id);

        Ok(())
    }
    async fn save_request(&self, req: &RequestSavedModel) -> Result<i64, String> {
        let headers_json = serde_json::to_string(&req.headers).map_err(|e| e.to_string())?;
        let params_json = serde_json::to_string(&req.params).map_err(|e| e.to_string())?;
        let auth_json = serde_json::to_string(&req.auth).map_err(|e| e.to_string())?;

        let result = sqlx::query(
            r#"
            INSERT INTO requests (folder_id, name, method, url, headers, params, auth, body, response)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
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
        .bind(&req.response)
        .execute(&self.pool)
        .await
        .map_err(|e| {
                tracing::error!("failed save request: {:?}", e);
                e.to_string()
            })?;

        tracing::debug!("task save request: {:?}", req);

        Ok(result.last_insert_rowid())
    }

    async fn update_request(&self, req: &RequestSavedModel) -> Result<(), String> {
        let id = req
            .id
            .ok_or_else(|| "Request ID required for update".to_string())?;
        let headers_json = serde_json::to_string(&req.headers).map_err(|e| e.to_string())?;
        let params_json = serde_json::to_string(&req.params).map_err(|e| e.to_string())?;
        let auth_json = serde_json::to_string(&req.auth).map_err(|e| e.to_string())?;

        let result = sqlx::query(
            r#"
            UPDATE requests 
            SET folder_id = ?, name = ?, method = ?, url = ?, headers = ?, params = ?, auth = ?, body = ?, updated_at = CURRENT_TIMESTAMP, response = ?
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
        .bind(&req.response)
        .bind(id)
        .execute(&self.pool)
        .await
        .map_err(|e| {
                tracing::error!("failed update request {:?}",e);
                e.to_string()
            })?;

        if result.rows_affected() == 0 {
            return Err(format!("Request {id} no longer exists"));
        }
        tracing::debug!("success update request: {:?}", id);

        Ok(())
    }

    async fn delete_request(&self, id: Option<i64>) -> Result<(), String> {
        sqlx::query("DELETE FROM requests WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("failed delete request {:?}", e);
                e.to_string()
            })?;

        tracing::debug!("success delete request: {:?}", id);

        Ok(())
    }
    //
    // async fn fetch_request_by_id(&self, id: i64) -> Result<RequestSavedModel, String> {
    //     let row = sqlx::query("SELECT * FROM requests WHERE id = ?")
    //         .bind(id)
    //         .fetch_one(&self.pool)
    //         .await
    //         .map_err(|e| e.to_string())?;
    //
    //     let headers_raw: String = row.get("headers");
    //     let params_raw: String = row.get("params");
    //     let auth_raw: String = row.get("auth");
    //
    //     Ok(RequestSavedModel {
    //         id: Some(row.get("id")),
    //         folder_id: row.get("folder_id"),
    //         name: row.get("name"),
    //         method: row.get("method"),
    //         url: row.get("url"),
    //         headers: serde_json::from_str(&headers_raw).unwrap_or_default(),
    //         params: serde_json::from_str(&params_raw).unwrap_or_default(),
    //         auth: serde_json::from_str(&auth_raw).unwrap_or(AuthConfig::None),
    //         body: row.get("body"),
    //         response: row.get("response"),
    //     })
    // }
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RequestSearchResult {
    pub id: i64,
    pub folder_id: Option<i64>,
    pub name: String,
    pub method: String,
    pub url: String,
}

impl SqliteRepository {
    pub async fn search_requests(&self, keyword: &str) -> Result<Vec<RequestSearchResult>, String> {
        let escaped = keyword.replace('\\', "\\\\").replace('%', "\\%").replace('_', "\\_");
        let pattern = format!("%{escaped}%");
        sqlx::query_as::<_, RequestSearchResult>(
            r#"SELECT id, folder_id, name, method, url FROM requests
               WHERE name LIKE ?1 ESCAPE '\' OR url LIKE ?1 ESCAPE '\'
               ORDER BY name COLLATE NOCASE LIMIT 50"#
        )
        .bind(pattern)
        .fetch_all(&self.pool)
        .await
        .map_err(|err| err.to_string())
    }

    pub async fn fetch_request_by_id(&self, id: i64) -> Result<RequestSavedModel, String> {
        use sqlx::Row;
        use crate::enums::auth_config::AuthConfig;
        let row = sqlx::query("SELECT * FROM requests WHERE id = ?")
            .bind(id).fetch_one(&self.pool).await.map_err(|err| err.to_string())?;
        let headers: String = row.get("headers");
        let params: String = row.get("params");
        let auth: String = row.get("auth");
        Ok(RequestSavedModel {
            id: Some(row.get("id")),
            folder_id: row.get("folder_id"),
            name: row.get("name"),
            method: row.get("method"),
            url: row.get("url"),
            headers: serde_json::from_str(&headers).map_err(|e| e.to_string())?,
            params: serde_json::from_str(&params).map_err(|e| e.to_string())?,
            auth: serde_json::from_str(&auth).unwrap_or(AuthConfig::None),
            body: row.get("body"),
            response: row.get("response"),
        })
    }
}
