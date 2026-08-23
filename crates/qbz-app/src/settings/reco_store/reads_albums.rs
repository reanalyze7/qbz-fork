use rusqlite::params;

use super::schema::RecoStore;
use super::types::TopArtistSeed;

impl RecoStore {
    // ---- Read APIs: albums / artists ----

    pub(super) fn get_recent_album_ids(&self, limit: u32) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT album_id, MAX(created_at) AS last_played
                FROM reco_events
                WHERE event_type = 'play' AND album_id IS NOT NULL
                GROUP BY album_id
                ORDER BY last_played DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare recent albums query: {}", e))?;
        let rows = stmt
            .query_map(params![limit], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query recent albums: {}", e))?;
        let mut albums = Vec::new();
        for row in rows {
            albums.push(row.map_err(|e| format!("Failed to read recent album row: {}", e))?);
        }
        Ok(albums)
    }

    pub(super) fn get_favorite_album_ids(&self, limit: u32) -> Result<Vec<String>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT album_id, MAX(created_at) AS last_favorite
                FROM reco_events
                WHERE event_type = 'favorite' AND album_id IS NOT NULL
                GROUP BY album_id
                ORDER BY last_favorite DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare favorite albums query: {}", e))?;
        let rows = stmt
            .query_map(params![limit], |row| row.get::<_, String>(0))
            .map_err(|e| format!("Failed to query favorite albums: {}", e))?;
        let mut albums = Vec::new();
        for row in rows {
            albums.push(row.map_err(|e| format!("Failed to read favorite album row: {}", e))?);
        }
        Ok(albums)
    }

    pub(super) fn get_top_artist_ids(&self, limit: u32) -> Result<Vec<TopArtistSeed>, String> {
        let mut stmt = self
            .conn
            .prepare(
                r#"
                SELECT artist_id, COUNT(*) AS play_count, MAX(created_at) AS last_played
                FROM reco_events
                WHERE event_type = 'play' AND artist_id IS NOT NULL
                GROUP BY artist_id
                ORDER BY play_count DESC, last_played DESC
                LIMIT ?
                "#,
            )
            .map_err(|e| format!("Failed to prepare top artists query: {}", e))?;
        let rows = stmt
            .query_map(params![limit], |row| {
                Ok(TopArtistSeed {
                    artist_id: row.get::<_, u64>(0)?,
                    play_count: row.get::<_, u32>(1)?,
                })
            })
            .map_err(|e| format!("Failed to query top artists: {}", e))?;
        let mut artists = Vec::new();
        for row in rows {
            artists.push(row.map_err(|e| format!("Failed to read top artist row: {}", e))?);
        }
        Ok(artists)
    }
}
