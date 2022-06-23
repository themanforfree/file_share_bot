use std::path::Path;

use chrono::Utc;
use once_cell::sync::Lazy;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::{params, Result};

const DATABASE_URL: &str = "test.db";
pub static CONN: Lazy<ConnectionPool> = Lazy::new(|| ConnectionPool::init(DATABASE_URL).unwrap());

pub struct ConnectionPool(Pool<SqliteConnectionManager>);

impl ConnectionPool {
    fn init(database_url: &str) -> Result<Self> {
        let sqlite_connection_manager = SqliteConnectionManager::file(database_url);
        let sqlite_pool = r2d2::Pool::new(sqlite_connection_manager)
            .expect("Failed to create r2d2 SQLite connection pool");
        let conn = sqlite_pool.get().unwrap();
        conn.execute(
            "CREATE TABLE IF NOT EXISTS file_infos (
                      file_id              TEXT PRIMARY KEY,
                      file_name            TEXT,
                      timestamp            BIGINT
                      )",
            [],
        )?;
        Ok(Self(sqlite_pool))
    }
    pub fn insert(&self, file_id: &str, file_name: Option<String>) -> Result<usize> {
        self.0.get().unwrap().execute(
            "INSERT INTO file_infos (file_id, file_name, timestamp) VALUES (?1, ?2, ?3)",
            params![file_id, file_name, Utc::now().timestamp()],
        )
    }

    pub fn get(&self, file_id: &str) -> Option<String> {
        self.0
            .get()
            .unwrap()
            .query_row(
                "SELECT file_name FROM file_infos WHERE file_id=?1",
                [file_id],
                |row| row.get(0),
            )
            .ok()
    }

    pub fn delete(&self, second: i64) -> Result<usize> {
        let mut num = 0;
        let ts = Utc::now().timestamp() - second;
        let conn = self.0.get().unwrap();
        let mut stmt = conn.prepare("SELECT file_id FROM file_infos WHERE timestamp<?1")?;
        let file_ids = stmt.query_map([ts], |row| row.get(0))?;
        for file_id in file_ids {
            let file_id: String = file_id.unwrap();
            let path = Path::new("./tmp").join(&file_id);
            std::fs::remove_file(path).unwrap();
            conn.execute("DELETE FROM file_infos WHERE file_id=?1", [file_id])?;
            num += 1;
        }

        Ok(num)
    }
}

pub mod gc {
    use super::CONN;
    use tokio::time::{sleep, Duration};

    const DELETE_TIME: i64 = 10;

    pub async fn run() {
        loop {
            sleep(Duration::from_secs(5)).await;
            match CONN.delete(DELETE_TIME) {
                Ok(n) => {
                    if n > 0 {
                        log::info!("Gc deleted {} file(s)", n)
                    }
                }
                Err(e) => {
                    log::error!("An error from gc: {}", e);
                }
            }
        }
    }
}
