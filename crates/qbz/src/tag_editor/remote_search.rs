//! Remote MusicBrainz/Discogs search.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, ModelRc, VecModel, Weak};

use crate::{AppWindow, TagEditorState};

use super::remote::REMOTE_GEN;

pub(super) fn map_search(r: &qbz_integrations::RemoteAlbumSearchResult) -> crate::RemoteResultItem {
    let provider = if matches!(r.provider, qbz_integrations::RemoteProvider::Discogs) {
        "discogs"
    } else {
        "musicbrainz"
    };
    crate::RemoteResultItem {
        provider: provider.into(),
        provider_id: r.provider_id.clone().into(),
        title: r.title.clone().into(),
        artist: r.artist.clone().into(),
        year: r.year.unwrap_or(0) as i32,
        has_year: r.year.is_some(),
        track_count: r.track_count.unwrap_or(0) as i32,
        has_track_count: r.track_count.is_some(),
        country: r.country.clone().unwrap_or_default().into(),
        format: r.format.clone().unwrap_or_default().into(),
        label: r.label.clone().unwrap_or_default().into(),
        catalog_number: r.catalog_number.clone().unwrap_or_default().into(),
    }
}

/// Search the selected provider (MusicBrainz/Discogs) for the current album
/// title + artist. Generation-guarded so a slow reply can't clobber a newer one.
pub fn search_remote(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let Some(w) = weak.upgrade() else {
        return;
    };
    let s = w.global::<TagEditorState>();
    let title = s.get_album_title().trim().to_string();
    let artist = s.get_album_artist().trim().to_string();
    let provider = s.get_remote_provider_index();
    if title.is_empty() && artist.is_empty() {
        crate::toast::error_weak(&weak, qbz_i18n::t("Enter a title or artist to search"));
        return;
    }
    let gen = REMOTE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    s.set_remote_searching(true);

    handle.spawn(async move {
        let results: Result<Vec<crate::RemoteResultItem>, String> = if provider == 1 {
            let dc = qbz_integrations::DiscogsClient::new();
            dc.search_releases(&artist, &title, None, 12).await.map(|v| {
                v.iter()
                    .map(|r| map_search(&qbz_integrations::discogs_extended_to_search_result(r)))
                    .collect()
            })
        } else {
            let mb = qbz_integrations::MusicBrainzClient::new();
            mb.search_releases_extended(&title, &artist, None, 12)
                .await
                .map(|resp| {
                    resp.releases
                        .iter()
                        .map(|r| map_search(&qbz_integrations::musicbrainz_release_to_search_result(r)))
                        .collect()
                })
                .map_err(|e| e.to_string())
        };
        let _ = weak.upgrade_in_event_loop(move |w| {
            if REMOTE_GEN.load(Ordering::SeqCst) != gen {
                return;
            }
            let s = w.global::<TagEditorState>();
            s.set_remote_searching(false);
            s.set_has_searched(true);
            match results {
                Ok(items) => {
                    let empty = items.is_empty();
                    s.set_remote_results(ModelRc::new(VecModel::from(items)));
                    s.set_show_remote_panel(!empty);
                }
                Err(e) => {
                    s.set_show_remote_panel(false);
                    crate::toast::error(&w, qbz_i18n::t_args("Search failed: {}", &[&e]));
                }
            }
        });
    });
}
