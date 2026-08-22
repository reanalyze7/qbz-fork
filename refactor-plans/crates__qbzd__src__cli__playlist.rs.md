# crates/qbzd/src/cli/playlist.rs (213 lines)

## Summary
Implements the `qbzd playlist list|show|create|edit|rm|add|remove` CLI
verbs (thin HTTP wrappers over the daemon's `/api/playlist*` endpoints),
plus small local helpers (`post`, `resolve_ids`, `render_list`) and a short
`#[cfg(test)]` module for `render_list`.

## Proposed split
Split by "read verbs" vs "write verbs" vs "shared internals", as a flat
sibling-module split (no directory needed — small file):

- `cli/playlist.rs` (~35 lines) — becomes the re-export surface: module doc
  + `mod playlist_read; mod playlist_write; mod playlist_internal;` (or
  similar) with `pub use playlist_read::{list, show}; pub use
  playlist_write::{create, edit, rm, add, remove};` so
  `crate::cli::playlist::list` etc. keep their exact paths. Alternatively,
  since this is a leaf CLI-verb file with no sub-callers outside the CLI
  dispatcher, it may be simplest to just declare the split modules directly
  and re-export at the top — either convention is fine per the project's
  existing pattern (check a sibling `cli/*.rs` file for precedent before
  deciding).
- `cli/playlist_read.rs` (~55 lines) — `list`, `show` (the two GET-based
  read verbs) + `render_list` (the list-view renderer they share).
- `cli/playlist_write.rs` (~95 lines) — `create`, `edit`, `rm`, `add`,
  `remove` (the five POST-based mutating verbs).
- `cli/playlist_internal.rs` (~45 lines) — `post` (the shared POST-and-render
  helper), `resolve_ids` (the stdin-or-args track-id parser used by `add`/
  `remove`).
- `cli/playlist_tests.rs` or inline `#[cfg(test)] mod tests` kept in
  whichever file owns `render_list` (`playlist_read.rs`) — only 2 small
  tests, no need for a separate file; keep them attached to `render_list`.

## Re-export surface
Whichever file becomes `cli/playlist.rs` (or the module is renamed to a
`cli/playlist/mod.rs` directory) is the target of the existing `mod
playlist;` in `crates/qbzd/src/cli/mod.rs`. The seven public async fns
(`list`, `show`, `create`, `edit`, `rm`, `add`, `remove`) — each already
documented as one `qbzd playlist <verb>` — must all remain reachable as
`crate::cli::playlist::<verb>` from the CLI arg-dispatch match in
`cli/mod.rs` (or wherever the `Playlist` subcommand enum is matched).

## Coupling / watch out
- `post`, `resolve_ids` are both `pub(crate)`-or-private helpers called
  from multiple verb functions across the read/write split — make sure
  they're visible (e.g. `pub(super)` or re-exported) to both
  `playlist_read.rs` and `playlist_write.rs` if `list`/`show` end up not
  needing `post` (they call `client.get` directly) but `add`/`remove`
  do need `resolve_ids` — check exactly which functions call which helper
  before finalizing file boundaries (from the read: `post` is used by
  `create`/`edit`/`rm`/`add`/`remove`, i.e. everything in `playlist_write.rs`;
  `resolve_ids` is used by `add`/`remove` only — both fit cleanly in
  `playlist_internal.rs` with no read-side usage).
- `crate::cli::browse::{collect_ids, render}` and `crate::cli::client::
  ApiClient` are imported once at the top today — both are used only by
  `playlist_read.rs` (`show`'s `--ids`/plain-render paths); `ApiClient` is
  also used by `playlist_write.rs`'s `post` helper — re-import in each file.
- Exit-code conventions (`e.exit_code()`, the `2` for usage errors like
  `--public --private` conflict or missing `--yes`) are used identically
  across verbs — no shared constant to worry about, just keep the literal
  `2` and `e.exit_code()` calls as-is.

## Verify after split
- `cargo build -p qbzd` and `cargo test -p qbzd cli::playlist::` (the two
  existing `render_list` tests must stay green).
- Manually smoke-test a few `qbzd playlist` verbs against a running daemon
  (or check existing integration/e2e test coverage for this CLI surface)
  since most of these functions are thin HTTP wrappers with no unit tests
  of their own beyond `render_list`.
