//! Public entry points: `open_track_info`, `load_track_info_inline`,
//! `open_album_credits`.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use qbz_core::FrontendAdapter;
use slint::ComponentHandle;

use crate::{AlbumInfoState, AppWindow, TrackInfoState};

use super::apply::{apply_album_credits, apply_track_info};
use super::map_album::map_album_credits;
use super::map_track::map_track_info;

/// Shared fetch+map+apply for Track Info. `open_modal` decides whether the
/// floating modal is shown: `true` for the explicit (i)-button flow, `false`
/// for the immersive split panel which renders the same data inline (and so
/// must NOT pop the overlay over the immersive view).
fn spawn_track_info<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    track_id: u64,
    open_modal: bool,
) where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let _ = weak.upgrade_in_event_loop(move |w| {
        let st = w.global::<TrackInfoState>();
        st.set_error("".into());
        st.set_loading(true);
        if open_modal {
            st.set_open(true);
        }
    });
    handle.spawn(async move {
        match runtime.core().get_track(track_id).await {
            Ok(track) => {
                let data = map_track_info(track);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    apply_track_info(&w, data);
                    w.global::<TrackInfoState>().set_loading(false);
                });
            }
            Err(e) => {
                log::error!("[qbz-slint] track-info load failed: {e}");
                let msg = e.to_string();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let st = w.global::<TrackInfoState>();
                    st.set_error(msg.into());
                    st.set_loading(false);
                });
            }
        }
    });
}

/// Fetch a track and open the Track Info modal.
pub fn open_track_info<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    track_id: u64,
) where
    A: FrontendAdapter + Send + Sync + 'static,
{
    spawn_track_info(runtime, weak, handle, track_id, true);
}

/// Fetch a track and populate TrackInfoState WITHOUT opening the modal — for
/// the immersive split Track Info panel (data-panels.md §7), which renders the
/// metadata + grouped credits + copyright inline. Same fetch+map+apply as
/// `open_track_info`; `open` stays false so the floating modal never appears
/// over the immersive overlay.
pub fn load_track_info_inline<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    track_id: u64,
) where
    A: FrontendAdapter + Send + Sync + 'static,
{
    spawn_track_info(runtime, weak, handle, track_id, false);
}

/// Fetch an album and open the Album Info (Credits/Review) modal.
pub fn open_album_credits<A>(
    runtime: Arc<AppRuntime<A>>,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    album_id: String,
) where
    A: FrontendAdapter + Send + Sync + 'static,
{
    let _ = weak.upgrade_in_event_loop(|w| {
        let st = w.global::<AlbumInfoState>();
        st.set_error("".into());
        st.set_active_tab("credits".into());
        st.set_loading(true);
        st.set_open(true);
    });
    handle.spawn(async move {
        match runtime.core().get_album(&album_id).await {
            Ok(album) => {
                let data = map_album_credits(album);
                let _ = weak.upgrade_in_event_loop(move |w| {
                    apply_album_credits(&w, data);
                    w.global::<AlbumInfoState>().set_loading(false);
                });
            }
            Err(e) => {
                log::error!("[qbz-slint] album-info load failed: {e}");
                let msg = e.to_string();
                let _ = weak.upgrade_in_event_loop(move |w| {
                    let st = w.global::<AlbumInfoState>();
                    st.set_error(msg.into());
                    st.set_loading(false);
                });
            }
        }
    });
}
