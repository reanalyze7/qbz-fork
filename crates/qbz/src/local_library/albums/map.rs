//! Local -> rendered mapping for the Albums tab.

/// Map one local album row to the shared plain `AlbumCard`. Kept as the
/// Send-safe plain struct (NOT `AlbumCardItem`, which holds a non-Send
/// `slint::Image`) so it can cross the `spawn_blocking` boundary; the
/// conversion to `AlbumCardItem` happens on the UI thread via
/// `album_map::to_item`. Genre is intentionally empty (the local DB carries
/// genre per-track, not per-album); the cover PATH rides on `artwork_url`
/// as the artwork-job carrier (the grid renders `artwork`, not the url).
pub fn map_local_album(a: qbz_library::LocalAlbum) -> crate::album_map::AlbumCard {
    // Format-first classification (mirrors Tauri): a lossy format (MP3) gets
    // the dedicated MP3 badge tier, never CD.
    // One shared classifier (see crate::quality::badge) so the card, the
    // album-detail header and the track rows can never disagree. `a.sample_rate`
    // is Hz; `badge` normalizes it to kHz (guarded).
    let (tier, quality_detail, quality_label) =
        crate::quality::badge(&a.format.to_string(), a.bit_depth, Some(a.sample_rate));
    let year = a.year.map(|y| y.to_string()).unwrap_or_default();
    let track_count = if a.track_count > 0 {
        a.track_count.to_string()
    } else {
        String::new()
    };
    // Real source for the SOURCE column + the always-visible card badge:
    // user files -> local, offline copies -> qobuz_download.
    let source = match a.source.as_str() {
        "qobuz_download" => "qobuz_download",
        _ => "local",
    }
    .to_string();
    crate::album_map::AlbumCard {
        id: a.id,
        title: a.title,
        artist: a.artist,
        artist_id: String::new(),
        genre: String::new(),
        year: year.clone(),
        quality_tier: tier.to_string(),
        quality_label,
        artwork_url: a.artwork_path.unwrap_or_default(),
        // Local albums carry no Qobuz label id — they never join the
        // per-label library index.
        label_id: String::new(),
        release_type: crate::album_map::classify_release_type(Some(a.track_count)).to_string(),
        source,
        quality_detail,
        track_count,
        plain_year: year,
    }
}

/// Format a local album's quality. `sample_rate_hz` is Hz (44100.0); the
/// detail is the bare "24-bit / 96 kHz" (QualityBadgeFull) and the label is
/// the grid badge tooltip "Hi-Res: 24-bit / 96 kHz".
pub(crate) fn local_quality(bit_depth: Option<u32>, sample_rate_hz: f64) -> (String, String) {
    let Some(bd) = bit_depth else {
        return (String::new(), String::new());
    };
    // DSD tracks store bit_depth = 1 and the DSD bit rate as sample_rate.
    if bd == 1 {
        let label = crate::quality::dsd_multiple_label(Some(sample_rate_hz));
        let tooltip = qbz_i18n::t_args("DSD: {}", &[&label]);
        return (label, tooltip);
    }
    let khz = if sample_rate_hz >= 1000.0 {
        sample_rate_hz / 1000.0
    } else {
        sample_rate_hz
    };
    let khz_str = if khz.fract().abs() < 0.05 {
        format!("{}", khz.round() as i64)
    } else {
        format!("{khz:.1}")
    };
    let prefix = if bd >= 24 { "Hi-Res" } else { "CD" };
    (
        format!("{bd}-bit / {khz_str} kHz"),
        format!("{prefix}: {bd}-bit / {khz_str} kHz"),
    )
}
