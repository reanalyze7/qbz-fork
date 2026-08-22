# crates/qbz-qobuz/src/cmaf.rs (534 lines)

CMAF streaming orchestration for Qobuz: session setup, full-download
(decrypted FLAC) and raw-download (encrypted bundle for offline cache)
variants, concurrent segment fetch with retry, in-place decryption.

## Proposed split

- `cmaf/mod.rs` (~60 lines) — re-export surface + `CmafProgressCallback`,
  `CmafProgressUpdate`, `CmafStreamingInfo`, `CmafRawBundle`,
  `CMAF_PREFETCH_CONCURRENCY` const.
- `cmaf/setup.rs` (~90 lines) — `setup_streaming` (fetch/derive-keys/parse
  init segment).
- `cmaf/download_full.rs` (~90 lines) — `download_full`,
  `download_full_with_progress`, `download_full_with_quality`,
  `download_full_with_quality_progress`.
- `cmaf/download_raw.rs` (~90 lines) — `download_raw`,
  `download_raw_with_progress`.
- `cmaf/fetch.rs` (~110 lines) — `build_cdn_client`,
  `fetch_bytes_with_retry`, `fetch_all_segments` (shared CDN fetch
  plumbing used by both download variants).
- `cmaf/decrypt.rs` (~40 lines) — `decrypt_segments_into`.

## Tricky coupling

- `setup_streaming` (setup.rs) is called by both `download_full.rs` and
  indirectly informs `download_raw.rs`'s duplicated session/key derivation
  — note `download_raw_with_progress` does NOT call `setup_streaming` (it
  re-derives the session/key inline); keep that duplication as-is rather
  than "fixing" it during the split (behavior-preserving refactor only).
- `fetch_all_segments` and `fetch_bytes_with_retry` (fetch.rs) are shared by
  both download paths — must be `pub(crate)` or `pub(super)` visible from
  `download_full.rs`/`download_raw.rs`.
- `decrypt_segments_into` is `pub` (used elsewhere, e.g. offline-cache
  playback) — keep it `pub` in `decrypt.rs` and re-export from `mod.rs`.

## Verify after split

`cargo build -p qbz-qobuz`, `cargo test -p qbz-qobuz cmaf::` (2 existing
tests are minimal; also grep for external users of
`qbz_qobuz::cmaf::decrypt_segments_into` / `CmafRawBundle` in the offline
cache and confirm those import paths still resolve).
