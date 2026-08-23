//! Artwork job builders for the sidebar's playlist collage covers.

use slint::{ComponentHandle, Model};

use crate::{AppWindow, SidebarState};

/// Build artwork-download jobs for every playlist row's collage covers,
/// targeting `SidebarState.entries` by row index. Call after `apply` /
/// `rebuild` updates the entries.
/// Returns `(qobuz_jobs, local_jobs)`. Qobuz playlist covers are http(s) URLs
/// (HTTP cache loader); LOCAL playlist covers are filesystem paths (the
/// local loader). They're split by the row's `local_kind` so
/// each set goes to the right loader — a file path sent through the HTTP loader
/// would silently fail to decode.
pub fn artwork_jobs(
    window: &AppWindow,
) -> (Vec<crate::artwork::ArtworkJob>, Vec<crate::artwork::ArtworkJob>) {
    let mut qobuz_jobs = Vec::new();
    let mut local_jobs = Vec::new();
    let entries = window.global::<SidebarState>().get_entries();
    for idx in 0..entries.row_count() {
        let Some(e) = entries.row_data(idx) else { continue };
        if e.kind != "playlist" {
            continue;
        }
        let is_local = !e.local_kind.is_empty();
        let urls = [e.url1, e.url2, e.url3, e.url4];
        for (slot, url) in urls.iter().enumerate() {
            if !url.is_empty() {
                let job = crate::artwork::ArtworkJob {
                    target: crate::artwork::ArtworkTarget::SidebarPlaylistCover { idx, slot },
                    url: url.to_string(),
                };
                if is_local {
                    local_jobs.push(job);
                } else {
                    qobuz_jobs.push(job);
                }
            }
        }
    }
    (qobuz_jobs, local_jobs)
}
