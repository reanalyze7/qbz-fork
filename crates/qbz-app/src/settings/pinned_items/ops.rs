use super::service::PinnedItemsService;
use super::PinnedItem;
use rusqlite::params;
use std::collections::HashSet;

impl PinnedItemsService {
    /// Check if an item is pinned - O(1) operation.
    #[inline]
    pub fn is_pinned(&self, kind: &str, id: &str) -> bool {
        // O(1) HashSet lookup.
        self.pinned_keys
            .read()
            .map(|set| set.contains(&(kind.to_string(), id.to_string())))
            .unwrap_or(false)
    }

    /// Pin an item (upsert). The stored `pinned_at` is stamped now — the
    /// value carried by `item` is ignored on write.
    pub fn pin(&self, item: &PinnedItem) -> Result<(), String> {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO pinned_items
                 (kind, id, title, subtitle, artwork_url, pinned_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                params![
                    item.kind,
                    item.id,
                    item.title,
                    item.subtitle,
                    item.artwork_url,
                    now
                ],
            )
            .map_err(|e| format!("Failed to pin item: {}", e))?;

        // Update in-memory set.
        if let Ok(mut set) = self.pinned_keys.write() {
            set.insert((item.kind.clone(), item.id.clone()));
        }

        log::info!(
            "[Pinned] Pinned {}: {} (id={})",
            item.kind,
            item.title,
            item.id
        );
        Ok(())
    }

    /// Unpin an item. Absent rows are Ok, not an error.
    pub fn unpin(&self, kind: &str, id: &str) -> Result<(), String> {
        self.conn
            .execute(
                "DELETE FROM pinned_items WHERE kind = ?1 AND id = ?2",
                params![kind, id],
            )
            .map_err(|e| format!("Failed to unpin item: {}", e))?;

        // Update in-memory set.
        if let Ok(mut set) = self.pinned_keys.write() {
            set.remove(&(kind.to_string(), id.to_string()));
        }

        log::info!("[Pinned] Unpinned {} id={}", kind, id);
        Ok(())
    }

    /// Get all pinned items, newest first.
    pub fn list(&self) -> Result<Vec<PinnedItem>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT kind, id, title, subtitle, artwork_url, pinned_at
                 FROM pinned_items
                 ORDER BY pinned_at DESC",
            )
            .map_err(|e| format!("Failed to prepare pinned items query: {}", e))?;

        let items = stmt
            .query_map([], |row| {
                Ok(PinnedItem {
                    kind: row.get(0)?,
                    id: row.get(1)?,
                    title: row.get(2)?,
                    subtitle: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    artwork_url: row.get::<_, Option<String>>(4)?.unwrap_or_default(),
                    pinned_at: row.get(5)?,
                })
            })
            .map_err(|e| format!("Failed to query pinned items: {}", e))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(items)
    }

    /// Get count of pinned items.
    pub fn count(&self) -> usize {
        self.pinned_keys.read().map(|set| set.len()).unwrap_or(0)
    }

    /// Snapshot of the in-memory `(kind, id)` set, for bulk card stamping.
    pub fn keys_snapshot(&self) -> HashSet<(String, String)> {
        self.pinned_keys
            .read()
            .map(|set| set.clone())
            .unwrap_or_default()
    }
}
