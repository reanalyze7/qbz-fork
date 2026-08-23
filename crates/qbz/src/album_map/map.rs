//! `map_album`: decode a Qobuz `Album` payload into an `AlbumCard`.

use qbz_models::Album;

use super::tier::{classify_release_type, tier_hires};
use super::{format_album_title, AlbumCard};

/// Map a decoded Qobuz album into an `AlbumCard`.
///
/// Prefers the V2 nested shape (`audio_info` / `dates` / `track_count` /
/// `artists[]`) returned by `/label/getAlbums` and the discover feeds,
/// falling back to the flat fields used by the favorites (`legacy`)
/// payload. The fallbacks resolve to the flat values when the nested
/// fields are absent, so favorites albums map losslessly too.
pub fn map_album(album: Album) -> AlbumCard {
    let bit_depth = album
        .audio_info
        .as_ref()
        .and_then(|a| a.maximum_bit_depth)
        .or(album.maximum_bit_depth);
    let sample_rate = album
        .audio_info
        .as_ref()
        .and_then(|a| a.maximum_sampling_rate)
        .or(album.maximum_sampling_rate);
    let quality_tier = tier_hires(bit_depth, album.hires || album.hires_streamable).to_string();
    let quality_label = match (bit_depth, sample_rate) {
        (Some(bd), Some(sr)) => format!("{}-bit / {} kHz", bd, sr),
        _ => String::new(),
    };
    let quality_detail = quality_label.clone();

    // Release date — nested `dates` first (original > download > stream),
    // else the flat `release_date_original`.
    let date = album
        .dates
        .as_ref()
        .and_then(|d| {
            d.original
                .clone()
                .or_else(|| d.download.clone())
                .or_else(|| d.stream.clone())
        })
        .or_else(|| album.release_date_original.clone());
    let year = crate::dates::release_label(date.as_deref());
    let plain_year = date
        .as_deref()
        .and_then(|s| s.get(..4).map(|y| y.to_string()))
        .unwrap_or_default();

    let tc = album.track_count.or(album.tracks_count);
    let track_count = tc
        .filter(|n| *n > 0)
        .map(|n| n.to_string())
        .unwrap_or_default();
    let release_type = release_type_label(album.release_type.as_deref(), tc);
    // Borrow `album` (artist + versioned title) before any owned field moves.
    let (artist, artist_id) = album_artist(&album);
    let title = format_album_title(&album.title, album.version.as_deref());
    let genre = album.genre.map(|g| g.name).unwrap_or_default();
    AlbumCard {
        id: album.id,
        title,
        artist,
        artist_id,
        genre,
        year,
        quality_tier,
        quality_label,
        artwork_url: album.image.best().cloned().unwrap_or_default(),
        label_id: album
            .label
            .as_ref()
            .map(|l| l.id.to_string())
            .unwrap_or_default(),
        release_type,
        // Qobuz album surfaces (Discover / Favorites / Label) hide the SOURCE
        // column and the badge, so leave it empty (preserves prior behavior).
        source: String::new(),
        quality_detail,
        track_count,
        plain_year,
    }
}

/// Display label for the TYPE column — the explicit `release_type` when the
/// payload provides a known one, else a track-count heuristic.
fn release_type_label(release_type: Option<&str>, track_count: Option<u32>) -> String {
    match release_type {
        Some("album") | Some("download") => qbz_i18n::t("Album"),
        Some("ep") | Some("epSingle") => qbz_i18n::t("EP"),
        Some("single") => qbz_i18n::t("Single"),
        Some("live") => qbz_i18n::t("Live"),
        Some("compilation") => qbz_i18n::t("Compilation"),
        _ => qbz_i18n::t(classify_release_type(track_count)),
    }
}

/// Album artist name + id. Many /label/getAlbums items leave the `artist`
/// object empty and only populate the `artists` credit array, which left
/// group-by-artist showing "Unknown Artist". Fall back to the main-artist
/// credit (else the first) — mirrors artist.rs map_release.
fn album_artist(album: &Album) -> (String, String) {
    if !album.artist.name.is_empty() {
        return (album.artist.name.clone(), album.artist.id.to_string());
    }
    if let Some(list) = album.artists.as_ref() {
        let pick = list
            .iter()
            .find(|a| {
                a.roles
                    .as_ref()
                    .map(|r| r.iter().any(|role| role == "main-artist"))
                    .unwrap_or(false)
            })
            .or_else(|| list.first());
        if let Some(a) = pick {
            return (a.name.clone(), a.id.to_string());
        }
    }
    (String::new(), String::new())
}
