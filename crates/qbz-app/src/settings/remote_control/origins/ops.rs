use rusqlite::params;

use super::store::AllowedOriginsStore;
use super::AllowedOrigin;
use crate::settings::remote_control::DEFAULT_ALLOWED_ORIGINS;

impl AllowedOriginsStore {
    pub fn add_origin(&self, origin: &str) -> Result<AllowedOrigin, String> {
        let normalized = origin.trim().to_lowercase();

        if normalized.is_empty() {
            return Err("Origin cannot be empty".to_string());
        }

        self.conn
            .execute(
                "INSERT INTO allowed_origins (origin, is_default) VALUES (?1, 0)",
                params![normalized],
            )
            .map_err(|e| {
                if e.to_string().contains("UNIQUE constraint") {
                    "Origin already exists".to_string()
                } else {
                    format!("Failed to add origin: {}", e)
                }
            })?;

        let id = self.conn.last_insert_rowid();
        Ok(AllowedOrigin {
            id,
            origin: normalized,
            is_default: false,
        })
    }

    pub fn remove_origin(&self, id: i64) -> Result<(), String> {
        let affected = self
            .conn
            .execute("DELETE FROM allowed_origins WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to remove origin: {}", e))?;

        if affected == 0 {
            return Err("Origin not found".to_string());
        }
        Ok(())
    }

    pub fn restore_defaults(&self) -> Result<(), String> {
        for origin in DEFAULT_ALLOWED_ORIGINS {
            let _ = self.conn.execute(
                "INSERT OR IGNORE INTO allowed_origins (origin, is_default) VALUES (?1, 1)",
                params![origin],
            );
        }
        Ok(())
    }
}
