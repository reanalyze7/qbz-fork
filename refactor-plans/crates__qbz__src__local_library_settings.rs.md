# crates/qbz/src/local_library_settings.rs (817 lines)

## 1. Summary
Settings > Local Library controller: folder CRUD (add/remove/edit/alias/
network-override) backed by a module-static `FOLDERS` list + a derived
filtered Slint model, folder-accessibility checks, maintenance (cleanup
missing files, two-step danger-zone clear), and — in a distinct back half —
the folder-scan engine (progress event sink + start/cancel).

## 2. Proposed module split
Two natural top-level domains (folders CRUD/maintenance vs. scan engine —
the file's own `// ==== Scan ====` banner at line 656 already marks the
seam), each further split by responsibility:

| New file | Owns | ~lines |
|---|---|---|
| `local_library_settings/mod.rs` | Module decls + re-exports; module doc comment | ~20 |
| `local_library_settings/state.rs` | `FolderData` struct, `FOLDERS` static, `FOLDERS_GEN`, `folders_lock`, `display_name`, `last_scan_label`, `fs_label_to_index`, `to_item`, `derive` (the pure model-derivation helpers + shared static) | ~110 |
| `local_library_settings/load.rs` | `load_folders`, `update_accessible`, `check_accessible` (reload + accessibility-check flow) | ~110 |
| `local_library_settings/crud.rs` | `add_folder`, `remove_folders`, `remove_folder`, `toggle_select`, `change_folder_path` | ~140 → split `add_folder`+`change_folder_path` (picker-driven) into `crud_picker.rs` (~80) and `remove_folders`+`remove_folder`+`toggle_select` into `crud_remove.rs` (~90) if still over |
| `local_library_settings/edit_modal.rs` | `edit_folder`, `save_folder_settings` | ~90 |
| `local_library_settings/maintenance.rs` | `cleanup_missing`, `clear_library`, `set_filter` | ~120 |
| `local_library_settings/scan.rs` | `SCAN_CANCEL` static, `basename`, `throttle_ok`, `run_scan` (the progress-event sink closure is the bulk of this) | ~110 → if `run_scan`'s sink closure alone pushes this over, extract `scan/sink.rs` for just the `ScanEvent` match arm | 
| `local_library_settings/scan_actions.rs` | `scan_all`, `scan_folder`, `stop_scan` | ~30 |

## 3. Re-export / public API surface
`local_library_settings/mod.rs` re-exports every current `pub fn` so
`crate::local_library_settings::X` callsites (from the Settings view wiring
in `main.rs`) are unaffected:

```rust
mod crud;
mod edit_modal;
mod load;
mod maintenance;
mod scan;
mod scan_actions;
mod state;

pub use crud::{add_folder, change_folder_path, remove_folder, remove_folders, toggle_select};
pub use edit_modal::{edit_folder, save_folder_settings};
pub use load::{check_accessible, load_folders};
pub use maintenance::{cleanup_missing, clear_library, set_filter};
pub use scan_actions::{scan_all, scan_folder, stop_scan};
```

(`state.rs`'s `derive`/`FOLDERS`/`folders_lock` stay `pub(super)` — internal
to the module, not part of the external surface.)

## 4. Tricky coupling / shared-state to watch out for
- `FOLDERS` (the `LazyLock<Mutex<Vec<FolderData>>>`) and `FOLDERS_GEN` are
  read/written from `load.rs`, `crud.rs`, `edit_modal.rs`, and `scan.rs` —
  they must live in `state.rs` and be `pub(super)`, not duplicated.
- `derive()` (rebuild the filtered Slint model from `FOLDERS`) is called
  after nearly every mutation (`add_folder`, `remove_folder(s)`,
  `toggle_select`, `save_folder_settings`, `set_filter`) — keep it in
  `state.rs` alongside `FOLDERS` since it's the one function that reads the
  static and writes the Slint state together.
- `load_folders` is called at the end of almost every mutating op (add/
  remove/edit/cleanup/clear/scan) to refresh from the DB — this creates a
  fan-in dependency from `crud.rs`/`edit_modal.rs`/`maintenance.rs`/
  `scan.rs` back into `load.rs`; make sure `load.rs` has no reverse
  dependency on any of them (it currently doesn't) to avoid a cycle.
- `run_scan`'s post-scan tail calls `crate::local_library::reset_browse_models`
  AND `load_folders` — both external-crate and internal-module deps, keep
  both when splitting into `scan.rs`.
- `SCAN_CANCEL` is a separate static from `FOLDERS_GEN`; `scan_folder`/
  `scan_all` (in `scan_actions.rs`) and `stop_scan` all touch it — keep it in
  `scan.rs` and have `scan_actions.rs` reference it via `super::scan::SCAN_CANCEL`
  or a small accessor, not a re-declared static.
- `remove_folders`/`remove_folder` both call `crate::recently::prune_albums`
  with album keys captured BEFORE the DB delete — this ordering (capture
  keys, then delete, then prune) is easy to break if the two functions are
  extracted independently; keep the sequencing intact.

## 5. What to verify after the real split
- `cargo build -p qbz`.
- Grep for `local_library_settings::` outside this file to confirm every
  external caller's import path still resolves.
- Smoke-test in the running app: Settings > Local Library — add a folder,
  edit its alias/network override, remove a folder, run "cleanup missing
  files", run a full scan and a single-folder scan (verify progress bar +
  cancel works), and the two-step "clear library" confirm dialogs.
