# crates/qbz-qobuz/src/client.rs (2757 lines)

## Summary
The `QobuzClient` struct plus ~80 methods implementing the entire Qobuz REST API
surface: init/bundle bootstrap, login (password + OAuth + token restore), the
403-circuit-breaker, header/signing helpers, search, album/artist/track/playlist/
genre/discover/label reads, authenticated favorites/playlist mutations, artist-page
endpoints, and CMAF streaming session + file-URL resolution — all in one `impl`
block, plus unit tests at the bottom.

## Proposed split
Rust allows multiple `impl QobuzClient { ... }` blocks across files, so this is a
low-risk "by API domain" split — no behavior or field-location changes needed.
Convert `client.rs` into a `client/` directory:

- `client/mod.rs` (~140 lines) — `QobuzClient` struct, `Clone`, `Default`, `new()`,
  `CmafSession` struct, `USER_AGENT` const, `body_preview` helper, `init()`,
  locale get/set, `app_id()`, `get_http()`, `http()` (offline gate), and
  `mod` declarations + `pub use` re-exporting nothing extra (methods stay on the
  inherent impl via `include`, so external callers see no change). This file owns
  the struct definition; all other files add `impl QobuzClient` blocks.
- `client/forbidden_breaker_guard.rs` (~40 lines) — `forbidden_guard`,
  `note_forbidden_status` (the two 403-breaker methods; the `ForbiddenBreaker` type
  itself already lives in its own module).
- `client/auth.rs` (~230 lines) — `secret()`, `test_secret()`, `login()`,
  `is_logged_in()`, `logout()`, `set_session()`, `login_with_oauth_code()`,
  `login_with_token()`, `get_user_info()`, `auth_token()`.
- `client/headers.rs` (~110 lines) — `api_headers()`, `authenticated_headers()`,
  `signed_get()`, `signed_get_auth()` (the shared request-building helpers every
  endpoint group below calls).
- `client/search.rs` (~140 lines) — `search_albums`, `search_tracks`,
  `search_artists`, `catalog_search`, `get_similar_artists`, `get_artist_tracks`.
- `client/catalog_reads.rs` (~230 lines) — `get_album`, `get_featured_albums`,
  `get_release_watch` (incl. the artist-backfill workaround), `get_album_suggest`,
  `get_dynamic_suggest`, `get_dynamic_suggest_full`.
- `client/discover.rs` (~170 lines) — `get_genres`, `get_discover_index`,
  `get_discover_albums`, `get_discover_playlists`, `get_playlist_tags`.
- `client/tracks_artists.rs` (~250 lines) — `get_track`, `get_artist_basic`,
  `get_artist`, `get_artist_detail`, `get_artist_with_pagination`,
  `get_artist_with_pagination_and_locale`.
- `client/playlist_reads.rs` (~130 lines) — `get_playlist_track_ids`,
  `get_tracks_batch`, `get_tracks_batch_chunk`.
- `client/playlist_paginated.rs` (~130 lines) — `get_playlist` (the
  auto-paginating, concurrent-fetch playlist reader — kept alone since it is
  algorithmically dense).
- `client/labels.rs` (~220 lines) — `get_label_page`, `get_label_explore`,
  `get_label_albums`, `get_label_next_releases`, `get_label_awarded_releases`,
  `get_label_playlists`, `get_label_top_artists`, `get_label_story`,
  `get_label_list`.
- `client/authenticated.rs` (~250 lines) — `get_stream_url`,
  `get_stream_url_with_fallback`, `get_favorites`, `get_user_playlists`,
  `search_playlists`, `create_playlist`, `delete_playlist`,
  `add_tracks_to_playlist`, `remove_tracks_from_playlist`, `update_playlist`,
  `subscribe_playlist`, `unsubscribe_playlist`, `add_favorite`, `remove_favorite`.
- `client/artist_page.rs` (~90 lines) — `get_artist_page`, `get_releases_grid`,
  `get_artist_story`.
- `client/cmaf.rs` (~210 lines) — `ensure_cmaf_session`, `get_file_url` (the CMAF
  streaming endpoints — already conceptually separate per the file's own
  `// === CMAF streaming endpoints ===` marker).
- `client/tests.rs` (~90 lines) — the existing `#[cfg(test)] mod tests` block
  (offline-gate fast-fail + login-exemption tests), included from `mod.rs` as
  `#[cfg(test)] mod tests;`.

## Re-export surface
`client/mod.rs` keeps the public path `qbz_qobuz::client::QobuzClient` (or
whatever the crate root currently re-exports) unchanged — since all the split
files add `impl QobuzClient` blocks under the SAME `client` module (declared via
`mod auth; mod search; ...` inside `mod.rs`, all `pub(crate)` or private modules
whose contents merge onto the one struct), no caller anywhere in the codebase
needs to change an import. Every method keeps its exact `pub async fn` signature
and visibility.

## Coupling / watch out
- `signed_get` / `signed_get_auth` / `api_headers` / `authenticated_headers` are
  used by almost every other file — put them in `headers.rs` FIRST and confirm
  every other split file can call `self.signed_get(...)` normally (inherent
  methods are visible across impl blocks in the same crate automatically, so this
  is purely organizational, not a visibility problem).
- `secret()` (in `auth.rs`) is called by `signed_get`/`signed_get_auth`
  (`headers.rs`) and directly by a couple of search methods — no special
  re-export needed since it's `pub(crate)` on the same struct, but note the
  cross-file dependency when ordering the actual split PRs.
- The private `CmafSession` struct must stay reachable from `cmaf.rs` — keep its
  definition in `mod.rs` (already planned) since the `cmaf_session` field lives
  on the main struct.
- `locale()` (private) vs `get_locale()` (pub) — both stay in `mod.rs` since
  several other files (`tracks_artists.rs`, `discover.rs`, `labels.rs`) call the
  private `locale()` helper; keeping it on the base struct avoids visibility
  churn.
- Test module references `ApiError`, `crate::offline_gate` — keep those imports
  when moving to `client/tests.rs`.
- Very large fan-out of `qbz_models::*` glob import — keep the same glob import
  at the top of each split file rather than trying to import individual types,
  to avoid subtle missing-import compile errors during the mechanical split.

## Verify after split
- `cargo check -p qbz-qobuz` and `cargo build -p qbz-qobuz`.
- `cargo test -p qbz-qobuz client` — the two offline-gate tests must stay green.
- `cargo check` at the workspace root (or at minimum `cargo check -p qbz` and
  `-p qbz-app`) since `QobuzClient` is consumed widely — confirm no caller
  referenced `client::` submodule paths directly (they shouldn't, since methods
  are all inherent).
