# crates/qbz-integrations/src/musicbrainz/client.rs (809 lines)

## Summary
`MusicBrainzClient`: a rate-limited `reqwest` HTTP client wrapping the
MusicBrainz web service (direct or via a Cloudflare proxy), covering
recording/artist/release/area search + lookup, ISRC/tag/area resolution
helpers, and a Lucene query-escaping utility.

## Proposed split
- `musicbrainz/client/mod.rs` (~90 lines) — module doc, `RateLimiter` struct
  (`new`/`for_proxy`/`with_interval`/`wait`/`Default`), the two URL consts,
  `pub use` of `MusicBrainzClient`/`MusicBrainzConfig` from `core.rs`.
- `musicbrainz/client/core.rs` (~80 lines) — `MusicBrainzConfig` (+
  `Default`), `MusicBrainzClient` struct + `Default` + `new`/`with_config`,
  `is_enabled`/`set_enabled`/`base_url` — construction + config only.
- `musicbrainz/client/recordings.rs` (~90 lines) — `search_recording_by_isrc`,
  `search_recording`, `get_recording_isrcs`, `resolve_track`. Recording-shaped
  lookups/resolution.
- `musicbrainz/client/artists.rs` (~140 lines) — `search_artist`,
  `resolve_artist`, `get_artist_with_relations`, `get_artist_tags`,
  `search_artists_by_tag`, `search_artists_by_tag_and_area`.
- `musicbrainz/client/releases.rs` (~90 lines) — `search_release_by_barcode`,
  `search_release`, `search_releases_extended`, `get_release_with_tracks`.
- `musicbrainz/client/areas.rs` (~200 lines) — `browse_artists_by_area`,
  `search_area`, `get_area_with_relations`, `resolve_parent_subdivision`,
  `resolve_area_country` — the largest chunk since
  `resolve_parent_subdivision`/`resolve_area_country` are each ~60 lines of
  hop-walking logic; kept together since they share the "walk the area
  parent-of hierarchy" pattern and both call `get_area_with_relations`.
- `musicbrainz/client/http.rs` (~90 lines) — internal helpers:
  `check_enabled`, `check_response` (placeholder), `handle_response_status`,
  `parse_retry_after`, `escape_query` — the response/error-handling +
  Lucene-escaping utilities every other file's methods call.

## Re-export surface
`musicbrainz/client/mod.rs` re-exports `MusicBrainzClient`,
`MusicBrainzConfig`, `RateLimiter` at `crate::musicbrainz::client::*` (already
re-exported further up at `qbz_integrations::musicbrainz::*` per the crate's
existing `mod.rs`) — no caller-visible path change; `client.rs` becomes
`client/mod.rs`.

## Coupling / watch out
- Every method across every file is `impl MusicBrainzClient { ... }` on the
  same struct with 3 fields (`client`, `rate_limiter`, `config`) — split the
  impl block across files (`impl MusicBrainzClient` repeated per file), all
  methods already only touch `self.client`/`self.rate_limiter`/`self.config`
  via public-ish accessors (`base_url`, `is_enabled`) so no field visibility
  issue as long as `client`/`rate_limiter`/`config` stay `pub(super)` or the
  helper methods (`base_url`, `check_enabled`) stay callable — they're
  currently private `async fn`s on the same impl, so need `pub(super)` (or
  `pub(crate)`) once spread across files.
- `escape_query` (in `http.rs`) is called as `Self::escape_query(...)` from
  nearly every method in `artists.rs`/`releases.rs`/`areas.rs` — needs
  `pub(super)` visibility once it's not in the same file.
- `handle_response_status` and `check_enabled` are similarly called from
  almost every network method across files — same visibility bump needed.
- `resolve_area_country`/`resolve_parent_subdivision` are near-duplicate hop-
  walking loops (both cap at `max_hops = 5`, both call
  `get_area_with_relations`) — a future cleanup could unify them, but this
  plan keeps them as-is (behavior-preserving split only), just co-located in
  `areas.rs` since they're the most obviously related pair.
- No shared mutable state beyond the 3 struct fields; `Arc<Mutex<...>>`
  wrapping (`rate_limiter`, `config`) is already clone-friendly so no locking
  concerns from the file split itself.

## Verify after split
- `cargo test -p qbz-integrations musicbrainz::` (no unit tests currently
  live in this file itself, but check the crate-level test suite for
  MusicBrainz client coverage and run it green).
- `cargo check -p qbz-integrations` and grep for `MusicBrainzClient::` /
  `musicbrainz::client::` importers (qbz-app musician/artist resolution
  flows) to confirm the public path and method signatures are unchanged.
