//! Shared helper for parsing JSPF `identifier` fields, used by both playlist
//! endpoints (`playlists.rs` and `playlist_tracks.rs`).

/// Extract the last `/`-delimited segment of a JSPF `identifier` value.
///
/// ListenBrainz returns the `identifier` either as a single string
/// (`"https://listenbrainz.org/playlist/{mbid}"`) or as an array of such
/// strings. Returns the last non-empty path segment of the first usable value,
/// or `None` when nothing parseable is present.
pub(super) fn last_identifier_segment(identifier: &serde_json::Value) -> Option<String> {
    fn last_segment(raw: &str) -> Option<String> {
        raw.trim_end_matches('/')
            .rsplit('/')
            .next()
            .map(str::trim)
            .filter(|segment| !segment.is_empty())
            .map(|segment| segment.to_string())
    }

    match identifier {
        serde_json::Value::String(raw) => last_segment(raw),
        serde_json::Value::Array(items) => items
            .iter()
            .filter_map(serde_json::Value::as_str)
            .find_map(last_segment),
        _ => None,
    }
}
