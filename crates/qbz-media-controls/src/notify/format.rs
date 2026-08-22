use super::NotificationMeta;

/// Format the quality line shown under the artist/album, identical to the Tauri
/// `v2_format_notification_quality`. Empty string = omit the line.
pub(super) fn format_quality(bit_depth: Option<u32>, sample_rate: Option<f64>) -> String {
    match (bit_depth, sample_rate) {
        (Some(bits), Some(rate)) if bits >= 24 || rate > 48.0 => {
            let rate_str = if rate.fract() == 0.0 {
                format!("{}", rate as u32)
            } else {
                format!("{rate}")
            };
            format!("Hi-Res - {bits}-bit/{rate_str}kHz")
        }
        (Some(bits), Some(rate)) => {
            let rate_str = if rate.fract() == 0.0 {
                format!("{}", rate as u32)
            } else {
                format!("{rate}")
            };
            format!("CD Quality - {bits}-bit/{rate_str}kHz")
        }
        _ => String::new(),
    }
}

/// Build the notification body: "artist · album" then a quality line.
/// `·` (middle dot) on macOS, `•` (bullet) elsewhere — matches Tauri.
pub(super) fn build_body(meta: &NotificationMeta) -> String {
    let separator = if cfg!(target_os = "macos") {
        " \u{00b7} "
    } else {
        " \u{2022} "
    };
    let mut lines = Vec::new();
    let mut line1 = Vec::new();
    if !meta.artist.is_empty() {
        line1.push(meta.artist.clone());
    }
    if !meta.album.is_empty() {
        line1.push(meta.album.clone());
    }
    if !line1.is_empty() {
        lines.push(line1.join(separator));
    }
    let quality = format_quality(meta.bit_depth, meta.sample_rate);
    if !quality.is_empty() {
        lines.push(quality);
    }
    lines.join("\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_quality_hi_res() {
        assert_eq!(format_quality(Some(24), Some(96.0)), "Hi-Res - 24-bit/96kHz");
    }

    #[test]
    fn format_quality_cd() {
        assert_eq!(
            format_quality(Some(16), Some(44.1)),
            "CD Quality - 16-bit/44.1kHz"
        );
    }

    #[test]
    fn format_quality_missing_is_empty() {
        assert_eq!(format_quality(None, None), "");
    }

    #[test]
    fn build_body_joins_artist_album_and_quality() {
        let meta = NotificationMeta {
            title: "Song".into(),
            artist: "Artist".into(),
            album: "Album".into(),
            bit_depth: Some(24),
            sample_rate: Some(96.0),
            art_url: None,
        };
        let body = build_body(&meta);
        assert!(body.contains("Artist"));
        assert!(body.contains("Album"));
        assert!(body.contains("Hi-Res"));
    }

    #[test]
    fn build_body_empty_meta_is_empty() {
        assert_eq!(build_body(&NotificationMeta::default()), "");
    }
}
