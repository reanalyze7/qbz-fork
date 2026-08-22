//! v2 CMAF-bundle-specific column read/write.

use rusqlite::params;

use super::schema::OfflineCacheDb;

/// Raw snapshot of the v2 bundle columns for a cached track.
///
/// `cache_format` tells the caller how to interpret the rest:
/// - `1` — legacy plain FLAC at `segments_path`; other fields empty.
/// - `2` — raw CMAF bundle; all fields populated.
#[derive(Debug, Clone)]
pub struct CmafBundleRow {
    pub cache_format: u8,
    pub segments_path: String,
    pub init_path: Option<String>,
    pub content_key_wrapped: Option<Vec<u8>>,
    pub infos_wrapped: Option<Vec<u8>>,
    pub format_id: Option<u32>,
    pub n_segments: Option<u32>,
}

impl OfflineCacheDb {
    /// Persist the CMAF-specific columns for a track after it was
    /// successfully downloaded as a raw encrypted bundle.
    ///
    /// `file_path` here is the concatenated-segments file (or primary
    /// segment file, depending on how the caller lays out the bundle on
    /// disk). `init_path` is the init.mp4. Both keys are already wrapped
    /// by the caller via `qbz-secrets::SecretBox::wrap`.
    #[allow(clippy::too_many_arguments)]
    pub fn set_cmaf_bundle(
        &self,
        track_id: u64,
        segments_path: &str,
        init_path: &str,
        content_key_wrapped: &[u8],
        infos_wrapped: &[u8],
        format_id: u32,
        n_segments: u32,
        total_bytes: u64,
    ) -> Result<(), String> {
        self.conn
            .execute(
                "UPDATE cached_tracks
                    SET cache_format = 2,
                        file_path = ?1,
                        init_path = ?2,
                        content_key_wrapped = ?3,
                        infos_wrapped = ?4,
                        format_id = ?5,
                        n_segments = ?6,
                        file_size_bytes = ?7
                    WHERE track_id = ?8",
                params![
                    segments_path,
                    init_path,
                    content_key_wrapped,
                    infos_wrapped,
                    format_id as i64,
                    n_segments as i64,
                    total_bytes as i64,
                    track_id as i64,
                ],
            )
            .map_err(|e| format!("Failed to write CMAF bundle fields: {}", e))?;
        Ok(())
    }

    /// Read back the bundle fields for a track, for offline playback.
    pub fn get_cmaf_bundle(&self, track_id: u64) -> Result<Option<CmafBundleRow>, String> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT cache_format, file_path, init_path, content_key_wrapped,
                        infos_wrapped, format_id, n_segments
                   FROM cached_tracks
                  WHERE track_id = ?1",
            )
            .map_err(|e| format!("Failed to prepare CMAF bundle select: {}", e))?;
        let row: Result<CmafBundleRow, _> = stmt.query_row(params![track_id as i64], |row| {
            Ok(CmafBundleRow {
                cache_format: row.get::<_, i64>(0)? as u8,
                segments_path: row.get(1)?,
                init_path: row.get::<_, Option<String>>(2)?,
                content_key_wrapped: row.get::<_, Option<Vec<u8>>>(3)?,
                infos_wrapped: row.get::<_, Option<Vec<u8>>>(4)?,
                format_id: row.get::<_, Option<i64>>(5)?.map(|v| v as u32),
                n_segments: row.get::<_, Option<i64>>(6)?.map(|v| v as u32),
            })
        });
        match row {
            Ok(r) => Ok(Some(r)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(format!("Failed to read CMAF bundle: {}", e)),
        }
    }
}
