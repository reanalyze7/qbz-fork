# crates/qbz-integrations/src/lastfm/client.rs (922 lines)

## Summary
`LastFmClient` — a Last.fm API client that routes every request through a
Cloudflare Workers proxy (no local API key/secret): auth (token/session),
scrobbling, now-playing, and a large set of read endpoints (similar
artists/tracks, top artists/tracks/albums, loved tracks, recent tracks) used
as taste-seed data for recommendations.

## Proposed split
By domain (auth vs write vs read-endpoints), plus the small shared JSON
helpers pulled out of the bottom of the file.

- `lastfm/client/mod.rs` (~60 lines) — `LastFmClient` struct definition,
  `Default`/`new`/`with_session_key`/`set_session_key`/`session_key`/
  `is_authenticated`/`clear_session`, `LASTFM_PROXY_URL` const, and
  `mod` declarations + re-exports.
- `lastfm/client/auth.rs` (~100 lines) — `get_token`, `get_session` (the two
  auth-flow methods, as an `impl LastFmClient` block).
- `lastfm/client/scrobble.rs` (~90 lines) — `scrobble`, `update_now_playing`
  (the two write/session-key-required endpoints).
- `lastfm/client/similarity.rs` (~180 lines) — `get_similar_artists`,
  `get_similar_tracks` (both use the `match_score` string-or-f64 parsing
  pattern).
- `lastfm/client/user_history.rs` (~280 lines) — `get_top_artists`,
  `get_top_tracks`, `get_loved_tracks`, `get_recent_tracks` (the four
  `user.*` endpoints keyed by Last.fm username).
- `lastfm/client/albums.rs` (~170 lines) — `get_artist_top_albums`,
  `get_user_top_albums` (both return `Vec<LastFmAlbum>`).
- `lastfm/client/json_helpers.rs` (~50 lines) — the free functions at the
  bottom: `extract_image`, `extract_mbid`, `extract_uts`, `parse_u64`. Shared
  by every read-endpoint file above.

## Re-export surface
`lastfm/client/mod.rs` is the public-API surface: it re-exports
`pub struct LastFmClient` (defined there) with its methods implemented via
separate `impl LastFmClient` blocks in each domain file (Rust allows this
within the same module tree as long as they're all `impl LastFmClient` for
the same type — put them in files that are `mod`-included as siblings, not
nested submodules, so `impl LastFmClient` in `auth.rs` sees the same type).
Callers currently do `use crate::lastfm::client::LastFmClient` (or via
`qbz_integrations::lastfm::LastFmClient` if re-exported higher up) — that
path is unaffected since `client.rs` → `client/mod.rs` is a transparent
rename in Rust's module system.

## Coupling / watch out
- Every read-endpoint method repeats the same "check status → parse JSON →
  check `error` field → extract array → filter_map" shape. Do NOT
  deduplicate this into a generic helper as part of the split — that's a
  separate refactor with its own risk; just move the methods verbatim into
  their new homes.
- `json_helpers.rs` functions (`extract_image`, `extract_mbid`, `extract_uts`,
  `parse_u64`) are free functions, not methods — they must be `pub(super)` or
  `pub(crate)` (not private) so `similarity.rs`, `user_history.rs`, and
  `albums.rs` can call them across file boundaries within the module.
- Check whether `lastfm/mod.rs` (parent module, not read here) does
  `pub use client::LastFmClient;` — if so it's unaffected by turning
  `client.rs` into `client/mod.rs`.
- No shared mutable state beyond `self.client` (reqwest::Client, cheap to
  clone/share) and `self.session_key` — splitting by method is safe, no
  interleaved state machine to worry about.

## Verify after split
- `cargo build -p qbz-integrations`
- `cargo test -p qbz-integrations lastfm` (if any exist; this file currently
  has no `#[cfg(test)]` block — consider flagging that gap, though adding
  tests is out of scope for this plan-only pass)
- Grep for `LastFmClient::` call sites in `qbz-app`/`qbz` (scrobbling,
  recommendations pipeline) to confirm the public method set is unchanged.
