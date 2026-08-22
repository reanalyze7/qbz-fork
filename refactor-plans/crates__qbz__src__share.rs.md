# crates/qbz/src/share.rs (184 lines)

## Summary
Share-link helpers for the track/album/artist/playlist/label context-menu
Share actions: canonical Qobuz URLs, a process-wide clipboard singleton, and
Song.link/Album.link (Odesli, ISRC/UPC-via-Deezer-first) resolution.

## Proposed split
By concern (URL builders / clipboard / external resolvers) — this file has
no `#[cfg(test)]` block, so the split is purely production code:

- `share/mod.rs` (~15 lines) — module doc, `mod` wiring (`mod urls; mod
  clipboard; mod songlink;`), re-exports.
- `share/urls.rs` (~30 lines) — `qobuz_track_url`, `qobuz_playlist_url`,
  `qobuz_album_url`, `qobuz_artist_url`, `qobuz_label_url` (pure string
  formatting, zero I/O, zero deps beyond `format!`).
- `share/clipboard.rs` (~35 lines) — the `CLIPBOARD` static +
  `copy_to_clipboard` (the arboard singleton, with its detailed doc comment
  about X11/Wayland selection-owner lifetime — keep that comment verbatim,
  it documents a real cross-platform bug fix, #514).
- `share/songlink.rs` (~110 lines) — `share_http_client`, `deezer_lookup`,
  `songlink_for_track`, `albumlink_for_album`, `songlink_url` (the Odesli/
  Deezer resolution pipeline; still comfortably under 130 lines as one file
  since these five functions form one cohesive async pipeline with real
  data flow between them).

## Re-export surface
`share/mod.rs` — becomes `crates/qbz/src/share/mod.rs`. Keeps
`pub fn qobuz_track_url`, `pub fn qobuz_playlist_url`, `pub fn
qobuz_album_url`, `pub fn qobuz_artist_url`, `pub fn qobuz_label_url`,
`pub fn copy_to_clipboard`, `pub async fn songlink_for_track`, `pub async fn
albumlink_for_album`, `pub async fn songlink_url` all re-exported (via `pub
use urls::*; pub use clipboard::copy_to_clipboard; pub use songlink::{
songlink_for_track, albumlink_for_album, songlink_url};`) so every
`crate::share::X` call site (the track/album/artist/playlist/label context
menus) is unaffected. `share_http_client` and `deezer_lookup` stay private —
used only within `songlink.rs`.

## Coupling / watch out
- `songlink_for_track` and `albumlink_for_album` both call `qobuz_track_url`/
  `qobuz_album_url` (from `urls.rs`) as their Odesli-fallback source URL —
  `songlink.rs` needs `use super::urls::{qobuz_track_url,
  qobuz_album_url};` after the split.
- `deezer_lookup` and `songlink_url` are both called by BOTH
  `songlink_for_track` and `albumlink_for_album` — keep all five functions in
  `songlink.rs` together rather than splitting further; they share the
  `share_http_client()` builder and the ISRC/UPC-first-then-Odesli-fallback
  pattern.
- `CLIPBOARD` is a `OnceLock` scoped to the whole process — genuinely
  independent of the URL/songlink logic, safe standalone module.
- No shared mutable state crosses the three proposed files (urls are pure,
  clipboard is self-contained, songlink is self-contained) — this is a
  low-risk, mechanical split.

## Verify after split
- No existing `#[cfg(test)]` in this file — check whether other files in
  `crates/qbz/src/` have integration/unit tests exercising `share::` (grep
  for `qobuz_track_url`, `songlink_for_track`, etc. in test contexts) and run
  those; otherwise verify by `cargo check -p qbz` alone plus a manual smoke
  test.
- `cargo check -p qbz` for any `crate::share::X` call site (context menu Share
  actions for track/album/artist/playlist/label).
- Smoke-test: trigger "Copy Qobuz link", "Copy Song.link", "Copy Album.link"
  from the running app's context menus and confirm clipboard contents and
  that the ISRC/UPC-first Deezer resolution still works (network-dependent —
  check logs for the "via ISRC"/"via UPC" info-level messages).
