# crates/qbz/src/remote_stream.rs (317 lines)

## Summary
Progressive HTTP streaming feeder for remote audio URLs (QConnect renderer
path): probes a URL for size/FLAC format via HEAD + a small ranged GET, opens
the player's progressive streaming sink, then downloads the body chunk-by-chunk
into the returned `BufferWriter` in a spawned task, plus reqwest error-message
helpers for diagnosing an Akamai header-flood failure mode.

## Proposed split
By responsibility — orchestration ↔ probing ↔ chunked download ↔ error
diagnostics:

- `remote_stream/mod.rs` (~55 lines) — module doc, `RemoteStreamInfo` struct,
  `stream_remote_track_into_player` (the public orchestration entry point that
  ties probe → sink-open → spawn download together), `mod` declarations,
  `pub use` re-exports.
- `remote_stream/probe.rs` (~80 lines) — `probe_remote_stream_info` (HEAD +
  ranged GET + FLAC STREAMINFO parse via `qbz_models::probe_streaminfo`).
- `remote_stream/download.rs` (~120 lines) — `download_and_stream_remote_track`,
  including the local `FailGuard` drop-guard struct (kept together — the guard
  only makes sense next to the loop it protects).
- `remote_stream/errors.rs` (~35 lines) — `describe_reqwest_error`,
  `is_header_flood_error` (self-contained diagnostic helpers with no
  dependency on the rest of the file).

## Re-export surface
`remote_stream/mod.rs` re-exports `RemoteStreamInfo`,
`stream_remote_track_into_player`, `probe_remote_stream_info`,
`download_and_stream_remote_track`, `describe_reqwest_error`,
`is_header_flood_error` — matching every current `pub` item — so the QConnect
renderer (`qconnect_engine.rs`, per the file's doc comment: "used by the QConnect
renderer... exactly one feeder") needs no import changes.

## Coupling / watch out
- `probe_remote_stream_info` (probe.rs) and `download_and_stream_remote_track`
  (download.rs) both call `describe_reqwest_error` (errors.rs) — trivial `use
  super::errors::describe_reqwest_error;` in each.
- `stream_remote_track_into_player` (mod.rs) calls both `probe_remote_stream_info`
  and spawns a task calling `download_and_stream_remote_track` — needs `use
  super::probe::probe_remote_stream_info;` and
  `use super::download::download_and_stream_remote_track;`.
- The "BIT-PERFECT" doc comment at the top of the file explains an important
  invariant (the decoded stream feeds the PROTECTED device init) — keep that
  comment on `mod.rs` since it describes the whole module's contract, not one
  function.
- `is_header_flood_error` looks unused within this file (no call site shown) —
  check its callers elsewhere in the crate before moving; it's likely called
  from playback-fallback logic in another file via
  `crate::remote_stream::is_header_flood_error`, so keep the re-export.

## Verify after split
- `cargo check -p qbz` / `cargo build`.
- No existing unit tests in this file; none to keep green. Consider whether
  `is_header_flood_error` deserves a couple of unit tests given it matches on
  literal substrings (optional, not required here).
- Smoke-test: play a remote/QConnect track end-to-end and confirm progressive
  streaming still starts before the full file downloads, and that a
  deliberately-broken URL still surfaces a fallback path (if one exists) via
  `is_header_flood_error`.
