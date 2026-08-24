//! Remote MusicBrainz/Discogs metadata lookup: select, apply, and "open in
//! browser". Search lives in `remote_search.rs` (shares [`REMOTE_GEN`]).

use slint::Model;
use std::sync::atomic::{AtomicU64, Ordering};

use slint::{ComponentHandle, ModelRc, VecModel, Weak};

use crate::{AppWindow, TagEditorState};

/// Remote search/apply generation — a newer request supersedes a slow one.
pub(super) static REMOTE_GEN: AtomicU64 = AtomicU64::new(0);

/// Mark a result card selected.
pub fn select_result(weak: Weak<AppWindow>, provider_id: String) {
    let _ = weak.upgrade_in_event_loop(move |w| {
        w.global::<TagEditorState>().set_selected_result_id(provider_id.into());
    });
}

/// Fetch the selected result's full metadata and apply it (album fields +
/// positional per-track titles). Generation-guarded.
pub fn apply_remote(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    let s = w.global::<TagEditorState>();
    let id = s.get_selected_result_id().to_string();
    if id.is_empty() {
        return;
    }
    let provider = s.get_remote_provider_index();
    let gen = REMOTE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    s.set_remote_loading(true);

    handle.spawn(async move {
        let meta: Result<qbz_integrations::RemoteAlbumMetadata, String> = if provider == 1 {
            match id.parse::<u64>() {
                Ok(rid) => {
                    let dc = qbz_integrations::DiscogsClient::new();
                    dc.get_release_metadata(rid)
                        .await
                        .map(|m| qbz_integrations::discogs_full_to_metadata(&m))
                }
                Err(_) => Err("Invalid Discogs release id".to_string()),
            }
        } else {
            let mb = qbz_integrations::MusicBrainzClient::new();
            mb.get_release_with_tracks(&id)
                .await
                .map(|r| qbz_integrations::musicbrainz_full_to_metadata(&r))
                .map_err(|e| e.to_string())
        };
        let _ = weak.upgrade_in_event_loop(move |w| {
            if REMOTE_GEN.load(Ordering::SeqCst) != gen {
                return;
            }
            let s = w.global::<TagEditorState>();
            s.set_remote_loading(false);
            match meta {
                Ok(m) => {
                    s.set_album_title(m.title.clone().into());
                    s.set_album_artist(m.artist.clone().into());
                    if let Some(y) = m.year {
                        s.set_year_input(y.to_string().into());
                    }
                    if let Some(g) = m.genres.first() {
                        s.set_genre(g.clone().into());
                    }
                    if let Some(c) = m.catalog_number.as_ref() {
                        s.set_catalog_number(c.clone().into());
                    }
                    if m.disc_count > 0 {
                        s.set_album_total_discs(m.disc_count as i32);
                    }
                    // Positional per-track title merge.
                    let model = s.get_tracks();
                    let local_n = model.row_count();
                    let n = local_n.min(m.tracks.len());
                    for i in 0..n {
                        if let Some(mut row) = model.row_data(i) {
                            row.title = m.tracks[i].title.clone().into();
                            model.set_row_data(i, row);
                        }
                    }
                    s.set_show_remote_panel(false);
                    let remote_n = m.tracks.len();
                    if remote_n > 0 && remote_n != local_n {
                        crate::toast::warning(
                            &w,
                            qbz_i18n::t("Track count differs from the result; titles applied by position"),
                        );
                    }
                }
                Err(e) => {
                    let lower = e.to_lowercase();
                    if lower.contains("429") || lower.contains("rate") {
                        crate::toast::error(&w, qbz_i18n::t("Rate limited, try again shortly"));
                    } else {
                        crate::toast::error(&w, qbz_i18n::t("Failed to fetch metadata"));
                    }
                }
            }
        });
    });
}

/// Open the selected result's provider page in the system browser.
pub fn open_in_browser(weak: Weak<AppWindow>) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    let s = w.global::<TagEditorState>();
    let id = s.get_selected_result_id().to_string();
    if id.is_empty() {
        return;
    }
    let url = if s.get_remote_provider_index() == 1 {
        format!("https://www.discogs.com/release/{id}")
    } else {
        format!("https://musicbrainz.org/release/{id}")
    };
    let _ = open::that(url);
}
