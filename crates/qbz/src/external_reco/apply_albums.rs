//! Per-row apply for the album rails (models built on the UI thread;
//! `slint::Image` is `!Send`).
use slint::ComponentHandle;

use qbz_external_reco::AlbumReco;
use slint::{ModelRc, VecModel};

use crate::artwork::{ArtworkJob, ArtworkTarget, ImageCache};
use crate::{AppWindow, DiscoverSection, ExternalRecoState};

use super::apply_rows::album_card;
use super::row_kinds::AlbumRow;

fn album_row_title(which: AlbumRow) -> String {
    match which {
        AlbumRow::RecAlbums => qbz_i18n::t("Recommended Albums"),
        AlbumRow::FreshReleases => qbz_i18n::t("Fresh Releases"),
        AlbumRow::DeepCuts => qbz_i18n::t("Deep cuts from artists you know"),
        AlbumRow::TopAlbums => qbz_i18n::t("Top albums on Qobuz"),
    }
}

pub(super) fn apply_albums(
    weak: &slint::Weak<AppWindow>,
    cache: &ImageCache,
    rows: Vec<AlbumReco>,
    which: AlbumRow,
) {
    let jobs: Vec<ArtworkJob> = rows
        .iter()
        .enumerate()
        .filter(|(_, a)| !a.artwork_url.is_empty())
        .map(|(i, a)| ArtworkJob {
            url: a.artwork_url.clone(),
            target: match which {
                AlbumRow::RecAlbums => ArtworkTarget::ExtRecoRecAlbum { index: i },
                AlbumRow::FreshReleases => ArtworkTarget::ExtRecoFreshAlbum { index: i },
                AlbumRow::DeepCuts => ArtworkTarget::ExtRecoDeepAlbum { index: i },
                AlbumRow::TopAlbums => ArtworkTarget::ExtRecoTopAlbum { index: i },
            },
        })
        .collect();
    let title = album_row_title(which);
    let w = weak.clone();
    let _ = w.upgrade_in_event_loop(move |w| {
        let section = DiscoverSection {
            title: title.into(),
            endpoint: "".into(),
            albums: ModelRc::new(VecModel::from(
                rows.iter().map(album_card).collect::<Vec<_>>(),
            )),
        };
        let s = w.global::<ExternalRecoState>();
        match which {
            AlbumRow::RecAlbums => {
                s.set_rec_albums(section);
                s.set_pending_rec_albums(false);
            }
            AlbumRow::FreshReleases => {
                s.set_fresh_releases(section);
                s.set_pending_fresh_releases(false);
            }
            AlbumRow::DeepCuts => {
                s.set_deep_cut_albums(section);
                s.set_pending_deep_cut_albums(false);
            }
            AlbumRow::TopAlbums => {
                s.set_top_albums(section);
                s.set_pending_top_albums(false);
            }
        }
    });
    crate::artwork::spawn_loads(jobs, weak.clone(), cache.clone());
}
