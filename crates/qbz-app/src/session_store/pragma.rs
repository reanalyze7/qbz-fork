use super::schema::SessionStore;

impl SessionStore {
    #[cfg(test)]
    pub(super) fn pragma_synchronous(&self) -> Result<i64, String> {
        self.conn
            .query_row("PRAGMA synchronous", [], |row| row.get(0))
            .map_err(|e| format!("Failed to read synchronous pragma: {}", e))
    }

    #[cfg(test)]
    pub(super) fn pragma_journal_mode(&self) -> Result<String, String> {
        self.conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .map_err(|e| format!("Failed to read journal mode pragma: {}", e))
    }
}
