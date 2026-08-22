//! Row-mapping and small string helpers shared by many `database` query
//! submodules (tracks, albums, search, folder_tree, ...). Kept as an
//! `impl LibraryDatabase` block (rather than free functions) so every other
//! submodule can keep calling `Self::row_to_track` / `Self::parse_format`
//! unchanged, exactly as it did before the file was split. `pub(super)`
//! makes them visible to every sibling submodule under `crate::database`.

use crate::AudioFormat;

use super::LibraryDatabase;

impl LibraryDatabase {
    /// Column list for SELECT queries (avoids fragile SELECT * with positional indices)
    pub(super) const TRACK_COLUMNS: &'static str = "id, file_path, title, artist, album, album_artist, \
         track_number, disc_number, year, genre, duration_secs, format, \
         bit_depth, sample_rate, channels, file_size_bytes, \
         cue_file_path, cue_start_secs, cue_end_secs, artwork_path, \
         last_modified, indexed_at, album_group_key, album_group_title, \
         source, qobuz_track_id, catalog_number, is_network_mount";

    /// Convert a database row to LocalTrack
    pub(super) fn row_to_track(row: &rusqlite::Row) -> rusqlite::Result<crate::LocalTrack> {
        Ok(crate::LocalTrack {
            id: row.get(0)?,                                                          // id
            file_path: row.get(1)?,                                                   // file_path
            title: row.get(2)?,                                                       // title
            artist: row.get(3)?,                                                      // artist
            album: row.get(4)?,                                                       // album
            album_artist: row.get(5)?,   // album_artist
            track_number: row.get(6)?,   // track_number
            disc_number: row.get(7)?,    // disc_number
            year: row.get(8)?,           // year
            genre: row.get(9)?,          // genre
            duration_secs: row.get(10)?, // duration_secs
            format: Self::parse_format(&row.get::<_, String>(11)?), // format
            bit_depth: row.get(12)?,     // bit_depth
            sample_rate: row.get::<_, f64>(13)?, // sample_rate
            channels: row.get(14)?,      // channels
            file_size_bytes: row.get(15)?, // file_size_bytes
            cue_file_path: row.get(16)?, // cue_file_path
            cue_start_secs: row.get(17)?, // cue_start_secs
            cue_end_secs: row.get(18)?,  // cue_end_secs
            artwork_path: row.get(19)?,  // artwork_path
            last_modified: row.get(20)?, // last_modified
            indexed_at: row.get(21)?,    // indexed_at
            album_group_key: row.get::<_, Option<String>>(22)?.unwrap_or_default(), // album_group_key
            album_group_title: row.get::<_, Option<String>>(23)?.unwrap_or_default(), // album_group_title
            source: row.get(24).ok().flatten(),                                       // source
            qobuz_track_id: row.get(25).ok().flatten(), // qobuz_track_id
            catalog_number: row.get(26).ok().flatten(), // catalog_number
            is_network_mount: row
                .get::<_, Option<i64>>(27)
                .ok()
                .flatten()
                .map(|v| v != 0)
                .unwrap_or(false),
        })
    }

    /// Parse format string to AudioFormat
    pub(super) fn parse_format(s: &str) -> AudioFormat {
        match s.to_uppercase().as_str() {
            "FLAC" => AudioFormat::Flac,
            "ALAC" => AudioFormat::Alac,
            "WAV" => AudioFormat::Wav,
            "AIFF" => AudioFormat::Aiff,
            "APE" => AudioFormat::Ape,
            "MP3" => AudioFormat::Mp3,
            _ => AudioFormat::Unknown,
        }
    }
}

/// Escape `%`, `_` and `\` characters so the input can be embedded as a
/// LIKE pattern fragment. Pair with `LIKE ?n || '/%' ESCAPE '\'` at the
/// SQL site. Used by [`LibraryDatabase::list_folder_children`] and
/// [`LibraryDatabase::list_folder_tracks`] to defend against
/// pattern-injection on filesystem paths that legitimately contain
/// metacharacters (a track named `100%.flac`, a folder containing
/// `_intro_`, etc.).
pub(super) fn escape_like_pattern(input: &str) -> String {
    input
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}
