//! The two-step danger-zone clear, and the filter re-derive.

use slint::{ComponentHandle, Weak};

use crate::{AppWindow, LibraryFoldersState};

use super::load::load_folders;
use super::state::derive;

/// Two-step danger-zone clear of all indexed tracks (audio files untouched).
pub fn clear_library(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    let h = handle.clone();
    handle.spawn(async move {
        let step1 = rfd::AsyncMessageDialog::new()
            .set_title(&qbz_i18n::t("Clear library database?"))
            .set_description(
                &qbz_i18n::t("This removes ALL indexed tracks from the database. Your audio files are NOT deleted. You will need to re-scan your folders afterward."),
            )
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            .await;
        if step1 != rfd::MessageDialogResult::Yes {
            return;
        }
        let step2 = rfd::AsyncMessageDialog::new()
            .set_title(&qbz_i18n::t("Are you absolutely sure?"))
            .set_description(&qbz_i18n::t("This action cannot be undone."))
            .set_buttons(rfd::MessageButtons::YesNo)
            .show()
            .await;
        if step2 != rfd::MessageDialogResult::Yes {
            return;
        }

        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<LibraryFoldersState>().set_clearing_library(true);
        });
        let ok = tokio::task::spawn_blocking(|| {
            crate::library_db::with_db(|db| db.clear_all_tracks()).is_some()
        })
        .await
        .unwrap_or(false);

        let _ = weak.upgrade_in_event_loop(|w| {
            w.global::<LibraryFoldersState>().set_clearing_library(false);
            // Reset the browse models so the tabs re-fetch on next visit.
            crate::local_library::reset_browse_models(&w);
        });
        if ok {
            crate::toast::success_weak(&weak, qbz_i18n::t("Library database cleared"));
        } else {
            crate::toast::error_weak(&weak, qbz_i18n::t("Couldn't clear the library database"));
        }
        load_folders(weak, h);
    });
}

/// Re-derive after the filter changed (the text is two-way bound already).
pub fn set_filter(weak: Weak<AppWindow>) {
    let _ = weak.upgrade_in_event_loop(|w| derive(&w));
}
