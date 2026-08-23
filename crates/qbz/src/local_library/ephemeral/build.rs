//! Build the ephemeral album-grouped pane from scanned tracks.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, EphemeralAlbum, LocalLibraryState};

use crate::local_library::tracks::map::map_local_track;

/// Last path segment (folder name) for the header.
pub(crate) fn folder_display_name(path: &str) -> String {
    std::path::Path::new(path)
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| path.to_string())
}

/// Load a cover from a cached artwork path (strips an optional `file://`).
/// Decoded to the rows tier (the album header renders it at 56px) so the
/// ephemeral model never retains full-resolution sources.
fn load_cover(path: &Option<String>) -> slint::Image {
    let Some(p) = path.as_deref().filter(|s| !s.is_empty()) else {
        return slint::Image::default();
    };
    let p = p.strip_prefix("file://").unwrap_or(p);
    crate::artwork::load_local_cover(p, 96).unwrap_or_default()
}

/// Group ephemeral tracks into album blocks (sorted by title), each with its
/// cover + tracks. Returns the blocks and whether the session spans >1 album.
/// MUST run on the UI thread — it loads `slint::Image`s (not Send).
pub(crate) fn build_ephemeral_albums(
    tracks: &[qbz_library::LocalTrack],
) -> (Vec<EphemeralAlbum>, bool) {
    use std::collections::BTreeMap;
    // Preserve scan order within a group; key order is stabilized by title sort.
    let mut groups: BTreeMap<String, Vec<qbz_library::LocalTrack>> = BTreeMap::new();
    for t in tracks {
        groups
            .entry(crate::ephemeral::ephemeral_album_key(t))
            .or_default()
            .push(t.clone());
    }
    let multi = groups.len() > 1;
    let mut albums: Vec<EphemeralAlbum> = groups
        .into_iter()
        .map(|(key, group)| {
            let first = &group[0];
            let title = if first.album_group_title.is_empty() {
                first.album.clone()
            } else {
                first.album_group_title.clone()
            };
            let artist = first
                .album_artist
                .clone()
                .unwrap_or_else(|| first.artist.clone());
            let count = group.len();
            let track_count_label =
                qbz_i18n::tf("{} track", "{} tracks", count as i64, &[&count.to_string()]);
            let meta = match first.year {
                Some(y) if y > 0 => format!("{y} · {track_count_label}"),
                _ => track_count_label,
            };
            let tier = if first.format.to_string().eq_ignore_ascii_case("mp3") {
                "mp3"
            } else {
                match first.bit_depth {
                    Some(b) if b >= 24 => "hires",
                    Some(_) => "cd",
                    None => "",
                }
            };
            let is_cue = first.cue_file_path.is_some() || first.cue_start_secs.is_some();
            let artwork = load_cover(&first.artwork_path);
            let items: Vec<crate::TrackItem> = group.into_iter().map(map_local_track).collect();
            EphemeralAlbum {
                group_key: key.into(),
                title: title.into(),
                artist: artist.into(),
                meta: meta.into(),
                quality_tier: tier.into(),
                is_cue,
                artwork,
                tracks: ModelRc::new(VecModel::from(items)),
            }
        })
        .collect();
    albums.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
    (albums, multi)
}

/// Push a scanned ephemeral result onto the UI (UI thread). `focus` switches to
/// the Folders tab (true for an explicit open; false on startup rehydrate so we
/// don't hijack the landing view).
pub(crate) fn apply_ephemeral(
    window: &AppWindow,
    name: &str,
    path: &str,
    tracks: &[qbz_library::LocalTrack],
    focus: bool,
) {
    let (albums, multi) = build_ephemeral_albums(tracks);
    let s = window.global::<LocalLibraryState>();
    s.set_ephemeral_active(true);
    s.set_ephemeral_loading(false);
    s.set_ephemeral_name(name.into());
    s.set_ephemeral_path(path.into());
    s.set_ephemeral_track_count(tracks.len() as i32);
    s.set_ephemeral_multi_album(multi);
    s.set_ephemeral_albums(ModelRc::new(VecModel::from(albums)));
    if focus {
        s.set_active_tab("folders".into());
    }
}

