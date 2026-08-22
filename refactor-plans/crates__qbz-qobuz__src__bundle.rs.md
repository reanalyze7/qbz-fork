# crates/qbz-qobuz/src/bundle.rs (413 lines)

Extracts app_id/secrets/private_key from the Qobuz web player JS bundle,
with a disk cache keyed by bundle version and a background version-check
refresh.

## Proposed split

- `bundle/mod.rs` (~90 lines) — re-export surface, `BundleTokens`,
  `CachedBundle` (+ `From<CachedBundle> for BundleTokens`), the URL/timeout
  consts.
- `bundle/cache.rs` (~60 lines) — `cache_path`, `load_cached_bundle`,
  `save_cached_bundle`, `now_unix`.
- `bundle/fetch.rs` (~110 lines) — `bundle_version_from_url`,
  `fetch_bundle_url`, `extract_bundle_tokens_once`,
  `extract_and_cache_bundle_tokens`, `refresh_bundle_if_changed`,
  `extract_bundle_tokens` (the network/orchestration layer).
- `bundle/parse.rs` (~140 lines) — the pure regex extraction:
  `extract_bundle_url`, `extract_app_id`, `extract_secrets`,
  `extract_private_key`.
- `bundle/tests.rs` (~20 lines) — existing test module.

## Tricky coupling

- `fetch.rs`'s `extract_bundle_tokens_once` calls `parse.rs`'s
  `extract_app_id`/`extract_secrets`/`extract_private_key` — pure fns, easy
  import.
- `cache.rs` and `fetch.rs` both need `now_unix`/`CachedBundle` — keep those
  in `mod.rs` (or `cache.rs`, with `fetch.rs` importing from there).

## Verify after split

`cargo build -p qbz-qobuz`, `cargo test -p qbz-qobuz bundle::` (2 existing
regex-extraction tests).
