//! Assembles every value `refresh_now_playing_meta` needs to push, from the
//! current queue track.

use super::fields_types::MetaFields;
use super::hydrate::spawn_quality_hydration;
use super::quality_fields::resolve_quality;
use super::super::Runtime;
use crate::AppWindow;
use qbz_models::QueueTrack;

/// Build [`MetaFields`] for `track` and fire the quality-hydration fetch
/// when the badge fields are missing (fire-and-forget — see
/// [`spawn_quality_hydration`]).
pub(super) fn build_meta_fields(
    runtime: &Runtime,
    weak: &slint::Weak<AppWindow>,
    track: &QueueTrack,
) -> MetaFields {
    let title = match track.version.as_deref().filter(|v| !v.is_empty()) {
        Some(version) => format!("{} ({version})", track.title),
        None => track.title.clone(),
    };
    let artist = track.artist.clone();
    let album = track.album.clone();
    // Album with its release variant appended ("Octavarium (2009 Remaster)") for
    // the now-playing bar, MPRIS, and the desktop notification. Scrobbling keeps
    // the CLEAN `album` (below) so Last.fm doesn't fragment stats across editions.
    let album_display = match track
        .album_version
        .as_deref()
        .map(str::trim)
        .filter(|v| !v.is_empty())
    {
        Some(v) => format!("{album} ({v})"),
        None => album.clone(),
    };
    let album_id = track.album_id.clone().unwrap_or_default();
    let artist_id = track.artist_id.map(|id| id.to_string()).unwrap_or_default();
    // "Playing from" origin for the now-playing song-card layers button. Derived
    // from the CURRENT track's own per-track stamp (context_kind/context_id set
    // at enqueue time by stamp_queue_context) and re-published on EVERY track
    // change below — so the button always carries the right source for the track
    // that is actually playing and is never a stale single global. When the
    // track carries no container origin (bare single-track / favorites / mix /
    // search / restored-session play) it falls back to the track's own album.
    let (context_kind, context_id) = match (
        track.context_kind.as_deref().filter(|s| !s.is_empty()),
        track.context_id.as_deref().filter(|s| !s.is_empty()),
    ) {
        (Some(kind), Some(id)) => (kind.to_string(), id.to_string()),
        _ => ("album".to_string(), album_id.clone()),
    };
    let track_id_num = track.id;
    let track_id = track.id.to_string();
    // Ephemeral tracks have no DB row → metadata-bound actions (favorite,
    // add-to-playlist, track-info) are gated off in the UI via this flag.
    let is_ephemeral = crate::ephemeral::is_ephemeral_id(track.id as i64);
    // Normalized source for the UI ("qobuz" | "local" | ...). Qobuz
    // tracks coerce a None source to "qobuz"; local tracks to "local". Gates the
    // Qobuz-only Track-info trigger.
    let source = track
        .source
        .clone()
        .unwrap_or_else(|| if track.is_local { "local" } else { "qobuz" }.to_string());
    // Active-row alias for offline-cache tracks: the local-library row id (the
    // queue id is the Qobuz catalog id there, so the row's track-id binding
    // never matches). Only trusted for qobuz_download — other builders reuse
    // the hint for unrelated things (album id).
    let local_track_id = if source == "qobuz_download" {
        track.source_item_id_hint.clone().unwrap_or_default()
    } else {
        String::new()
    };
    let duration = track.duration_secs;
    // Same `artwork_ref()` value feeds the now-playing bar, MPRIS
    // (`to_mpris_url`), and the desktop notification.
    let artwork = track.artwork_ref();
    let bar_artwork = artwork.clone();
    // Higher-res cover for the hover preview that floats above the bar art. Same
    // value; the high-res comes from the larger decode in
    // load_now_playing_artwork_large.
    let preview_artwork = artwork.clone();

    let q = resolve_quality(track);
    spawn_quality_hydration(
        runtime,
        weak,
        track_id_num,
        q.governed,
        track.bit_depth.is_none() && track.sample_rate.is_none(),
    );

    // Seed the "+" flyout's album-collection entry from the favorite-album
    // cache — the SAME source the card/header toggles flip — so the entry
    // renders add vs remove honestly for the new track. Kept live between
    // track changes by set_album_row_favorite (main.rs).
    let album_favorite = crate::fav_cache::is_album_favorite(&album_id);

    MetaFields {
        title,
        artist,
        album,
        album_display,
        album_id,
        artist_id,
        context_kind,
        context_id,
        track_id_num,
        track_id,
        is_ephemeral,
        source,
        local_track_id,
        duration,
        bar_artwork,
        preview_artwork,
        quality_tier: q.quality_tier,
        quality_detail: q.quality_detail,
        bit_depth: q.bit_depth,
        sample_rate: q.sample_rate,
        album_favorite,
    }
}
