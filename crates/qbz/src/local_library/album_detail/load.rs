//! Load a local album (dedicated LocalAlbumView), splitting its tracks into
//! versions by source directory.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::TrackItem;

use super::apply::apply_album_version;
use super::state::{album_versions, set_album_query, version_label, version_rank, version_source};

/// Set the album track filter and re-render the current version in place.
pub fn search_album(weak: slint::Weak<crate::AppWindow>, query: String) {
    set_album_query(query);
    let _ = weak.upgrade_in_event_loop(|w| {
        let index = w.global::<crate::LocalAlbumState>().get_version_index();
        apply_album_version(&w, index);
    });
}

/// Load a local album (dedicated LocalAlbumView), splitting its tracks into
/// VERSIONS by source directory so multiple copies don't merge into a
/// duplicate-track list. Applies the best-quality version first; the picker
/// switches versions in place. Does NOT touch nav — the caller sets the view.
pub fn open_local_album(
    weak: slint::Weak<crate::AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: ImageCache,
    group_key: String,
) {
    // Fresh album → clear any leftover track filter from the previous one.
    set_album_query(String::new());
    let _ = weak.upgrade_in_event_loop(|w| {
        let s = w.global::<crate::LocalAlbumState>();
        s.set_loading(true);
        s.set_tracks(ModelRc::new(VecModel::from(Vec::<TrackItem>::new())));
        s.set_versions(ModelRc::new(VecModel::from(Vec::<crate::LocalAlbumVersion>::new())));
        s.set_version_index(0);
        s.set_cover(slint::Image::default());
    });
    let gk = group_key.clone();
    handle.spawn(async move {
        let tracks = tokio::task::spawn_blocking(move || {
            let mut t = crate::local_library::shared::fetch_album_tracks_blocking(&gk);
            // Backfill covers from cover.jpg/folder.jpg on disk (the DB may not
            // have an artwork_path even when a cover sits in the folder).
            crate::playback::fill_missing_covers(&mut t);
            t
        })
        .await
        .unwrap_or_default();
        // Group by source directory (LocalTrack.album_group_key = the dir key).
        let mut groups: std::collections::HashMap<String, Vec<qbz_library::LocalTrack>> =
            std::collections::HashMap::new();
        let mut order: Vec<String> = Vec::new();
        for t in tracks {
            let key = t.album_group_key.clone();
            if !groups.contains_key(&key) {
                order.push(key.clone());
            }
            groups.entry(key).or_default().push(t);
        }
        let mut versions: Vec<(String, Vec<qbz_library::LocalTrack>)> = order
            .into_iter()
            .filter_map(|k| {
                groups.remove(&k).map(|mut v| {
                    v.sort_by_key(|t| (t.disc_number.unwrap_or(1), t.track_number.unwrap_or(0)));
                    (k, v)
                })
            })
            .collect();
        // Best quality first (so the default selection is the highest-res copy).
        versions.sort_by(|a, b| {
            let qa = a.1.iter().map(version_rank).max().unwrap_or((0, 0));
            let qb = b.1.iter().map(version_rank).max().unwrap_or((0, 0));
            qb.cmp(&qa)
        });
        // (label, source) per version — best-quality first (already sorted).
        let infos: Vec<(String, String)> = versions
            .iter()
            .map(|(_, v)| (version_label(v), version_source(v)))
            .collect();
        // Album cover = the FIRST version (best quality first) that has a cover
        // on disk; fall through versions; else empty (placeholder). Album-level
        // + stable across version switches.
        let album_cover = versions
            .iter()
            .find_map(|(_, v)| {
                v.iter()
                    .find_map(|t| t.artwork_path.clone().filter(|p| !p.is_empty()))
            })
            .unwrap_or_default();
        *album_versions() = versions;
        let _ = weak.upgrade_in_event_loop(move |w| {
            let s = w.global::<crate::LocalAlbumState>();
            s.set_id(group_key.into());
            let vlist: Vec<crate::LocalAlbumVersion> = infos
                .into_iter()
                .map(|(label, source)| crate::LocalAlbumVersion {
                    label: label.into(),
                    source: source.into(),
                })
                .collect();
            s.set_versions(ModelRc::new(VecModel::from(vlist)));
            s.set_loading(false);
            s.set_cover_url(album_cover.clone().into());
            apply_album_version(&w, 0);
            // Decode the album cover once (stable across version switches).
            if !album_cover.is_empty() {
                crate::artwork::spawn_local_loads(
                    vec![ArtworkJob {
                        target: ArtworkTarget::LocalAlbumViewCover,
                        url: album_cover,
                    }],
                    w.as_weak(),
                    image_cache,
                );
            }
        });
    });
}
