# crates/qbz-playlist-import/src/importer.rs (137 lines)

## Summary
Top-level playlist-import orchestrator: `preview_public_playlist` (fetch +
detect only, no matching/creation) and `import_public_playlist` (the full
flow — fetch, match tracks against Qobuz, create the destination
playlist(s), chunked track-add with progress events, and part-splitting for
playlists over the 2000-track Qobuz limit).

## Proposed split
This file is only 7 lines over budget, and it's a single cohesive
orchestration flow (not obviously separable into pure/IO/render) — the
cleanest split is to carve out the "split into ≤2000-track parts and create
each" inner block as its own helper, since it's the most self-contained
chunk of the big function:

- `importer.rs` (~55 lines) — module doc, `ADD_CHUNK_SIZE`,
  `QOBUZ_PLAYLIST_TRACK_LIMIT` consts, `preview_public_playlist` (unchanged,
  lines 16-19), and a slimmed `import_public_playlist` that does the
  fetch + match-tracks + dedup-ids block (lines 28-46) then delegates to
  the new helper below and assembles the final `ImportSummary`.
- `create_parts.rs` (~85 lines) — a new `pub(crate) async fn
  create_playlist_parts(client: &QobuzClient, matched_track_ids: &[u64],
  base_name: &str, description: Option<String>, is_public: bool, progress:
  Arc<dyn ImportProgressSink>) -> Result<Vec<u64>, PlaylistImportError>`
  extracted from lines 48-116 (the "if !matched_track_ids.is_empty()"
  block: part-splitting, per-part playlist creation, chunked
  `add_tracks_to_playlist` calls with progress events). Returns the
  `qobuz_playlist_ids` Vec that `import_public_playlist` currently builds
  inline.

If the reviewer prefers not to introduce a new file for a 137-line file
that's barely over budget, an equally valid alternative is: leave the file
as one module and simply trim ~10 lines of blank/comment padding plus
tighten the `part_desc`/`playlist_name` per-part formatting into one
one-line-per-branch style — but since the project rule is a hard 130-line
ceiling, the `create_parts.rs` extraction above is the safer choice to
guarantee compliance without shrinking readability.

## Re-export surface
`importer.rs` stays the single public entry point — `preview_public_playlist`
and `import_public_playlist` remain defined there (or re-exported from
there if moved) so callers keep using
`qbz_playlist_import::importer::import_public_playlist(...)` (or the
flattened `qbz_playlist_import::import_public_playlist` if the crate root
re-exports it) unchanged. `create_playlist_parts` is `pub(crate)` only —
it has no external callers, so it does not need to be re-exported from the
crate root.

## Coupling / watch out
- `import_public_playlist` currently builds `matched_track_ids` (deduped,
  order-preserved) BEFORE the part-splitting block — `create_playlist_parts`
  needs that `Vec<u64>` passed in by reference/value; keep the dedup logic
  (lines 34-42) in `import_public_playlist` itself since it also computes
  `matched_count`/`total_tracks`/`skipped_tracks` used later in the
  `ImportSummary` construction (lines 118-135), which stays in
  `import_public_playlist`.
- `progress: Arc<dyn ImportProgressSink>` is cloned/passed into the
  extracted helper — it's already `Arc`-wrapped so this is a cheap clone,
  no new coupling.
- The final `ImportSummary`'s `playlist_name` field has a DELIBERATE
  behavior fix vs the original Tauri code, called out in a comment (lines
  122-124: reports the actually-created name, i.e. respects
  `name_override`, rather than the original source playlist name) — this
  comment and behavior must be preserved exactly; don't let it get lost or
  reordered during extraction since it documents an intentional
  divergence from the ported-from behavior.
- `client.create_playlist` and `client.add_tracks_to_playlist` (both
  `QobuzClient` methods) are the only external I/O calls in the extracted
  block — no additional dependency beyond what `importer.rs` already
  imports (`qbz_qobuz::QobuzClient`).

## Verify after split
- `cargo check -p qbz-playlist-import` and `cargo test -p qbz-playlist-import`
  (no `#[cfg(test)]` module exists in this specific file, but the crate
  likely has integration tests against `match_qobuz`/`providers` that
  exercise `import_public_playlist` end-to-end — confirm those still pass).
- Smoke-test an actual import through the running app: import a public
  Spotify/Deezer playlist with fewer than 2000 tracks (single-part path)
  AND, if feasible, verify the multi-part path's `"Part N of M"` naming
  logic is unchanged (may require a synthetic large playlist or a targeted
  unit test around `create_playlist_parts` if one didn't already exist).
