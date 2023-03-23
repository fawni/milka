use std::sync::Arc;

use miette::{Context, IntoDiagnostic};
use sqlx::{Pool, Sqlite, SqlitePool};

pub type Database = Arc<DatabaseState>;

pub struct DatabaseState {
    pool: Pool<Sqlite>,
}

impl DatabaseState {
    pub async fn new() -> miette::Result<Self> {
        let pool = SqlitePool::connect(
            &std::env::var("DATABASE_URL")
                .into_diagnostic()
                .wrap_err("DATABASE_URL")?,
        )
        .await
        .into_diagnostic()?;
        Ok(Self { pool })
    }

    pub async fn set(&self, id: &String, status: u8) -> miette::Result<()> {
        let status = status.to_string();
        sqlx::query!("INSERT INTO favorites (id, status) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET status = excluded.status", id, status)
            .execute(&self.pool)
            .await.into_diagnostic()?;

        Ok(())
    }

    pub async fn get_status(&self, id: &String) -> miette::Result<i64> {
        let rec = sqlx::query!("SELECT status FROM favorites WHERE id = ?", id)
            .fetch_one(&self.pool)
            .await
            .into_diagnostic()?;

        Ok(rec.status)
    }

    pub async fn get_new_favorites(&self) -> miette::Result<Vec<String>> {
        let favorites = sqlx::query!("SELECT id FROM favorites WHERE status = 0")
            .fetch_all(&self.pool)
            .await
            .into_diagnostic()?
            .into_iter()
            .map(|r| r.id)
            .collect::<Vec<String>>();

        Ok(favorites)
    }
}

pub async fn open() -> miette::Result<Database> {
    let result = Arc::new(DatabaseState::new().await?);

    Ok(result)
}
