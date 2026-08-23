//! Small text-formatting helpers shared by the album/track mappers.

/// Last.fm path segment: percent-encode, then render spaces as `+` (Last.fm's
/// `/music/{artist}/{album}` paths use `+` for spaces, like Tauri's link
/// builder). `urlencoding::encode` already emits `%20` for spaces, so swap
/// them — the remaining percent-escapes (e.g. `/`, `?`) stay path-safe.
pub(in crate::album) fn lastfm_segment(text: &str) -> String {
    urlencoding::encode(text).replace("%20", "+")
}

/// Truncate text to at most `max` characters, cutting back to the last
/// word boundary and appending an ellipsis. Returns the text unchanged
/// when it is already short enough.
pub(super) fn truncate_words(text: &str, max: usize) -> String {
    if text.chars().count() <= max {
        return text.to_string();
    }
    let truncated: String = text.chars().take(max).collect();
    let cut = truncated.rfind(' ').unwrap_or(truncated.len());
    format!("{}…", truncated[..cut].trim_end())
}

/// `Xh Ym` / `Ym` album duration.
pub(in crate::album) fn format_duration(secs: u32) -> String {
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    if hours > 0 {
        format!("{hours}h {minutes}m")
    } else {
        format!("{minutes}m")
    }
}
