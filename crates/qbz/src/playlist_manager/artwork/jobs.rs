//! Artwork jobs for the grid/list/tree covers (targeting
//! `PlaylistManagerState.playlists`/`.tree` by row index).

use slint::{ComponentHandle, Model};

use crate::artwork::{ArtworkJob, ArtworkTarget};
use crate::{AppWindow, PlaylistManagerState};

pub fn artwork_jobs(window: &AppWindow) -> Vec<ArtworkJob> {
    let state = window.global::<PlaylistManagerState>();
    let mut jobs = Vec::new();

    let playlists = state.get_playlists();
    for index in 0..playlists.row_count() {
        let Some(p) = playlists.row_data(index) else {
            continue;
        };
        for (slot, url) in [p.url1, p.url2, p.url3, p.url4].iter().enumerate() {
            if !url.is_empty() {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::PmPlaylistCover { index, slot },
                    url: url.to_string(),
                });
            }
        }
    }

    let tree = state.get_tree();
    for index in 0..tree.row_count() {
        let Some(row) = tree.row_data(index) else {
            continue;
        };
        if row.kind != "playlist" {
            continue;
        }
        let p = row.playlist;
        for (slot, url) in [p.url1, p.url2, p.url3, p.url4].iter().enumerate() {
            if !url.is_empty() {
                jobs.push(ArtworkJob {
                    target: ArtworkTarget::PmTreeCover { index, slot },
                    url: url.to_string(),
                });
            }
        }
    }
    jobs
}
