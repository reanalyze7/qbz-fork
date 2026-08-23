use crate::session_store::model::{
    PersistedPlaybackSession, PersistedQueueTrack, PersistedSessionSnapshot,
    PersistedShellViewState,
};
use crate::session_store::schema::SessionStore;

impl SessionStore {
    pub fn load_session(&self) -> Result<PersistedSessionSnapshot, String> {
        let (
            current_index,
            current_position_secs,
            volume,
            shuffle_enabled,
            repeat_mode,
            was_playing,
            saved_at,
            last_view,
            view_context_id,
            view_context_type,
        ): (
            Option<i64>,
            i64,
            f64,
            i64,
            String,
            i64,
            i64,
            String,
            Option<String>,
            Option<String>,
        ) = self
            .conn
            .query_row(
                "SELECT current_index, current_position_secs, volume, shuffle_enabled, repeat_mode, was_playing, saved_at, last_view, view_context_id, view_context_type
                 FROM player_state WHERE id = 1",
                [],
                |row| {
                    Ok((
                        row.get(0)?,
                        row.get(1)?,
                        row.get(2)?,
                        row.get(3)?,
                        row.get(4)?,
                        row.get(5)?,
                        row.get(6)?,
                        row.get::<_, String>(7)
                            .unwrap_or_else(|_| "home".to_string()),
                        row.get(8)?,
                        row.get(9)?,
                    ))
                },
            )
            .map_err(|e| format!("Failed to load player state: {}", e))?;

        let mut stmt = self.conn
            .prepare("SELECT track_id, title, artist, album, duration_secs, artwork_url, hires, bit_depth, sample_rate, is_local, album_id, artist_id, source, streamable, parental_warning, source_item_id_hint FROM queue_tracks ORDER BY position")
            .map_err(|e| format!("Failed to prepare queue query: {}", e))?;

        let tracks: Vec<PersistedQueueTrack> = stmt
            .query_map([], |row| {
                Ok(PersistedQueueTrack {
                    id: row.get::<_, i64>(0)? as u64,
                    title: row.get(1)?,
                    artist: row.get(2)?,
                    album: row.get(3)?,
                    duration_secs: row.get::<_, i64>(4)? as u64,
                    artwork_url: row.get(5)?,
                    hires: row.get::<_, i64>(6).unwrap_or(0) != 0,
                    bit_depth: row.get::<_, Option<i64>>(7)?.map(|v| v as u32),
                    sample_rate: row.get(8)?,
                    is_local: row.get::<_, i64>(9).unwrap_or(0) != 0,
                    album_id: row.get(10)?,
                    artist_id: row.get::<_, Option<i64>>(11)?.map(|v| v as u64),
                    source: row.get(12)?,
                    streamable: row.get::<_, i64>(13).unwrap_or(1) != 0,
                    parental_warning: row.get::<_, i64>(14).unwrap_or(0) != 0,
                    source_item_id_hint: row.get(15)?,
                })
            })
            .map_err(|e| format!("Failed to query queue tracks: {}", e))?
            .filter_map(|result| result.ok())
            .collect();

        Ok(PersistedSessionSnapshot {
            playback: PersistedPlaybackSession {
                queue_tracks: tracks,
                current_index: current_index.map(|i| i as usize),
                current_position_secs: current_position_secs as u64,
                volume: volume as f32,
                shuffle_enabled: shuffle_enabled != 0,
                repeat_mode,
                was_playing: was_playing != 0,
                saved_at,
            },
            shell_view: PersistedShellViewState {
                last_view,
                view_context_id,
                view_context_type,
            },
        })
    }
}
