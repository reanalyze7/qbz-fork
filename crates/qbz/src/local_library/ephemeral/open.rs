//! Open the native folder picker (or rehydrate a persisted path) and scan +
//! show the ephemeral pane.

use std::sync::Arc;

use qbz_app::shell::AppRuntime;
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::adapter::SlintAdapter;
use crate::{AppWindow, EphemeralAlbum, LocalLibraryState};

use super::build::{apply_ephemeral, folder_display_name};
use super::reset::reset_ephemeral_state;

pub(crate) type EphRuntime = Arc<AppRuntime<SlintAdapter>>;

/// Open the native folder picker, then scan + show the ephemeral pane.
pub fn open_ephemeral(
    runtime: EphRuntime,
    weak: slint::Weak<AppWindow>,
    handle: tokio::runtime::Handle,
) {
    handle.spawn(async move {
        let Some(dir) = rfd::AsyncFileDialog::new()
            .set_title(&qbz_i18n::t("Choose a folder to play"))
            .pick_folder()
            .await
        else {
            return;
        };
        let path = dir.path().to_string_lossy().to_string();
        scan_ephemeral(Some(runtime), weak, path, true).await;
    });
}

/// Re-open a previously-persisted ephemeral path on startup (no picker). Skips
/// silently if the path is gone, clearing the stale pref.
pub fn rehydrate_ephemeral(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let Some(path) = crate::locallibrary_prefs::ephemeral_path() else {
        return;
    };
    handle.spawn(async move {
        scan_ephemeral(None, weak, path, false).await;
    });
}

/// Shared scan path used by both the picker and rehydrate. When a `runtime` is
/// given (explicit open), any ephemeral track currently playing is wiped first
/// so it can't bleed into the freshly-loaded session (its synthetic id would be
/// reused). Rehydrate passes `None` (startup — nothing is playing).
async fn scan_ephemeral(
    runtime: Option<EphRuntime>,
    weak: slint::Weak<AppWindow>,
    path: String,
    from_picker: bool,
) {
    if let Some(rt) = &runtime {
        crate::playback::wipe_ephemeral_if_playing(rt, &weak).await;
    }
    let name = folder_display_name(&path);
    {
        let nm = name.clone();
        let p = path.clone();
        let _ = weak.upgrade_in_event_loop(move |w| {
            let s = w.global::<LocalLibraryState>();
            s.set_ephemeral_active(true);
            s.set_ephemeral_loading(true);
            s.set_ephemeral_name(nm.into());
            s.set_ephemeral_path(p.into());
            s.set_ephemeral_albums(ModelRc::new(VecModel::from(Vec::<EphemeralAlbum>::new())));
            if from_picker {
                s.set_active_tab("folders".into());
            }
        });
    }
    let scan_path = path.clone();
    let result = tokio::task::spawn_blocking(move || {
        crate::ephemeral::open(std::path::Path::new(&scan_path))
    })
    .await;

    match result {
        Ok(Ok(res)) => {
            let tracks = res.tracks;
            let skipped = res.skipped_files;
            let nm = name.clone();
            let p = path.clone();
            let _ = weak.upgrade_in_event_loop(move |w| {
                apply_ephemeral(&w, &nm, &p, &tracks, from_picker);
            });
            crate::locallibrary_prefs::save_ephemeral_path(Some(&path));
            if from_picker {
                if skipped > 0 {
                    crate::toast::success_weak(
                        &weak,
                        qbz_i18n::t_args(
                            "Opened folder ({} files skipped)",
                            &[&skipped.to_string()],
                        ),
                    );
                } else {
                    crate::toast::success_weak(&weak, qbz_i18n::t("Folder opened"));
                }
            }
        }
        Ok(Err(e)) => {
            log::warn!("[qbz-slint] ephemeral open failed: {e}");
            let _ = weak.upgrade_in_event_loop(|w| {
                reset_ephemeral_state(&w);
            });
            crate::ephemeral::clear();
            crate::locallibrary_prefs::save_ephemeral_path(None);
            if from_picker {
                crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't open that folder"));
            }
        }
        Err(_) => {
            let _ = weak.upgrade_in_event_loop(|w| {
                reset_ephemeral_state(&w);
            });
        }
    }
}
