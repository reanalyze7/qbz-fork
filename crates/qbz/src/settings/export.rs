//! Settings > Developer > "Export settings…" — the `.qbzb` bundle export
//! flow. Self-contained, no coupling to the rest of the settings module.

use qbz_app::settings::bundle::{self, ExportOptions, ExportSource};

use crate::{AppWindow, SettingsExportState};

/// Export the desktop's settings to a user-chosen `.qbzb` bundle (Settings >
/// Developer > "Export settings…", 04 §4.2). The modal collects the single
/// `--include-auth` gate; this glue reads it, closes the modal, then off the
/// event loop: builds the bundle via the shared `qbz_app::settings::bundle`
/// engine (no new export logic), opens a native save dialog seeded with
/// `qbz-settings-YYYYMMDD.qbzb`, writes the file 0600 (the engine enforces the
/// mode), and toasts the import command. Wired to `SettingsExportActions
/// .confirm()` in `main.rs`. Any failure surfaces an error toast; a cancelled
/// dialog is a silent no-op.
pub fn export_settings(weak: slint::Weak<AppWindow>, handle: tokio::runtime::Handle) {
    // We are on the Slint event loop (invoked from the modal callback): read
    // the auth gate and close the modal here, before going async.
    let include_auth = weak
        .upgrade()
        .map(|w| {
            let st = w.global::<SettingsExportState>();
            let v = st.get_include_auth();
            st.set_open(false);
            v
        })
        .unwrap_or(false);

    handle.spawn(async move {
        // Build the bundle off the event loop (blocking SQLite reads).
        let built = tokio::task::spawn_blocking(move || {
            bundle::export(ExportSource::Desktop, &ExportOptions { include_auth })
        })
        .await;
        let doc = match built {
            Ok(Ok(b)) => b,
            Ok(Err(e)) => {
                crate::toast::error_weak(
                    &weak,
                    format!("{}: {e}", qbz_i18n::t("Could not export settings")),
                );
                return;
            }
            Err(e) => {
                crate::toast::error_weak(
                    &weak,
                    format!("{}: {e}", qbz_i18n::t("Could not export settings")),
                );
                return;
            }
        };

        // Native "save as" dialog seeded with the suggested filename; cancel =
        // silent no-op (matches booklet::download_booklet).
        let default_name = bundle::default_filename();
        let Some(dest) = rfd::AsyncFileDialog::new()
            .set_title(&qbz_i18n::t("Export settings"))
            .set_file_name(&default_name)
            .add_filter("QBZ settings bundle", &["qbzb"])
            .save_file()
            .await
        else {
            return;
        };
        let path = dest.path().to_path_buf();
        let file_name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| default_name.clone());

        // Write 0600 off the event loop.
        let write_path = path.clone();
        let written =
            tokio::task::spawn_blocking(move || bundle::write_bundle_file(&write_path, &doc)).await;
        match written {
            Ok(Ok(())) => {
                // Teach the import step (04 §4.2 toast copy). Only the lead-in is
                // localized; the command stays verbatim so it copies correctly.
                crate::toast::success_weak(
                    &weak,
                    format!(
                        "{} qbzd settings import {file_name}",
                        qbz_i18n::t("Bundle saved. On the daemon box:")
                    ),
                );
            }
            Ok(Err(e)) => crate::toast::error_weak(
                &weak,
                format!("{}: {e}", qbz_i18n::t("Could not export settings")),
            ),
            Err(e) => crate::toast::error_weak(
                &weak,
                format!("{}: {e}", qbz_i18n::t("Could not export settings")),
            ),
        }
    });
}
