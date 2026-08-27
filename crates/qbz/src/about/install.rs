//! `install`: seed the static About fields + wire the open-url callback.

use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{AboutActions, AboutState, AppWindow};

use super::avatars::spawn_avatar_loads;
use super::meta::{
    app_version, build_commit, build_contributor_groups, build_date, platform_label,
    AUTHOR_HANDLE,
};

/// Seed the static fields, wire the open-url callback, and dispatch the avatar
/// fetches. Call once at shell setup. `handle` runs the avatar downloads off the
/// UI thread.
pub fn install(window: &AppWindow, handle: tokio::runtime::Handle) {
    let state = window.global::<AboutState>();

    let version = app_version();
    state.set_version(version.into());
    state.set_platform_label(platform_label().into());
    state.set_build_date(build_date().into());
    state.set_build_commit(build_commit().into());
    state.set_release_url(format!("https://github.com/reanalyze7/qoqobuz/releases/tag/v{version}").into());
    state.set_author_name(AUTHOR_HANDLE.into());
    state.set_author_url(format!("https://github.com/{AUTHOR_HANDLE}").into());

    state.set_contributor_rows(ModelRc::new(VecModel::from(build_contributor_groups())));

    window.global::<AboutActions>().on_open_url(|url| {
        let url = url.to_string();
        if url.is_empty() {
            return;
        }
        if let Err(e) = open::that(&url) {
            log::warn!("[qbz-slint] open About URL failed ({url}): {e}");
        }
    });

    spawn_avatar_loads(window.as_weak(), handle);
}
