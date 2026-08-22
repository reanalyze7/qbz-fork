# crates/qbz-music-link/src/odesli.rs (346 lines)

## Summary
Self-contained Odesli/song.link API client (ported from the Tauri `share`
module): wire-format response types, a simplified `SongLinkResponse`,
`ContentType`, and a `SongLinkClient` with an in-memory TTL cache that fetches
cross-platform share links by URL.

## Proposed split
By responsibility (error type / wire models / simplified model & content
type / client+cache), mirroring the file's own `// ── ──` section banners:

- `music_link/odesli/mod.rs` (~20 lines) — module doc, `pub use` re-exports
  of `ShareError`, `OdesliResponse`, `PlatformLink`, `Entity`,
  `SongLinkResponse`, `ContentType`, `SongLinkClient`; the `ODESLI_API_URL` /
  `REQUEST_TIMEOUT` / `CACHE_TTL` consts.
- `music_link/odesli/error.rs` (~15 lines) — lines 19-31: `ShareError` enum
  (thiserror-derived).
- `music_link/odesli/models.rs` (~120 lines) — lines 33-182:
  `OdesliResponse`, `PlatformLink`, `Entity` (the raw wire-format structs) +
  `deserialize_string_or_number` (the custom Visitor handling Bandcamp's
  numeric-vs-string entity IDs) — kept together since the Visitor exists
  solely to deserialize `Entity::id`.
- `music_link/odesli/simplified.rs` (~20 lines) — lines 109-149:
  `SongLinkResponse` (the consumer-facing simplified struct) + `ContentType`
  enum and its `as_str()` impl. (Small overlap in line range with models.rs
  above is fine — these are two independent structs adjacent in the source;
  split at the `SongLinkResponse` doc comment, line ~109.)
- `music_link/odesli/client.rs` (~150 lines) — lines 184-346: `CacheEntry`,
  `SongLinkClient` (struct + `Default` + `new`), `get_by_url`,
  `convert_response`, `get_from_cache`, `store_in_cache`, `clear_cache` — the
  whole networking+caching client. Comfortably under 130 if `mod.rs`'s consts
  move with it instead of to `mod.rs`; otherwise trim by moving
  `convert_response` into its own `client/convert.rs` (~40 lines) if the
  reviewer wants every file well under budget.

## Re-export surface
`music_link/odesli/mod.rs` stays the `mod odesli;` (or `pub mod odesli;`)
target — callers currently write `qbz_music_link::odesli::SongLinkClient`,
`odesli::ContentType`, `odesli::SongLinkResponse`, `odesli::ShareError`; all
four (plus `OdesliResponse`/`PlatformLink`/`Entity` if used outside this
module for testing/debugging) must be re-exported from `mod.rs` at their
current paths.

## Coupling / watch out
- `Entity::id`'s custom deserializer (`deserialize_string_or_number`) is
  specifically there because Bandcamp's Odesli entities return numeric IDs
  while other providers return strings — keep this function attached to
  `models.rs` (or wherever `Entity` lives) since it's meaningless without
  the struct it deserializes.
- `SongLinkClient::convert_response` reads BOTH `OdesliResponse` (from
  `models.rs`) and constructs `SongLinkResponse` (from `simplified.rs`) — it
  needs `use` imports from both new files; this is the one real
  cross-file coupling point in this split, but it's a single well-contained
  function, not spread state.
- The in-memory `Mutex<HashMap<String, CacheEntry>>` cache is process-lifetime
  only (comment doesn't say so explicitly but there's no persistence) — no
  external state to worry about when splitting, just keep `CacheEntry` in
  the same file as `SongLinkClient` since nothing else touches it.
- `#[allow(dead_code)]` annotations on `OdesliResponse` and `Entity` fields
  are intentional ("wire-shape fields kept for fidelity; not all are read
  here") — do not drop them or the fields when moving to `models.rs`, and do
  not let clippy dead-code-warnings tempt a slimming pass here.

## Verify after split
- `cargo check -p qbz-music-link` and `cargo build -p qbz-music-link`.
- `cargo test -p qbz-music-link` (no inline tests currently in this file, but
  confirm no test elsewhere in the crate references `odesli::` internals
  directly that would need path updates).
- Smoke-test the actual "Share" flow (whatever caller invokes
  `SongLinkClient::get_by_url`) against a real Spotify/Apple Music URL to
  confirm the client still resolves and caches correctly.
