# crates/qbz-integrations/src/listenbrainz/client.rs (966 lines)

## Summary
`ListenBrainzClient`: direct (no-proxy) HTTP client for ListenBrainz —
config/token lifecycle (validate/set/restore/disconnect), scrobble submission
(now-playing + listen), and a set of read-only recommendation/history
endpoints (CF recommendations, recent listens, recording metadata hydration,
"created for you" playlists + their tracks, fresh releases).

## Proposed split
Split by "API concern" — auth/config vs. scrobble-submission vs. each
read-only discovery endpoint's fetch+parse pair, since each endpoint follows
the same status-check/parse-JSON boilerplate and is independently sized:

- `listenbrainz/client/mod.rs` (~70 lines) — module doc, `LISTENBRAINZ_API_URL`
  const, `ListenBrainzConfig` struct + `Default`, `ListenBrainzClient` struct
  definition + `Default` + `new`/`with_config`/`set_version`, `pub use` of the
  other files' `impl` blocks (Rust allows multiple `impl` blocks for one type
  across files in the same crate, so no re-export mechanics are actually
  needed beyond `mod` declarations).
- `listenbrainz/client/auth.rs` (~90 lines) — `is_enabled`, `set_enabled`,
  `is_authenticated`, `get_status`, `set_token`, `restore_token`, `get_token`,
  `get_user_name`, `disconnect`, `validate_token` (private) — the whole
  config/session lifecycle, as a second `impl ListenBrainzClient` block.
- `listenbrainz/client/submit.rs` (~110 lines) — `submit_playing_now`,
  `submit_listen`, `prepare_additional_info` (private), `submit_listens`
  (private) — the scrobble-submission path.
- `listenbrainz/client/recommendations.rs` (~95 lines) — `get_cf_recommendations`
  (CF recommendations) only; it's the most complex single parse (mbid list +
  score + timestamp) and deserves its own file.
- `listenbrainz/client/history.rs` (~90 lines) — `get_recent_listens`.
- `listenbrainz/client/metadata.rs` (~120 lines) — `get_metadata_recordings`
  (the recording_mbid-keyed object hydration — the biggest single parse in the
  file).
- `listenbrainz/client/playlists.rs` (~180 lines) — `get_created_for_playlists`
  + `get_playlist_tracks` (both playlist-shaped JSPF endpoints, share the
  `last_identifier_segment` helper, so keep them together) + the private
  `last_identifier_segment` helper and its nested `last_segment` closure-fn
  (lines 942-966) — this module already contains its own natural pure helper,
  worth noting as the one non-async, non-networked function in the whole file.
- `listenbrainz/client/releases.rs` (~100 lines) — `get_fresh_releases`.

## Re-export surface
`listenbrainz/client/mod.rs` re-declares `mod auth; mod submit; mod
recommendations; mod history; mod metadata; mod playlists; mod releases;`
(each just adding another `impl ListenBrainzClient { ... }` block, no `pub
use` of functions needed since they're all inherent methods). The
`ListenBrainzClient` type and `ListenBrainzConfig` stay importable at
`crate::listenbrainz::client::{ListenBrainzClient, ListenBrainzConfig}`
exactly as before (assuming today's `pub mod client;` in
`listenbrainz/mod.rs` stays a single file becoming a directory — check
whether `client.rs` needs to become `client/mod.rs` at the filesystem level,
which is mechanical and doesn't change the import path).

## Coupling / watch out
- `last_identifier_segment` (currently a free fn at file end) is used by BOTH
  `get_created_for_playlists` and `get_playlist_tracks` — keep it in
  `playlists.rs` as `pub(super)` or `pub(crate)` so both call sites in the
  same file can use it without needing a `use` from elsewhere; do NOT
  duplicate it.
- Every read-only method repeats the same "204/404 → empty, non-2xx → Err,
  empty/malformed body → empty" boilerplate — when splitting, consider (but
  do not implement, since this is plan-only) whether a private
  `fetch_optional_json(url, query) -> IntegrationResult<Option<Value>>`
  helper in `mod.rs` would reduce duplication across the 5 read-only files;
  flag this for the actual implementer as a nice-to-have, not required for
  the split itself.
- All methods share `self.client` (reqwest) and `self.config` (the
  `Arc<Mutex<ListenBrainzConfig>>>` token store) — both stay defined once on
  the struct in `mod.rs`; the split files only ever read `self.config.lock()`
  and `self.client`, no field relocation needed.
- `super::models::*` (the `use super::models::*;` at the top) must be
  re-imported in every new file that references `ListenBrainzStatus`,
  `UserInfo`, `SubmitListensPayload`, `Listen`, `TrackMetadata`,
  `ListenType`, `AdditionalInfo`, `CfRecommendation`, `LbListen`,
  `LbRecordingMeta`, `LbPlaylistMeta`, `LbPlaylistTrack`, `LbFreshRelease` —
  double check the models file's own path depth doesn't change (`super::` vs
  `super::super::` differs between `client.rs` at
  `listenbrainz/client.rs` and a new `client/*.rs` file, since the latter is
  one directory deeper — use `super::super::models::*` or `crate::listenbrainz::models::*` in the split files instead of `super::models::*`).

## Verify after split
- `cargo test -p qbz-integrations listenbrainz::` — no unit tests currently
  exist in this file (confirmed by inspection), but any workspace-level
  ListenBrainz integration test must still compile and pass.
- `cargo check -p qbz-integrations` and grep for
  `ListenBrainzClient::`/`use ... listenbrainz::client::` across the
  workspace (likely `qbz-app`'s scrobble controller / offline queue flush) to
  confirm the public path and method signatures are unchanged.
