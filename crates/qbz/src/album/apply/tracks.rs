//! Build the `TrackItem` row model for `apply_album`: multi-disc + work
//! run-length header grouping, plus the per-row blacklist stamp.

use crate::TrackItem;

use super::super::data::TrackData;

/// Build the Slint track rows for one album, stamping disc/work headers and
/// the blacklist flag. `album_id` is the row's clickable-album-link target;
/// `album_artist_id` is the fallback blacklist key for rows with no own
/// performer id (Task 6: `track.artist_id ?? album.artist_id`).
pub(super) fn build_track_items(
    tracks: Vec<TrackData>,
    album_id: &slint::SharedString,
    album_artist_id: &str,
) -> Vec<TrackItem> {
    // Multi-disc grouping (Tauri PurchaseAlbumDetailView: groupByDisc +
    // isMultiDisc). The album is "multi-disc" when its tracks span more than
    // one distinct disc number; only then do we emit "Disc N" headers. The
    // header is stamped on the first track of each disc run, and tracks stay
    // in their delivered order (Qobuz returns them disc-then-track ordered).
    let is_multi_disc = {
        let mut seen: Option<u32> = None;
        let mut multi = false;
        for track in &tracks {
            match seen {
                Some(d) if d != track.disc => {
                    multi = true;
                    break;
                }
                _ => seen = Some(track.disc),
            }
        }
        multi
    };
    let mut prev_disc: Option<u32> = None;
    // Run-length work grouping (PR #536): the header is stamped on the first
    // row of each consecutive same-work run, mirroring the disc grouping above.
    // Albums with no work metadata leave every header "" → flat list, unchanged.
    let mut prev_work: Option<String> = None;
    tracks
        .into_iter()
        .map(|track| {
            // Stamp the disc number on the first row of each disc run, but
            // only for multi-disc albums (single-disc renders flat → 0). The
            // delegate renders `@tr("Disc") <n>` above the row when this is
            // > 0, matching the Tauri `{$t('album.disc')} {discNum}` markup.
            let disc_header_number = if is_multi_disc && prev_disc != Some(track.disc) {
                track.disc as i32
            } else {
                0
            };
            prev_disc = Some(track.disc);
            // Work header on the first row of each consecutive same-work run;
            // an empty work resets the run so a later same-named work re-heads.
            let work_header = if !track.work.is_empty()
                && prev_work.as_deref() != Some(track.work.as_str())
            {
                track.work.clone()
            } else {
                String::new()
            };
            // Composer (name + id) accompanies the header only on its leading row.
            let (work_composer_name, work_composer_id) = if work_header.is_empty() {
                (String::new(), String::new())
            } else {
                (
                    track.work_composer_name.clone(),
                    track.work_composer_id.clone(),
                )
            };
            prev_work = if track.work.is_empty() {
                None
            } else {
                Some(track.work.clone())
            };
            // Blacklist key: the row's own performer id, falling back to the
            // album's primary artist when the track carries none (Task 6).
            // NOTE: the album-track row model (`TrackData`) does NOT carry a
            // composer id — only performer/album-primary — so the composer leg
            // of the D-FEAT predicate is not available here. The album queue
            // builder filters off the raw `Track` (which DOES carry composer)
            // via `track_is_blacklisted_full`, so play-all still honors
            // composer; only this row greyout is performer/album-primary-only.
            let row_artist_id = if track.artist_id.is_empty() {
                album_artist_id
            } else {
                track.artist_id.as_str()
            };
            let is_blacklisted = crate::artist_blacklist::stamp_row(
                "qobuz",
                &[row_artist_id],
                Some(album_id.as_str()),
            );
            TrackItem {
                id: track.id.clone().into(),
                number: track.number.into(),
                title: track.title.into(),
                artist: track.artist.into(),
                album: "".into(),
                duration: track.duration.into(),
                quality_tier: track.quality_tier.into(),
                quality_detail: track.quality_detail.into(),
                explicit: track.explicit,
                selected: false,
                artwork_url: "".into(),
                artwork: slint::Image::default(),
                is_favorite: crate::fav_cache::is_favorite(&track.id),
                artist_id: track.artist_id.into(),
                album_id: album_id.clone(),
                is_blacklisted,
                removing: false,
                cache_status: if crate::offline_cache::is_cached(&track.id) { 3 } else { 0 },
                cache_progress: 0.0,
                // Qobuz album-detail rows; local albums override via map_local_track.
                source: "qobuz".into(),
                unlocking: false,
                disc_header_number,
                work_header: work_header.into(),
                work_composer_name: work_composer_name.into(),
                work_composer_id: work_composer_id.into(),
            }
        })
        .collect()
}
