# crates/qbz-offline-cache/src/cmaf_store.rs (238 lines)

## Summary
On-disk layout and pure I/O for v2 CMAF-bundle offline cache entries:
persists a freshly-downloaded `CmafRawBundle` as `init.mp4` +
`segments.bin` (concatenated) + `manifest.json` under
`<offline_root>/tracks-cmaf/<track_id>/`, reads it back for playback, and
decrypts it into a playable FLAC byte stream. No network, no crypto beyond
the final decrypt step (delegated to `qbz_qobuz::cmaf`).

## Proposed split
By domain — on-disk layout/types vs. write path vs. read+decrypt path:

- `cmaf_store/mod.rs` (~15 lines) — module doc, `pub use` re-exports of
  `BundleManifest`, `BundleLayout`, `LoadedBundle`, `persist_bundle`,
  `read_bundle`, `remove_bundle`.
- `cmaf_store/layout.rs` (~55 lines) — the `SUBDIR`/`INIT_FILENAME`/
  `SEGMENTS_FILENAME`/`MANIFEST_FILENAME` consts, `BundleManifest` struct,
  `BundleLayout` struct + its `new()` — pure path/naming logic, no I/O.
- `cmaf_store/write.rs` (~65 lines) — `persist_bundle()` (creates the dir,
  writes init/segments/manifest, computes offsets, logs the summary) and
  the shared `write_atomic()` helper (used by both write and could be
  reused if read-side ever needs it — keep it here since only writes use
  atomic-rename-into-place today).
- `cmaf_store/read.rs` (~75 lines) — `read_bundle()`, `remove_bundle()`,
  the `LoadedBundle` struct, and its `decrypt_to_flac()` method — the
  read-back-and-decrypt path used at playback time.

## Re-export surface
`cmaf_store/mod.rs` re-exports everything currently public:
`pub use layout::{BundleManifest, BundleLayout}; pub use
write::persist_bundle; pub use read::{LoadedBundle, read_bundle,
remove_bundle};`. Callers today reach these via `crate::cmaf_store::*`
(e.g. `db.rs`'s `CmafBundleRow`-adjacent code, `downloader.rs`'s
`crate::cmaf_store::persist_bundle`, `playback.rs`'s
`crate::cmaf_store::{self, BundleLayout}`, `maintenance.rs`'s
`crate::cmaf_store::BundleLayout`) — all these paths are unaffected by an
internal split since they all go through `crate::cmaf_store::X`, not
`crate::cmaf_store::write::X`.

## Coupling / watch out
- `write_atomic()` is a private helper used only by `persist_bundle` for
  the init.mp4 and manifest.json writes (segments.bin uses its own
  create+write+rename sequence inline, NOT `write_atomic`, since it needs
  the offsets tracked during the write) — keep `write_atomic` private to
  `write.rs`, don't over-generalize it into a shared utility unless
  `read.rs` needs it (it doesn't).
- `BundleLayout::new()` is called from THREE other files in this crate
  (`downloader.rs`, `playback.rs`, `maintenance.rs`) as well as internally
  by `persist_bundle`/`read_bundle` — it's the most load-bearing single
  function in this module; keep its signature (`offline_root: &Path,
  track_id: u64`) untouched.
- `LoadedBundle::decrypt_to_flac` depends on `qbz_cmaf::parse_init_segment`
  and `qbz_qobuz::cmaf::decrypt_segments_into` — both external-crate calls,
  unaffected by the internal split but confirm `read.rs` carries the two
  `use` statements (`qbz_cmaf`, `qbz_qobuz::cmaf`).
- `BundleManifest`'s `segment_offsets` invariant (`length == n_segments +
  1`, last offset == total bytes) is checked in `read_bundle` with an
  explicit error message on mismatch — this validation is exactly the
  belt-and-suspenders behavior the module doc describes; don't drop it.
- No `#[cfg(test)]` block currently exists in this file — no tests to
  preserve, but this is fs+crypto code worth flagging as under-tested to
  whoever does the actual split.

## Verify after split
- `cargo check -p qbz-offline-cache` and `cargo build -p qbz-offline-cache`.
- Grep for `cmaf_store::` across the crate (and workspace, in case another
  crate reaches in) to confirm every caller's import path still resolves.
- Manually verify a full download → persist → read-back → decrypt round
  trip for one track still produces byte-identical FLAC output (compare
  against the legacy plain-FLAC path's output for the same track if
  available) since there is no automated test coverage here.
