use rusqlite::{params, Connection};

use super::model::{AlbumPlayMeta, AlbumPlayRow};

/// Insert one play event + upsert the album meta, at an explicit timestamp.
/// Internal so tests can drive it against an in-memory connection.
pub(crate) fn record_on(conn: &Connection, m: &AlbumPlayMeta, now: i64) {
    if let Err(e) = conn.execute(
        "INSERT INTO album_play_events (album_id, occurred_at) VALUES (?, ?)",
        params![m.album_id, now],
    ) {
        log::warn!("[qbz-slint] album_play_history insert event failed: {e}");
    }
    if let Err(e) = conn.execute(
        r#"
        INSERT INTO album_meta
            (album_id, title, artist, artist_id, artwork_url,
             quality_tier, quality_label, year, source, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        ON CONFLICT(album_id) DO UPDATE SET
            title = excluded.title,
            artist = excluded.artist,
            artist_id = excluded.artist_id,
            artwork_url = excluded.artwork_url,
            quality_tier = excluded.quality_tier,
            quality_label = excluded.quality_label,
            year = excluded.year,
            source = excluded.source,
            updated_at = excluded.updated_at
        "#,
        params![
            m.album_id,
            m.title,
            m.artist,
            m.artist_id,
            m.artwork_url,
            m.quality_tier,
            m.quality_label,
            m.year,
            m.source,
            now
        ],
    ) {
        log::warn!("[qbz-slint] album_play_history upsert meta failed: {e}");
    }
}

/// Rank albums by play count (desc), tie-broken by most-recent play so ties
/// are stable and intuitive. `limit` caps the carousel; `None` = full list.
pub(crate) fn query_on(conn: &Connection, limit: Option<u32>) -> Vec<AlbumPlayRow> {
    let sql = format!(
        r#"
        SELECT m.album_id, m.title, m.artist, m.artist_id, m.artwork_url,
               m.quality_tier, m.quality_label, m.year, m.source, p.plays
        FROM album_meta m
        JOIN (
            SELECT album_id, COUNT(*) AS plays, MAX(occurred_at) AS last_at
            FROM album_play_events
            GROUP BY album_id
        ) p ON p.album_id = m.album_id
        ORDER BY p.plays DESC, p.last_at DESC
        {}
        "#,
        limit.map(|n| format!("LIMIT {n}")).unwrap_or_default()
    );
    let out = (|| -> Option<Vec<AlbumPlayRow>> {
        let mut stmt = conn.prepare(&sql).ok()?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AlbumPlayRow {
                    album_id: row.get(0)?,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    artist_id: row.get(3)?,
                    artwork_url: row.get(4)?,
                    quality_tier: row.get(5)?,
                    quality_label: row.get(6)?,
                    year: row.get(7)?,
                    source: row.get(8)?,
                    plays: row.get::<_, i64>(9)? as u32,
                })
            })
            .ok()?;
        Some(rows.flatten().collect())
    })();
    out.unwrap_or_default()
}
