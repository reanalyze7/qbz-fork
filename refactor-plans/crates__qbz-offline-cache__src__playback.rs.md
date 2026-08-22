# crates/qbz-offline-cache/src/playback.rs (166 lines)

## Summary
Pure (no Tauri/no UI-framework) resolution of a v2 CMAF-cached
`CmafBundleRow` into ready-to-play FLAC bytes: reads the bundle from disk,
unwraps its content key via the secret vault, and decrypts it — plus a thin
UI-events wrapper that emits `UnlockStart`/`UnlockEnd` around the blocking
work so the frontend can show an "unlocking" animation.

## Proposed split
By domain — UI-events wrapper (async, event sink) vs. the actual
pure/blocking resolution logic:

- `playback/mod.rs` (~15 lines) — module doc, `pub use` re-exports of
  `load_cmaf_bundle_with_ui_events` and `load_cmaf_bundle`.
- `playback/ui_events.rs` (~30 lines) — `load_cmaf_bundle_with_ui_events`:
  the async wrapper that emits `UnlockStart`/`UnlockEnd` around a
  `spawn_blocking` call into the resolver.
- `playback/resolve.rs` (~125 lines) — `load_cmaf_bundle`: validates
  `cache_format == 2`, reads init_path/content_key_wrapped, builds the
  `BundleLayout`, calls `cmaf_store::read_bundle`, unwraps the key via
  `secret_vault::get_or_init(...).unwrap(...)`, validates key length, and
  calls `decrypt_to_flac`. This is the pure/blocking resolution the doc
  comment explicitly calls out as "PURE resolution (no Tauri, no events)".

## Re-export surface
`playback/mod.rs` re-exports both functions so
`crate::playback::{load_cmaf_bundle_with_ui_events, load_cmaf_bundle}`
(used by whatever wires up offline playback — likely `downloader.rs`'s
sibling playback-trigger code or a `qbzd`/`qbz` caller) stays unchanged.

## Coupling / watch out
- `load_cmaf_bundle_with_ui_events` calls `load_cmaf_bundle` inside
  `tokio::task::spawn_blocking` — needs `use super::resolve::load_cmaf_bundle;`
  after the split; keep the `move` closure capturing `cmaf_track_id`, `row`,
  and `cache_path` (converted to owned `String`) exactly as-is, since this
  is the one place a `Path::new(&cache_path)` reconstruction happens inside
  the blocking closure.
- Two DIFFERENT track ids are threaded through: `display_track_id` (what
  the frontend/UI keys events on — could be a Qobuz id OR a Local Library
  row id) vs. `cmaf_track_id` (always the Qobuz id, used for on-disk
  bundle lookup and log lines) — this distinction is explained in the
  function's doc comment and must not be collapsed or confused when
  splitting; keep the doc comment intact in `ui_events.rs`.
- `load_cmaf_bundle` returns `Option<Vec<u8>>`, deliberately swallowing all
  error detail into `log::warn!` — every failure path (missing init_path,
  missing content_key_wrapped, bad bundle read, vault init failure, unwrap
  failure, wrong key size, decrypt failure) has its own distinct log
  message; preserve each one's wording since these are the primary
  observability signal for offline-playback bugs.
- Depends on `crate::cmaf_store::{self, BundleLayout}`, `crate::db::CmafBundleRow`,
  `crate::event::{CacheEvent, CacheEventSink}`, `crate::secret_vault` — all
  four imports need to land in the right file (`cmaf_store`/`db` types in
  `resolve.rs`; `event` types in `ui_events.rs`, though `resolve.rs` does
  NOT need `event` since it has no sink parameter).
- No `#[cfg(test)]` block exists in this file — this is exactly the code
  path this crate's `secret_vault`/CMAF-decrypt machinery is most fragile
  around; flag to whoever splits it that adding a unit test around the
  key-length validation (`unwrapped.len() != 16`) would be cheap and
  valuable.

## Verify after split
- `cargo check -p qbz-offline-cache` / `cargo build -p qbz-offline-cache`.
- Grep `crate::playback::` and `playback::load_cmaf_bundle` across the
  workspace to confirm both the events-wrapper and pure-resolver call
  sites still resolve.
- Manually play back a v2-cached (CMAF) offline track and confirm the
  "unlocking" UI animation still fires exactly once per playback attempt
  and audio starts correctly — no automated coverage exists to lean on.
