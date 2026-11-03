use async_trait::async_trait;
use sqlx::SqlitePool;

#[derive(Debug, Clone)]
pub struct SqliteRepository {
    pub pool: SqlitePool,
}

impl SqliteRepository {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }
}

#[async_trait]
pub trait DBRepository {
    async fn init_db(&self) -> Result<(), String>;
}

#[async_trait]
impl DBRepository for SqliteRepository {
    async fn init_db(&self) -> Result<(), String> {
        sqlx::migrate!("./migrations")
            .run(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("failed migration table {e}");
                e.to_string()
            })?;

        Ok(())
    }
}
