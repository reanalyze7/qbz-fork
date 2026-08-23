//! `Album` → plain-struct mapping (worker thread).

use qbz_models::Album;
use qbz_qobuz::performers::parse_performers;

use super::format::{album_duration, album_quality, format_title, full_release_date, roles_suffix};
use super::types::{AlbumCreditsData, AlbumTrackData, PerformerData};

pub(super) fn map_album_credits(album: Album) -> AlbumCreditsData {
    let album_artist = album.artist.name.clone();

    let (label, label_id) = match album.label.as_ref() {
        Some(l) => (l.name.clone(), l.id.to_string()),
        None => (String::new(), String::new()),
    };

    let raw_tracks = album
        .tracks
        .as_ref()
        .map(|c| c.items.clone())
        .unwrap_or_default();

    let track_count = album
        .tracks_count
        .or(album.track_count)
        .unwrap_or(raw_tracks.len() as u32);

    // "Hard Rock · 10 tracks · 1h 21m" — gated on a genre, 1:1 with Tauri.
    let meta_line = match album.genre.as_ref().filter(|g| !g.name.is_empty()) {
        Some(g) => {
            // Tauri always appends formatAlbumDuration(album.duration || 0),
            // so an absent duration shows "· 0m" rather than dropping it.
            let parts = vec![
                g.name.clone(),
                qbz_i18n::tf("{} track", "{} tracks", track_count as i64, &[&track_count.to_string()]),
                album_duration(album.duration.unwrap_or(0)),
            ];
            parts.join(" · ")
        }
        None => String::new(),
    };

    let tracks = raw_tracks
        .into_iter()
        .enumerate()
        .map(|(index, t)| {
            let performers: Vec<PerformerData> = parse_performers(
                t.performers.as_deref().unwrap_or_default(),
            )
            .into_iter()
            .map(|p| PerformerData {
                roles: roles_suffix(&p.roles),
                primary_role: p
                    .roles
                    .first()
                    .cloned()
                    .unwrap_or_else(|| qbz_i18n::t("Performer")),
                name: p.name,
            })
            .collect();
            let copyright = t
                .copyright
                .as_deref()
                .map(crate::strip_html::decode_html_entities)
                .unwrap_or_default();
            let number = if t.track_number > 0 {
                t.track_number.to_string()
            } else {
                (index + 1).to_string()
            };
            let artist = t
                .performer
                .as_ref()
                .map(|a| a.name.clone())
                .filter(|n| !n.is_empty())
                .unwrap_or_else(|| {
                    if album_artist.is_empty() {
                        qbz_i18n::t("Unknown Artist")
                    } else {
                        album_artist.clone()
                    }
                });
            AlbumTrackData {
                id: t.id.to_string(),
                number,
                title: format_title(&t.title, t.version.as_deref()),
                artist,
                has_credits: !performers.is_empty() || !copyright.is_empty(),
                performers,
                copyright,
            }
        })
        .collect();

    let review = album
        .description
        .as_deref()
        .map(crate::strip_html::strip_html)
        .unwrap_or_default();
    let has_review = !review.trim().is_empty();

    AlbumCreditsData {
        title: album.title,
        artist: album_artist,
        label,
        label_id,
        release_date: full_release_date(album.release_date_original.as_deref()),
        meta_line,
        quality: album_quality(album.maximum_bit_depth, album.maximum_sampling_rate),
        review,
        has_review,
        tracks,
    }
}
