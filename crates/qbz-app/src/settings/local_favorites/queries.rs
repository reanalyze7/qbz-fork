use super::model::LocalFavItem;
use super::service::LocalFavoritesService;
use rusqlite::params;
use std::collections::HashSet;

impl LocalFavoritesService {
    /// Check if a local item is favorited — O(1).
    #[inline]
    pub fn is_favorite(&self, kind: &str, id: &str) -> bool {
        self.keys
            .read()
            .map(|set| set.contains(&(kind.to_string(), id.to_string())))
            .unwrap_or(false)
    }

    /// Favorite an item (upsert). `favorited_at` is stamped now.
    pub fn favorite(&self, item: &LocalFavItem) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO local_favorites
                 (kind, id, title, subtitle, artwork_url, artist, source, favorited_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    item.kind,
                    item.id,
                    item.title,
                    item.subtitle,
                    item.artwork_url,
                    item.artist,
                    item.source,
                    now
                ],
            )
            .map_err(|e| format!("Failed to favorite item: {}", e))?;

        if let Ok(mut set) = self.keys.write() {
            set.insert((item.kind.clone(), item.id.clone()));
        }
        Ok(())
    }

    /// Unfavorite an item. Absent rows are Ok, not an error.
    pub fn unfavorite(&self, kind: &str, id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM local_favorites WHERE kind = ?1 AND id = ?2",
                params![kind, id],
            )
            .map_err(|e| format!("Failed to unfavorite item: {}", e))?;

        if let Ok(mut set) = self.keys.write() {
            set.remove(&(kind.to_string(), id.to_string()));
        }
        Ok(())
    }

    /// All favorites, newest first.
    pub fn list(&self) -> Result<Vec<LocalFavItem>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT kind, id, title, subtitle, artwork_url, artist, source, favorited_at
                 FROM local_favorites
                 ORDER BY favorited_at DESC",
            )
            .map_err(|e| format!("Failed to prepare local favorites query: {}", e))?;

        let items = stmt
            .query_map([], |row| {
                Ok(LocalFavItem {
                    kind: row.get(0)?,
                    id: row.get(1)?,
                    title: row.get(2)?,
                    subtitle: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    artwork_url: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    artist: row.get::<_, Option<String>>(5)?.unwrap_or_default(),
                    source: row.get(6)?,
                    favorited_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query local favorites: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }

    /// Per-artist favorite counts (album + track kinds carry an artist).
    pub fn count_by_artist(&self) -> Result<Vec<(String, i64)>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT artist, COUNT(*) FROM local_favorites
                 WHERE artist IS NOT NULL AND artist != ''
                 GROUP BY artist ORDER BY COUNT(*) DESC",
            )
            .map_err(|e| format!("Failed to prepare count-by-artist query: {}", e))?;

        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| format!("Failed to query count-by-artist: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(rows)
    }

    /// Count of favorites.
    pub fn count(&self) -> usize {
        self.keys.read().map(|set| set.len()).unwrap_or(0)
    }

    /// Snapshot of the in-memory `(kind, id)` set, for bulk card stamping.
    pub fn keys_snapshot(&self) -> HashSet<(String, String)> {
        self.keys.read().map(|set| set.clone()).unwrap_or_default()
    }
}
