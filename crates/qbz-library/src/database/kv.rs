//! Frontend-agnostic small key/value settings store (`library_kv` table).

use rusqlite::{params, OptionalExtension};

use crate::LibraryError;

use super::LibraryDatabase;

impl LibraryDatabase {
    /// Read a small key-value setting (frontend-agnostic; e.g. the tag-editor
    /// direct-write acknowledgement flag). Returns None when the key is absent.
    pub fn get_kv(&self, key: &str) -> Result<Option<String>, LibraryError> {
        self.conn
            .query_row(
                "SELECT value FROM library_kv WHERE key = ?",
                params![key],
                |row| row.get::<_, String>(0),
            )
            .optional()
            .map_err(|e| LibraryError::Database(e.to_string()))
    }

    /// Write a small key-value setting (upsert).
    pub fn set_kv(&self, key: &str, value: &str) -> Result<(), LibraryError> {
        self.conn
            .execute(
                "INSERT INTO library_kv (key, value) VALUES (?, ?)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )
            .map_err(|e| LibraryError::Database(e.to_string()))?;
        Ok(())
    }
}
