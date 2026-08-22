# crates/qbz-offline-cache/src/downloader.rs (760 lines)

## Summary
Two layers: `StreamFetcher` (a retrying HTTP stream-to-file downloader with
per-attempt fresh clients to dodge HTTP/2 connection-pool poisoning) and the
per-track download orchestration (`try_cmaf_offline_download` for the v2
CMAF-first path, `spawn_track_cache_download` as the shared entry point that
tries CMAF then falls back to the legacy plain-FLAC fetch, tags, artwork,
and library-row insertion).

## Proposed split
By domain — the generic HTTP fetcher vs. the two download strategies vs.
the shared spawn/orchestration wrapper:

- `downloader/mod.rs` (~15 lines) — module doc, `pub use` of
  `StreamFetcher`, `spawn_track_cache_download`; `try_cmaf_offline_download`
  and `validate_download_size` stay `pub(crate)`, re-exported only within
  the crate if other files need them (check `crate::downloader::` grep
  first — `try_cmaf_offline_download` currently looks internal-only).
- `downloader/fetcher.rs` (~230 lines) — `StreamFetcher` struct + `new()`,
  `build_client()`, `fetch_to_file()` (the retry loop with backoff),
  `try_download()` (single-attempt streaming write with progress-event
  throttling), `fetch_to_memory()`, `impl Default`, plus
  `validate_download_size()` and its `#[cfg(test)] mod download_size_tests`
  (4 tests) — this whole cluster is the reusable "download bytes reliably"
  primitive, independent of CMAF/legacy specifics.
- `downloader/cmaf_path.rs` (~195 lines) — `try_cmaf_offline_download()`:
  the CMAF-first download strategy (fetch raw bundle, wrap keys via
  spawn_blocking around the secret vault, persist bundle, flip DB row to
  v2, fetch+save metadata/artwork, insert library row).
- `downloader/spawn.rs` (~250 lines) — `spawn_track_cache_download()`: the
  shared tokio::spawn wrapper that acquires the semaphore permit, tries the
  CMAF path first, falls back to legacy (`fetcher.fetch_to_file` +
  `write_flac_tags`/`embed_artwork`/`organize_cached_file` +
  bit-depth/sample-rate detection via `lofty` + library-row insertion),
  and updates DB status + emits `CacheEvent`s throughout both paths.

## Re-export surface
`downloader/mod.rs` re-exports `StreamFetcher` and
`spawn_track_cache_download` — the two symbols external code actually
calls (`crate::StreamFetcher`, `crate::downloader::spawn_track_cache_download`
or however the crate root wires it — check `lib.rs`'s existing `pub use
downloader::{...}` line and mirror exactly that set). `try_cmaf_offline_download`
stays `pub(crate)` and is only called from `spawn.rs`, so it doesn't need a
crate-root re-export, just `use super::cmaf_path::try_cmaf_offline_download;`
inside `spawn.rs`.

## Coupling / watch out
- `spawn_track_cache_download` calls `try_cmaf_offline_download` FIRST and
  only falls through to the legacy `fetcher.fetch_to_file` path on `Err` —
  this fallback ordering (CMAF-first, log-and-continue on failure) is the
  core behavior of the whole module per its own comments ("Falls through to
  the legacy path below if any step fails... keeps existing users unblocked
  while we validate the new path") — preserve this exact control flow,
  don't reorder or short-circuit it.
- `StreamFetcher::fetch_to_file`'s retry loop (`MAX_RETRIES = 3`,
  `RETRY_BACKOFFS`) creates a FRESH `reqwest::Client` per attempt — the
  comment on the struct explains why (HTTP/2 pool poisoning); this must
  stay true after the split — don't let anyone "optimize" it into a shared
  client across retries.
- `validate_download_size` is called from inside `try_download` (fetcher
  path) AFTER the streaming loop completes — its 4 tests
  (`accepts_matching_length`, `rejects_truncated`, `rejects_empty_even_without_length`,
  `accepts_nonzero_unknown_length`) exercise it directly as a pure fn, no
  network — keep it and its tests together in `fetcher.rs`.
- `spawn.rs`'s legacy-path post-processing (tag write → artwork embed →
  organize file → cover.jpg save → bit-depth detect via `lofty` → library
  insert → DB file-path update → `Processed` event) is a long sequential
  chain where each step logs-and-continues on error rather than aborting —
  this "best effort, never blocks the already-cached file" behavior must
  be preserved exactly; do not introduce early returns that would skip
  later steps (e.g. skipping the DB `update_file_path` if artwork save
  fails) since none exist in the original.
- `cmaf_path.rs`'s `try_cmaf_offline_download` uses `spawn_blocking` around
  the secret-vault wrap calls specifically because "The OS keyring... does
  a synchronous D-Bus round-trip via zbus, which PANICS... when run on an
  async worker" — this comment is safety-critical context, must travel
  with the code, not get trimmed.
- Both `cmaf_path.rs` and `spawn.rs`'s legacy branch independently build an
  `album_group_key` as `format!("{}|{}", metadata.album, album_artist)` and
  call `insert_qobuz_cached_track_with_grouping` with near-identical
  argument lists (differing mainly in path and bit-depth/sample-rate
  source) — this duplication predates the split and is NOT something to
  fix here (out of scope — pure move, no behavior change); note it for a
  future cleanup pass instead.
- Many `std::sync::{Arc, Mutex/RwLock}` and `tokio::sync::{Mutex, RwLock,
  Semaphore}` types are passed by full path (not `use`-imported) throughout
  — when splitting, either keep this fully-qualified style for a literal
  diff-minimal move, or add `use` statements consistently within each new
  file — don't mix.

## Verify after split
- `cargo test -p qbz-offline-cache download_size` (or wherever the moved
  test module's path resolves) — all 4 `validate_download_size` tests must
  stay green.
- `cargo check -p qbz-offline-cache` / `cargo build -p qbz-offline-cache`.
- Grep `crate::downloader::` and `downloader::StreamFetcher` across the
  crate/workspace to confirm the offline-cache manager (whatever calls
  `spawn_track_cache_download` to kick off a download) still resolves.
- Manually trigger one CMAF-path download and one forced-legacy-fallback
  download (e.g. by stubbing the CMAF fetch to fail) and confirm both
  still end with a correct library row, correct `cache_format`, and the
  right `CacheEvent` sequence (`Started` → `Progress`* → `Completed` →
  `Processed`, or `Failed`).
