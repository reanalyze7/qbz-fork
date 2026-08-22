//! Batch variant of local-copy detection, for checking many Qobuz track
//! ids against `local_tracks` in a single query.

use crate::LibraryError;

use super::super::LibraryDatabase;

impl LibraryDatabase {
    /// Batch check which track IDs have local copies
    /// Returns a set of Qobuz track IDs that have local versions
    pub fn get_tracks_with_local_copies(
        &self,
        qobuz_track_ids: &[u64],
    ) -> Result<std::collections::HashSet<u64>, LibraryError> {
        use std::collections::HashSet;

        if qobuz_track_ids.is_empty() {
            return Ok(HashSet::new());
        }

        // Build placeholders for IN clause
        let placeholders: Vec<String> = (1..=qobuz_track_ids.len())
            .map(|i| format!("?{}", i))
            .collect();
        let placeholders_str = placeholders.join(",");

        let query = format!(
            "SELECT DISTINCT qobuz_track_id FROM local_tracks WHERE qobuz_track_id IN ({})",
            placeholders_str
        );

        let mut stmt = self
            .conn
            .prepare(&query)
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let params: Vec<rusqlite::types::Value> = qobuz_track_ids
            .iter()
            .map(|&id| rusqlite::types::Value::Integer(id as i64))
            .collect();

        let rows = stmt
            .query_map(rusqlite::params_from_iter(params.iter()), |row| {
                row.get::<_, i64>(0)
            })
            .map_err(|e| LibraryError::Database(e.to_string()))?;

        let mut result = HashSet::new();
        for row in rows {
            if let Ok(id) = row {
                result.insert(id as u64);
            }
        }

        Ok(result)
    }
}
