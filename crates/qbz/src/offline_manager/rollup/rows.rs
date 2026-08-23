//! Build the flat interleaved album/track row list for the sorted +
//! filtered album order.

use qbz_offline_cache::OfflineCacheStatus;

use super::super::filters::Filters;
use super::super::format::{album_size, cover_path, fmt_duration, track_status_int, COVER_DECODE_SIZE};
use super::super::row::RowData;
use super::albums::{sorted_order, AlbumsMap};

pub(super) fn build(order: &[String], albums: &AlbumsMap, cache_path: &str, f: &Filters) -> Vec<RowData> {
    let order = sorted_order(order, albums, f.sort);

    let mut rows: Vec<RowData> = Vec::new();
    for aid in &order {
        let (artist, title, group) = &albums[aid];
        if !f.selected_artist.is_empty() && *artist != f.selected_artist {
            continue;
        }
        let any_failed = group
            .iter()
            .any(|t| matches!(t.status, OfflineCacheStatus::Failed));
        if f.show_only_failed && !any_failed {
            continue;
        }
        let any_active = group.iter().any(|t| {
            matches!(
                t.status,
                OfflineCacheStatus::Queued | OfflineCacheStatus::Downloading
            )
        });
        let all_ready = group
            .iter()
            .all(|t| matches!(t.status, OfflineCacheStatus::Ready));
        let album_status = if any_failed {
            4
        } else if any_active {
            2
        } else if all_ready {
            3
        } else {
            0
        };
        // First track whose cover resolves — within an album only some
        // tracks may carry one (per-track CMAF folders, mixed v1/v2 rows).
        let cover_path = group
            .iter()
            .map(|t| cover_path(cache_path, t))
            .find(|p| !p.is_empty())
            .unwrap_or_default();
        rows.push(RowData {
            kind: "album",
            album_id: aid.clone(),
            track_id: String::new(),
            title: title.clone(),
            subtitle: artist.clone(),
            meta: qbz_i18n::t_args(
                "{} tracks · {}",
                &[&group.len().to_string(), &super::super::human_size(album_size(group))],
            ),
            status: album_status,
            progress: 0.0,
            cover: crate::artwork::decode_local_pixels(
                &cover_path,
                crate::artwork::scaled_decode(COVER_DECODE_SIZE),
            ),
            number: String::new(),
        });
        for (i, t) in group.iter().enumerate() {
            if f.show_only_failed && !matches!(t.status, OfflineCacheStatus::Failed) {
                continue;
            }
            rows.push(RowData {
                kind: "track",
                album_id: aid.clone(),
                track_id: t.track_id.to_string(),
                title: t.title.clone(),
                subtitle: t.artist.clone(),
                meta: fmt_duration(t.duration_secs),
                status: track_status_int(&t.status),
                progress: t.progress_percent as f32 / 100.0,
                cover: None,
                number: (i + 1).to_string(),
            });
        }
    }

    rows
}
