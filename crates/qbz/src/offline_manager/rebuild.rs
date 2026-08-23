//! `rebuild`: read the index.db, roll it up, and push it to
//! `OfflineManagerState`.

use qbz_offline_cache::CachedTrackInfo;
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{AppWindow, OfflineManagerState, OfflineRow};

use super::filters::current_filters;
use super::format::human_size;
use super::rollup;
use super::GB;

/// Read the index.db, build the artist→album→track rollup + stats (applying
/// the current toolbar filters), and push them to `OfflineManagerState`.
/// `pub` so the cache mutation fns refresh the manager after their DB op.
pub async fn rebuild(weak: slint::Weak<AppWindow>) {
    let f = current_filters();
    let off = crate::offline::get().await;
    let (tracks, limit, cache_path): (Vec<CachedTrackInfo>, Option<u64>, String) = match off {
        Some(ref o) => {
            let limit = *o.limit_bytes.lock().await;
            let cp = o.get_cache_path();
            let guard = o.db.lock().await;
            let tracks = guard
                .as_ref()
                .and_then(|db| db.get_all_tracks().ok())
                .unwrap_or_default();
            (tracks, limit, cp)
        }
        None => (Vec::new(), None, String::new()),
    };

    let total_size: u64 = tracks.iter().map(|t| t.file_size_bytes).sum();
    let tracks_count = tracks.len() as i32;

    let rollup::Rollup { artists, rows } = rollup::build(tracks, &cache_path, &f);

    let (limit_text, usage, limit_gb) = match limit {
        Some(l) if l > 0 => (
            qbz_i18n::t_args("· of {}", &[&human_size(l)]),
            (total_size as f32 / l as f32).clamp(0.0, 1.0),
            (l / GB).max(1) as i32,
        ),
        _ => (qbz_i18n::t("· Unlimited"), 0.0, 5),
    };
    let size_text = human_size(total_size);

    let _ = weak.upgrade_in_event_loop(move |w| {
        let st = w.global::<OfflineManagerState>();
        // Build OfflineRow on the UI thread (slint::Image is not Send, so it
        // can't be built on the worker); the pixels were decoded there.
        let offline_rows: Vec<OfflineRow> = rows
            .into_iter()
            .map(|rd| OfflineRow {
                kind: rd.kind.into(),
                album_id: rd.album_id.into(),
                track_id: rd.track_id.into(),
                title: rd.title.into(),
                subtitle: rd.subtitle.into(),
                meta: rd.meta.into(),
                status: rd.status,
                progress: rd.progress,
                cover: rd
                    .cover
                    .map(|(px, pw, ph)| crate::artwork::pixels_to_image(&px, pw, ph))
                    .unwrap_or_default(),
                number: rd.number.into(),
                selected: false,
            })
            .collect();
        st.set_rows(ModelRc::new(VecModel::from(offline_rows)));
        st.set_selected_count(0);
        st.set_artists(ModelRc::new(VecModel::from(artists)));
        st.set_tracks_count(tracks_count);
        st.set_size_text(SharedString::from(size_text));
        st.set_limit_text(SharedString::from(limit_text));
        st.set_usage(usage);
        st.set_limit_gb(limit_gb);
        st.set_selected_artist(SharedString::from(f.selected_artist));
        st.set_sort_index(f.sort);
        st.set_show_only_failed(f.show_only_failed);
        st.set_loading(false);
    });
}
