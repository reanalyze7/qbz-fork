//! Collapse normalized-equal artist spellings into merged rows.

use super::normalize::normalize_artist;

/// Build the per-normalized-artist set of album ids, so merged rows get an
/// accurate unique album count independent of per-track spelling. Mirrors
/// Tauri's `artistAlbumIds`.
pub(crate) fn build_artist_album_ids(
    albums: &[qbz_library::LocalAlbum],
) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
    let mut map: std::collections::HashMap<String, std::collections::HashSet<String>> =
        std::collections::HashMap::new();
    for al in albums {
        if !al.all_artists.is_empty() {
            for part in al.all_artists.split(',') {
                let n = normalize_artist(part);
                if n.is_empty() || n == "various artists" {
                    continue;
                }
                map.entry(n).or_default().insert(al.id.clone());
            }
        } else {
            let n = normalize_artist(&al.artist);
            if !n.is_empty() && n != "various artists" {
                map.entry(n).or_default().insert(al.id.clone());
            }
        }
    }
    map
}

/// Send-safe merged-artist row (no `slint::Image`, so it can cross the
/// `spawn_blocking` boundary). Converted to `LocalArtistItem` on the UI thread
/// in `apply_artists` (where the non-Send decoded `image` is added).
pub(crate) struct ArtistRow {
    pub(crate) name: String,
    pub(crate) display_name: String,
    pub(crate) album_count: i32,
    pub(crate) track_count: i32,
    pub(crate) image_path: String,
}

/// Collapse normalized-equal artist spellings into one canonical row and
/// attach accurate album counts + a custom-image path. Mirrors Tauri's
/// `artistMergeResult`: canonical = the variant with most albums (tie: most
/// tracks); merged track count = sum across variants.
pub(crate) fn merge_artists(
    artists: Vec<qbz_library::LocalArtist>,
    albums: &[qbz_library::LocalAlbum],
    custom_images: &std::collections::HashMap<String, String>,
) -> Vec<ArtistRow> {
    let album_ids = build_artist_album_ids(albums);
    let norm_imgs: std::collections::HashMap<String, String> = custom_images
        .iter()
        .map(|(k, v)| (normalize_artist(k), v.clone()))
        .collect();

    let mut groups: std::collections::HashMap<String, Vec<qbz_library::LocalArtist>> =
        std::collections::HashMap::new();
    let mut order: Vec<String> = Vec::new();
    for a in artists {
        let n = normalize_artist(&a.name);
        if n.is_empty() {
            continue;
        }
        if !groups.contains_key(&n) {
            order.push(n.clone());
        }
        groups.entry(n).or_default().push(a);
    }

    let mut out: Vec<ArtistRow> = Vec::with_capacity(order.len());
    for n in order {
        let variants = match groups.remove(&n) {
            Some(v) => v,
            None => continue,
        };
        let album_set_len = album_ids.get(&n).map(|s| s.len()).unwrap_or(0) as i32;
        let (canonical, album_count, track_count) = if variants.len() == 1 {
            let v = &variants[0];
            let ac = if album_set_len > 0 {
                album_set_len
            } else {
                v.album_count as i32
            };
            (v.name.clone(), ac, v.track_count as i32)
        } else {
            let canon = variants
                .iter()
                .max_by(|a, b| {
                    a.album_count
                        .cmp(&b.album_count)
                        .then(a.track_count.cmp(&b.track_count))
                })
                .unwrap();
            let total_tracks: u32 = variants.iter().map(|v| v.track_count).sum();
            let ac = if album_set_len > 0 {
                album_set_len
            } else {
                canon.album_count as i32
            };
            (canon.name.clone(), ac, total_tracks as i32)
        };
        // Portrait: custom/cached (incl. previously-fetched Qobuz). A
        // filesystem album cover decodes as a local file, routed through
        // `spawn_local_loads` at dispatch time.
        let image_path = norm_imgs.get(&n).cloned().unwrap_or_default();
        out.push(ArtistRow {
            name: canonical.clone(),
            display_name: canonical,
            album_count,
            track_count,
            image_path,
        });
    }
    out.sort_by(|a, b| {
        a.display_name
            .to_lowercase()
            .cmp(&b.display_name.to_lowercase())
    });
    out
}
