//! Pure string/enum helpers used across the detail controller.

use qbz_models::mixtape::{
    AlbumSource, CollectionKind, CollectionPlayMode, ItemType, MixtapeCollectionItem,
};

pub(super) fn kind_str(kind: CollectionKind) -> &'static str {
    match kind {
        CollectionKind::Mixtape => "mixtape",
        CollectionKind::Collection => "collection",
        CollectionKind::ArtistCollection => "artist_collection",
    }
}

/// Eyebrow label (Tauri `kindLabel`): mixtapes.label / collections.artistLabel
/// / collections.label, uppercased to match the grid card eyebrow.
pub(super) fn kind_label(kind: CollectionKind) -> String {
    match kind {
        CollectionKind::Mixtape => qbz_i18n::t("MIXTAPE"),
        CollectionKind::ArtistCollection => qbz_i18n::t("ARTIST"),
        CollectionKind::Collection => qbz_i18n::t("COLLECTION"),
    }
}

pub(super) fn play_mode_str(mode: CollectionPlayMode) -> &'static str {
    match mode {
        CollectionPlayMode::InOrder => "in_order",
        CollectionPlayMode::AlbumShuffle => "album_shuffle",
    }
}

pub fn source_str(source: AlbumSource) -> &'static str {
    match source {
        AlbumSource::Qobuz => "qobuz",
        AlbumSource::Local => "local",
    }
}

pub fn item_type_str(t: ItemType) -> &'static str {
    match t {
        ItemType::Album => "album",
        ItemType::Track => "track",
        ItemType::Playlist => "playlist",
    }
}

/// `mixtapes.albumCount` ICU plural — always "album(s)" regardless of
/// item_type (1:1 with the PSD / the grid card meta).
pub(super) fn album_count_label(count: usize) -> String {
    qbz_i18n::tf("{} album", "{} albums", count as i64, &[&count.to_string()])
}

/// Type-cell label, uppercase (spec 12 §6.3 col-3 `itemTypeLabel`). Release-type
/// overrides (album rows showing EP/Single/…) are a later slice — albums render
/// "ALBUM" here.
pub(super) fn type_label(t: ItemType) -> String {
    match t {
        ItemType::Album => qbz_i18n::t("ALBUM"),
        ItemType::Track => qbz_i18n::t("TRACK"),
        ItemType::Playlist => qbz_i18n::t("PLAYLIST"),
    }
}

/// TRACKS column (spec 12 §6.3 col-6 `itemTracks`): "1" for a track, else the
/// count or an em-dash.
pub(super) fn tracks_text(item: &MixtapeCollectionItem) -> String {
    match item.item_type {
        ItemType::Track => "1".to_string(),
        _ => match item.track_count {
            Some(n) => n.to_string(),
            None => "—".to_string(),
        },
    }
}

/// YEAR column (spec 12 §6.3 col-7 `itemYear`): the year or "".
pub(super) fn year_text(item: &MixtapeCollectionItem) -> String {
    item.year.map(|y| y.to_string()).unwrap_or_default()
}

/// Stable per-item key for the inline-tracks cache (`source|source_item_id`).
/// `source_item_id` alone is the row's logical key, but pairing it with the
/// source keeps qobuz-vs-local collisions impossible.
pub(super) fn inline_cache_key(source: &str, source_item_id: &str) -> String {
    format!("{source}|{source_item_id}")
}

/// "m:ss" track duration (spec 12 §8 `formatSec`). A zero/missing duration
/// renders the placeholder "--:--" (NOT "0:00") so an unresolved length reads as
/// unknown, matching the Tauri formatter. (`duration_secs` is `u64`, so the
/// "negative" case collapses to the zero case.)
pub(super) fn track_duration_str(secs: u64) -> String {
    if secs == 0 {
        "--:--".to_string()
    } else {
        format!("{}:{:02}", secs / 60, secs % 60)
    }
}

/// Title + parenthesized Qobuz version suffix (spec 12 §8 `formatTrackTitle`).
pub(super) fn inline_track_title(track: &qbz_models::QueueTrack) -> String {
    match track.version.as_deref().filter(|v| !v.is_empty()) {
        Some(version) => format!("{} ({version})", track.title),
        None => track.title.clone(),
    }
}
