//! Apply a cached version's tracks to `LocalAlbumState` + resolve helpers.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, TrackItem};

use super::state::{album_query, album_versions, fmt_album_duration};
use crate::local_library::tracks::map::map_local_track;

/// Apply version `index` of the open album to LocalAlbumState (tracks, header,
/// quality). Reads the cached versions; no DB round-trip. The cover is
/// album-level (set once by `open_local_album`), so it is NOT touched here.
pub fn apply_album_version(window: &AppWindow, index: i32) {
    let versions = album_versions();
    let Some((_, tracks)) = versions.get(index as usize) else {
        return;
    };
    let s = window.global::<crate::LocalAlbumState>();
    let group_key = s.get_id().to_string();
    let title = tracks
        .first()
        .map(|t| t.album_group_title.clone())
        .unwrap_or_default();
    let artist_of =
        |t: &qbz_library::LocalTrack| t.album_artist.clone().unwrap_or_else(|| t.artist.clone());
    let artist = match tracks.first() {
        Some(first) => {
            let name = artist_of(first);
            if tracks.iter().all(|t| artist_of(t) == name) {
                name
            } else {
                qbz_i18n::t("Various Artists")
            }
        }
        None => String::new(),
    };
    let total_secs: u64 = tracks.iter().map(|t| t.duration_secs).sum();
    // Distinct track artists, first-appearance order — the collapsable
    // multi-artist header list (spec §B2). Raw `artist` (NOT album_artist):
    // a compilation's 10 track artists are the list's whole point.
    let mut all_artists: Vec<slint::SharedString> = Vec::new();
    for t in tracks {
        let a = t.artist.trim();
        if !a.is_empty() && !all_artists.iter().any(|x| x.as_str() == a) {
            all_artists.push(a.into());
        }
    }
    let track_count = qbz_i18n::tf("{} track", "{} tracks", tracks.len() as i64, &[&tracks.len().to_string()]);
    let info_line = format!("{} · {}", track_count, fmt_album_duration(total_secs));
    let (tier, detail) = match tracks.iter().max_by_key(|t| t.bit_depth.unwrap_or(0)) {
        Some(t) => {
            // Same shared classifier as the card + rows (badge), so the header
            // matches them; un-hydrated lossless → generic "FLAC".
            let (tier, detail, _) =
                crate::quality::badge(&t.format.to_string(), t.bit_depth, Some(t.sample_rate));
            (tier.to_string(), detail)
        }
        None => (String::new(), String::new()),
    };
    // Client-side filter (Qobuz album view parity): match title/artist; the
    // header badge/info stay album-level (computed from the full version above).
    let q = album_query().to_lowercase();
    let shown: Vec<&qbz_library::LocalTrack> = tracks
        .iter()
        .filter(|t| {
            q.is_empty()
                || t.title.to_lowercase().contains(&q)
                || t.artist.to_lowercase().contains(&q)
        })
        .collect();
    // Multi-disc grouping (mirrors the Qobuz album view): the album is
    // multi-disc when its shown tracks span more than one distinct disc
    // number, and only then do we stamp "Disc N" headers on the first row of
    // each disc run. Local tracks are already disc-then-track sorted upstream
    // (album_versions sorts by (disc_number, track_number)).
    let is_multi_disc = {
        let mut seen: Option<u32> = None;
        let mut multi = false;
        for t in &shown {
            let disc = t.disc_number.unwrap_or(1);
            match seen {
                Some(d) if d != disc => {
                    multi = true;
                    break;
                }
                _ => seen = Some(disc),
            }
        }
        multi
    };
    let mut prev_disc: Option<u32> = None;
    let items: Vec<TrackItem> = shown
        .into_iter()
        .map(|t| {
            let disc = t.disc_number.unwrap_or(1);
            let disc_header_number = if is_multi_disc && prev_disc != Some(disc) {
                disc as i32
            } else {
                0
            };
            prev_disc = Some(disc);
            let mut it = map_local_track(t.clone());
            it.album_id = group_key.clone().into();
            it.disc_header_number = disc_header_number;
            it
        })
        .collect();
    s.set_title(title.into());
    s.set_artist(artist.into());
    s.set_all_artists(ModelRc::new(VecModel::from(all_artists)));
    s.set_info_line(info_line.into());
    s.set_quality_tier(tier.into());
    s.set_quality_detail(detail.into());
    s.set_tracks(ModelRc::new(VecModel::from(items)));
    s.set_version_index(index);
}
