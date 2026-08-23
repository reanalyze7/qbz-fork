//! Folder removal (bulk + single) and selection toggling.

use slint::{ComponentHandle, Weak};

use crate::AppWindow;

use super::load::load_folders;
use super::state::{derive, display_name, folders_lock};

/// Bulk-remove the selected folders (with confirm + cascade track delete).
pub fn remove_folders(weak: Weak<AppWindow>, handle: tokio::runtime::Handle) {
    // Removes only the LocalLibrary DB entries + their indexed tracks — never
    // the files on disk. Reversible (re-add + re-scan reindexes), so we skip a
    // confirm dialog: rfd's message dialog needs `zenity`, which isn't present
    // on every Linux box, and silently fails-closed there (the original "delete
    // does nothing" bug). A Toast gives feedback instead.
    let paths: Vec<String> = folders_lock()
        .iter()
        .filter(|f| f.selected)
        .map(|f| f.path.clone())
        .collect();
    if paths.is_empty() {
        return;
    }
    let count = paths.len();
    let h = handle.clone();
    handle.spawn(async move {
        let paths2 = paths.clone();
        let keys = tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| {
                let mut keys: Vec<String> = Vec::new();
                for p in &paths2 {
                    keys.extend(db.album_keys_in_folder(p).unwrap_or_default());
                    db.remove_folder_with_tracks(p)?;
                }
                Ok(keys)
            })
        })
        .await
        .ok()
        .flatten()
        .unwrap_or_default();
        crate::recently::prune_albums(&keys);
        crate::toast::success_weak(
            &weak,
            qbz_i18n::tf("Removed {} folder", "Removed {} folders", count as i64, &[&count.to_string()]),
        );
        load_folders(weak, h);
    });
}

/// Remove ONE folder by id (confirm + cascade track delete). The per-row
/// delete button; independent of the multi-select state, so a previously
/// added folder can be removed without first selecting it + the toolbar trash.
pub fn remove_folder(weak: Weak<AppWindow>, handle: tokio::runtime::Handle, id: i64) {
    // DB-only removal (entry + indexed tracks), never the files. No confirm
    // dialog — see remove_folders for why (zenity-less boxes fail-closed).
    let (path, name) = {
        let g = folders_lock();
        match g.iter().find(|f| f.id == id) {
            Some(f) => (f.path.clone(), display_name(f)),
            None => return,
        }
    };
    let h = handle.clone();
    handle.spawn(async move {
        let p = path.clone();
        let result = tokio::task::spawn_blocking(move || {
            crate::library_db::with_db(|db| {
                // Capture album keys BEFORE the delete so we can prune them out
                // of Recently Played too (not just the DB rows).
                let keys = db.album_keys_in_folder(&p).unwrap_or_default();
                let n = db.remove_folder_with_tracks(&p)?;
                Ok((n, keys))
            })
        })
        .await
        .ok()
        .flatten();
        let (n, keys) = result.unwrap_or((0, Vec::new()));
        crate::recently::prune_albums(&keys);
        let tracks_label = qbz_i18n::tf("{} track", "{} tracks", n as i64, &[&n.to_string()]);
        crate::toast::success_weak(
            &weak,
            qbz_i18n::t_args("Removed \"{}\" ({})", &[&name, &tracks_label]),
        );
        load_folders(weak, h);
    });
}

/// Toggle one folder's selection (UI state in the static), then re-derive.
pub fn toggle_select(weak: Weak<AppWindow>, id: i64) {
    {
        let mut g = folders_lock();
        if let Some(f) = g.iter_mut().find(|f| f.id == id) {
            f.selected = !f.selected;
        }
    }
    let _ = weak.upgrade_in_event_loop(|w| derive(&w));
}
