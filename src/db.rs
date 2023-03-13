use std::sync::Arc;

use sqlx::{Pool, Sqlite, SqlitePool};

pub type Database = Arc<DatabaseState>;

pub struct DatabaseState {
    pool: Pool<Sqlite>,
}

impl DatabaseState {
    pub async fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let pool = SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;
        Ok(Self { pool })
    }

    pub async fn set(&self, id: &String, status: u8) -> Result<(), Box<dyn std::error::Error>> {
        let status = status.to_string();
        sqlx::query!("INSERT INTO favorites (id, status) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET status = excluded.status", id, status)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    pub async fn get_status(&self, id: &String) -> Result<i64, Box<dyn std::error::Error>> {
        let rec = sqlx::query!("SELECT status FROM favorites WHERE id = ?", id)
            .fetch_one(&self.pool)
            .await?;

        Ok(rec.status)
    }

    pub async fn get_new_favorites(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        let favorites = sqlx::query!("SELECT id FROM favorites WHERE status = 0")
            .fetch_all(&self.pool)
            .await?
            .into_iter()
            .map(|r| r.id)
            .collect::<Vec<String>>();

        Ok(favorites)
    }
}

pub async fn open() -> Result<Database, Box<dyn std::error::Error>> {
    let result = Arc::new(DatabaseState::new().await?);

    Ok(result)
}
