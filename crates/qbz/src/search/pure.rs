//! Pure formatting helpers used by the mappers.

use qbz_models::Playlist;

/// 24-bit and up is Hi-Res, anything else with depth info is CD-quality.
pub(crate) fn tier(bit_depth: Option<u32>) -> &'static str {
    match bit_depth {
        Some(depth) if depth >= 24 => "hires",
        Some(_) => "cd",
        None => "",
    }
}

/// Quality-badge tooltip, e.g. "Hi-Res 24-bit / 96 kHz". Empty when no
/// quality info is available.
pub(crate) fn quality_label(bit_depth: Option<u32>, sample_rate: Option<f64>) -> String {
    match bit_depth {
        None => String::new(),
        Some(depth) => {
            let prefix = if depth >= 24 { "Hi-Res" } else { "CD" };
            let rate = sample_rate.unwrap_or(if depth >= 24 { 96.0 } else { 44.1 });
            let rate = if rate.fract().abs() < f64::EPSILON {
                format!("{}", rate as i64)
            } else {
                format!("{rate}")
            };
            format!("{prefix} {depth}-bit / {rate} kHz")
        }
    }
}

/// `m:ss` track duration.
pub(crate) fn mmss(secs: u32) -> String {
    format!("{}:{:02}", secs / 60, secs % 60)
}

/// First four characters of an ISO date, or empty.
pub(crate) fn year_of(date: Option<&str>) -> String {
    date.and_then(|d| d.get(0..4)).unwrap_or("").to_string()
}

/// Up to four distinct cover URLs for a playlist collage. Qobuz returns
/// pre-built cover lists in `images300` / `images150` / `images`; the
/// highest-resolution non-empty list wins.
pub(crate) fn playlist_cover_urls(playlist: &Playlist) -> Vec<String> {
    let source = [
        &playlist.images300,
        &playlist.images150,
        &playlist.images,
    ]
    .into_iter()
    .flatten()
    .find(|v| !v.is_empty());

    let mut out: Vec<String> = Vec::new();
    if let Some(list) = source {
        for url in list {
            if !url.is_empty() && !out.contains(url) {
                out.push(url.clone());
            }
            if out.len() == 4 {
                break;
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mmss_pads_seconds() {
        assert_eq!(mmss(5), "0:05");
        assert_eq!(mmss(65), "1:05");
        assert_eq!(mmss(225), "3:45");
    }

    #[test]
    fn tier_classifies_bit_depth() {
        assert_eq!(tier(Some(24)), "hires");
        assert_eq!(tier(Some(16)), "cd");
        assert_eq!(tier(None), "");
    }

    #[test]
    fn quality_label_formats_known_quality() {
        assert_eq!(quality_label(Some(24), Some(96.0)), "Hi-Res 24-bit / 96 kHz");
        assert_eq!(quality_label(Some(16), Some(44.1)), "CD 16-bit / 44.1 kHz");
        assert_eq!(quality_label(None, None), "");
    }
}
