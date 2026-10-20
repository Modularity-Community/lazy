use async_trait::async_trait;
use sqlx::SqlitePool;

use crate::domain::{request_model::RequestSavedModel, request_repository::RequestSavedRepository};

pub struct SqliteRepository {
    pool: SqlitePool,
}

impl SqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RequestSavedRepository for SqliteRepository {
    async fn save(&self, req: &RequestSavedModel) -> Result<i64, String> {
        Result::Ok(0)
    }
    async fn fetch(&self) -> Result<Vec<RequestSavedModel>, String> {
        Result::Ok(Vec::new())
    }
    async fn delete(&self) -> Result<i64, String> {
        Result::Ok(0)
    }
    async fn update(&self) -> Result<i64, String> {
        Result::Ok(0)
    }
    async fn fetch_one(&self) -> Result<RequestSavedModel, String> {
        Result::Err(String::new())
    }
}
