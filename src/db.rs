use std::sync::Arc;

use rusqlite::Connection;
use tokio::sync::Mutex;

pub type Database = Arc<DatabaseState>;

#[derive(Debug)]
pub struct DatabaseState {
    conn: Mutex<Connection>,
}

impl DatabaseState {
    pub fn new() -> Result<Self, Box<dyn std::error::Error>> {
        let conn = Mutex::new(Connection::open("data.db")?);
        Ok(Self { conn })
    }

    pub async fn update_schema(&self) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().await;
        conn.execute(
            "CREATE TABLE IF NOT EXISTS favorites (
                    id TEXT PRIMARY KEY UNIQUE,
                    downloaded INTEGER NOT NULL
                )",
            (),
        )?;

        Ok(())
    }

    pub async fn set(&self, key: &String, val: u8) -> Result<(), Box<dyn std::error::Error>> {
        let conn = self.conn.lock().await;
        let mut key_set = conn.prepare_cached("INSERT INTO favorites (id, downloaded) VALUES (?, ?) ON CONFLICT(id) DO UPDATE SET downloaded = excluded.downloaded")?;
        key_set.execute([key, &val.to_string()])?;
        Ok(())
    }

    pub async fn get(&self, key: &String) -> Option<u8> {
        let conn = self.conn.lock().await;
        let stmt = conn.prepare("SELECT downloaded FROM favorites WHERE id = ?");
        if let Ok(mut key_get) = stmt {
            key_get
                .query_row([key], |r| {
                    let val: u8 = r.get(0)?;
                    Ok(val)
                })
                .ok()
        } else {
            None
        }
    }

    pub async fn get_favorites(&self) -> Result<Option<Vec<String>>, Box<dyn std::error::Error>> {
        let conn = self.conn.lock().await;
        let stmt = conn.prepare("SELECT id FROM favorites WHERE downloaded = 0");
        let mut favorites: Vec<String> = Vec::new();
        if let Ok(mut rows) = stmt {
            let ids = rows.query_map([], |r| r.get::<_, u8>(0)).unwrap();
            for id in ids {
                favorites.push(id.unwrap().to_string());
            }

            Ok(Some(favorites))
        } else {
            Ok(None)
        }
    }
}

pub async fn open() -> Result<Database, Box<dyn std::error::Error>> {
    let result = Arc::new(DatabaseState::new()?);
    result.update_schema().await?;

    Ok(result)
}
