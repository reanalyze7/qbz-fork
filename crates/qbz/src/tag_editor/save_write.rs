//! Direct-write confirm dialog + applying the save result back to the UI
//! thread. The blocking DB/lofty write itself lives in `save_index.rs`.

use std::sync::atomic::Ordering;

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, TagEditorState};

use super::refresh::refresh_after_save;
use super::save_index::write_and_index;
use super::save_payload::SavePayload;
use super::{ACK_KEY, SAVE_GEN};

/// Run the full async save: direct-write confirm (if needed), the blocking
/// write + DB index update, and applying the result to the UI.
pub(super) async fn run_save(
    weak: Weak<AppWindow>,
    handle: tokio::runtime::Handle,
    image_cache: crate::artwork::ImageCache,
    directory_path: String,
    payload: SavePayload,
) {
    let direct = payload.direct;
    if direct && !confirm_direct_write_once().await {
        return;
    }

    let gen = SAVE_GEN.fetch_add(1, Ordering::SeqCst) + 1;
    let _ = weak.upgrade_in_event_loop(|w| {
        w.global::<TagEditorState>().set_saving(true);
    });

    let result = write_and_index(weak.clone(), payload).await;

    let ok = result.is_ok();
    let err_msg = result.err().map(|e| e.to_string()).unwrap_or_default();
    let _ = weak.upgrade_in_event_loop(move |w| {
        if SAVE_GEN.load(Ordering::SeqCst) != gen {
            return;
        }
        let s = w.global::<TagEditorState>();
        s.set_saving(false);
        s.set_write_progress_current(0);
        s.set_write_progress_total(0);
        if ok {
            s.set_open(false);
        }
    });

    if ok {
        crate::toast::success_weak(&weak, qbz_i18n::t("Album metadata saved"));
        // Refresh the open album detail + reset browse models (D7).
        refresh_after_save(weak.clone(), handle, image_cache);
    } else {
        crate::toast::error_weak(&weak, qbz_i18n::t_args("Couldn't save metadata: {}", &[&err_msg]));
    }
    let _ = directory_path; // reserved (explicit directory plumbing, if added later)
}

/// One-time direct-write confirm dialog, gated on a persisted kv ack bit.
/// Returns whether the caller may proceed.
async fn confirm_direct_write_once() -> bool {
    let acked = tokio::task::spawn_blocking(|| crate::library_db::with_db(|db| db.get_kv(ACK_KEY)))
        .await
        .ok()
        .flatten()
        .flatten()
        .as_deref()
        == Some("1");
    if acked {
        return true;
    }
    let ok = rfd::AsyncMessageDialog::new()
        .set_title(&qbz_i18n::t("Write tags to audio files?"))
        .set_description(&qbz_i18n::t("This modifies your audio files on disk and cannot be undone."))
        .set_buttons(rfd::MessageButtons::YesNo)
        .show()
        .await
        == rfd::MessageDialogResult::Yes;
    if !ok {
        return false;
    }
    let _ = tokio::task::spawn_blocking(|| crate::library_db::with_db(|db| db.set_kv(ACK_KEY, "1"))).await;
    true
}
