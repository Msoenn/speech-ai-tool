use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;

use crate::error::AppError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranscriptionRecord {
    pub id: String,
    pub raw_text: String,
    pub cleaned_text: String,
    pub created_at: String,
    pub duration_secs: f64,
    pub model_used: String,
}

pub struct HistoryDb {
    conn: Mutex<Connection>,
}

impl HistoryDb {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, AppError> {
        std::fs::create_dir_all(app_data_dir)
            .map_err(|e| AppError::History(e.to_string()))?;

        let db_path = app_data_dir.join("history.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| AppError::History(format!("Failed to open database: {}", e)))?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS transcriptions (
                id TEXT PRIMARY KEY,
                raw_text TEXT NOT NULL,
                cleaned_text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                duration_secs REAL NOT NULL,
                model_used TEXT NOT NULL
            )",
            [],
        )
        .map_err(|e| AppError::History(format!("Failed to create table: {}", e)))?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn insert(&self, record: &TranscriptionRecord) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO transcriptions (id, raw_text, cleaned_text, created_at, duration_secs, model_used)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                record.id,
                record.raw_text,
                record.cleaned_text,
                record.created_at,
                record.duration_secs,
                record.model_used,
            ],
        )
        .map_err(|e| AppError::History(format!("Insert failed: {}", e)))?;
        Ok(())
    }

    pub fn list(&self) -> Result<Vec<TranscriptionRecord>, AppError> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, raw_text, cleaned_text, created_at, duration_secs, model_used FROM transcriptions ORDER BY created_at DESC")
            .map_err(|e| AppError::History(e.to_string()))?;

        let records = stmt
            .query_map([], |row| {
                Ok(TranscriptionRecord {
                    id: row.get(0)?,
                    raw_text: row.get(1)?,
                    cleaned_text: row.get(2)?,
                    created_at: row.get(3)?,
                    duration_secs: row.get(4)?,
                    model_used: row.get(5)?,
                })
            })
            .map_err(|e| AppError::History(e.to_string()))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(records)
    }

    pub fn delete(&self, id: &str) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transcriptions WHERE id = ?1", params![id])
            .map_err(|e| AppError::History(e.to_string()))?;
        Ok(())
    }

    pub fn clear_all(&self) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM transcriptions", [])
            .map_err(|e| AppError::History(e.to_string()))?;
        Ok(())
    }

    pub fn prune(&self, max_items: usize) -> Result<(), AppError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM transcriptions WHERE id NOT IN (
                SELECT id FROM transcriptions ORDER BY created_at DESC LIMIT ?1
            )",
            params![max_items],
        )
        .map_err(|e| AppError::History(e.to_string()))?;
        Ok(())
    }
}
