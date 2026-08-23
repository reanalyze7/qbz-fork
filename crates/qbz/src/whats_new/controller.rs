//! `install`: wire the `WhatsNewActions` callbacks + apply a fetched release.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AppWindow, WhatsNewActions, WhatsNewBlock, WhatsNewState, WhatsNewTocEntry};

use super::fetch::{fetch_release_for_version, FetchedRelease};
use super::markdown::render_markdown;

/// Wire the `WhatsNewActions` callbacks. Call once at shell setup. `handle` runs
/// the network fetch off the UI thread.
pub fn install(window: &AppWindow, handle: tokio::runtime::Handle) {
    // close() — just hide.
    {
        let weak = window.as_weak();
        window.global::<WhatsNewActions>().on_close(move || {
            if let Some(w) = weak.upgrade() {
                w.global::<WhatsNewState>().set_open(false);
            }
        });
    }

    // open_url() — open a standalone release-notes link in the browser.
    window.global::<WhatsNewActions>().on_open_url(|url| {
        let url = url.to_string();
        if let Err(e) = open::that(&url) {
            log::warn!("[qbz-slint] whats-new open-url failed for {url}: {e}");
        }
    });

    // open() — show, mark loading, then fetch + render on a worker thread.
    {
        let weak = window.as_weak();
        let handle = handle.clone();
        window.global::<WhatsNewActions>().on_open(move || {
            let version = crate::about::app_version().to_string();
            // Paint the modal immediately in its loading state.
            if let Some(w) = weak.upgrade() {
                let st = w.global::<WhatsNewState>();
                st.set_open(true);
                st.set_loading(true);
                st.set_has_body(false);
                st.set_version(version.clone().into());
                st.set_date("".into());
                st.set_blocks(ModelRc::new(VecModel::from(Vec::<WhatsNewBlock>::new())));
                st.set_toc(ModelRc::new(VecModel::from(Vec::<WhatsNewTocEntry>::new())));
            }

            let weak = weak.clone();
            handle.spawn(async move {
                let fetched = fetch_release_for_version(&version).await;
                let _ = weak.upgrade_in_event_loop(move |w| apply(&w, fetched));
            });
        });
    }
}

/// Apply the fetched release (or its absence) to `WhatsNewState`. Runs on the UI
/// thread.
fn apply(window: &AppWindow, fetched: Option<FetchedRelease>) {
    let st = window.global::<WhatsNewState>();
    st.set_loading(false);

    let Some(rel) = fetched else {
        st.set_has_body(false);
        return;
    };

    st.set_version(rel.version.into());
    st.set_date(rel.date.into());

    let body = rel.body.unwrap_or_default();
    let (blocks, toc) = render_markdown(&body);
    if blocks.is_empty() {
        st.set_has_body(false);
        return;
    }

    st.set_blocks(ModelRc::new(VecModel::from(blocks)));
    st.set_toc(ModelRc::new(VecModel::from(toc)));
    st.set_has_body(true);
}
