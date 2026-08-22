# crates/qbz/src/tag_editor.rs (611 lines)

## Summary
Local album tag editor controller (Slint side): opens the editor pre-fetching
tracks off-thread, saves edits via sidecar or direct file-write (with a one-time
confirm dialog), and offers remote MusicBrainz/Discogs metadata lookup + apply.

## Proposed split
By domain, matching the file's own section marker (`// === Remote lookup ===`)
plus the natural open/populate vs save vs remote-lookup phases.

- `tag_editor/mod.rs` (~40 lines) — module doc, imports, `ACK_KEY` const,
  `SAVE_GEN` static, `parse_year`/`parse_num` shared parse helpers, and
  `pub use` re-exports from the split files so `crate::tag_editor::open_tag_editor`
  etc. keep working.
- `tag_editor/open.rs` (~90 lines) — `open_tag_editor`, `populate`,
  `close_tag_editor`. The "open the modal, seed state" phase.
- `tag_editor/save.rs` (~240 lines) — `save_tags` (the largest single function,
  ~230 lines: validation, payload building, direct-write confirm dialog, the
  blocking DB/lofty write, apply result). This alone may need an internal
  private-fn split (e.g. `validate_inputs`, `build_payloads`,
  `confirm_direct_write_once`, `write_and_index`) to stay readable even inside
  one ~240-line file — flag for the real split to consider function-level
  extraction, not just file-level.
- `tag_editor/refresh.rs` (~20 lines) — `refresh_after_save` (re-open the local
  album view + reset browse models after a successful save).
- `tag_editor/remote.rs` (~180 lines) — the whole "Remote lookup" section:
  `REMOTE_GEN` static, `map_search`, `search_remote`, `select_result`,
  `apply_remote`, `open_in_browser`.

## Re-export surface
`tag_editor/mod.rs` stays the public surface: `pub use open::{open_tag_editor,
close_tag_editor}`, `pub use save::save_tags`, `pub use
remote::{search_remote, select_result, apply_remote, open_in_browser}`. All
call sites (`crate::tag_editor::open_tag_editor(...)` etc., invoked from the
album detail view / Slint callback wiring in main.rs) need zero changes.

## Coupling / watch out
- `SAVE_GEN` (save.rs) and `REMOTE_GEN` (remote.rs) are independent
  generation counters — don't accidentally merge them into one shared static;
  they guard different async races (save vs remote fetch) and can legitimately
  overlap.
- `save_tags` calls `refresh_after_save` at the end on success — keep this
  cross-file call working (`refresh::refresh_after_save`); the two are
  logically one flow split only for line-count reasons.
- `populate` (open.rs) and `save_tags` (save.rs) both read/write the same
  `TagEditorState` global fields (album_title, tracks model shape, etc.) —
  no shared mutable Rust state between them (all via the Slint global), so no
  additional cross-file `Mutex`/static concern beyond `SAVE_GEN`.
- `parse_year`/`parse_num` are used by BOTH `save.rs` (validation there) — keep
  them in `mod.rs` (or a small `tag_editor/parse.rs`) and `use super::{parse_year,
  parse_num};` from `save.rs`.
- The remote-apply merge in `remote.rs::apply_remote` mutates the same
  `TagEditorState.tracks` `ModelRc` that `open.rs::populate` seeds and
  `save.rs::save_tags` reads — all three files touch this one model by
  position; no code change needed but worth a comment cross-referencing all
  three when splitting.

## Verify after split
- `cargo build -p qbz` (this is a qbz-slint controller module, called from the
  local album view's edit-pencil action).
- Smoke-test: `grep -rn "tag_editor::" crates/qbz/src` still resolves.
- Manual/UI smoke-test: open the tag editor on a local album, edit + save via
  sidecar mode, edit + save via direct-write mode (confirm dialog appears once
  then not again), search MusicBrainz and Discogs, apply a remote result,
  open a result in the browser.
