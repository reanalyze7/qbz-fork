# crates/qbz-cmaf/src/crypto.rs (191 lines)

## Summary
CMAF/FLAC decryption crypto primitives for Qobuz streaming: HKDF session-key
derivation, AES-CBC content-key unwrapping, AES-CTR frame decryption, and the MD5
request-signature helper used by the CMAF session/start API calls.

## Proposed split
Just barely over budget (mostly due to the `#[cfg(test)]` block, ~65 lines). Split
production code from tests, which is also the pure/IO-adjacent boundary here (these
are all pure crypto functions, no IO) — so split by "algorithm concern" instead:

- `crypto/mod.rs` (~15 lines) — module doc, `use` block, type aliases
  (`Aes128CbcDec`, `Aes128Ctr`), re-exports of all pub items.
- `crypto/session_key.rs` (~55 lines) — `hex_decode` (private helper) +
  `derive_session_key` (HKDF derivation).
- `crypto/content_key.rs` (~40 lines) — `unwrap_content_key` (AES-CBC unwrap) +
  `decrypt_frame` (AES-CTR frame decrypt) — the two "operate on a per-track key"
  functions.
- `crypto/signature.rs` (~25 lines) — `compute_request_sig` (MD5 request signing).
- `crypto/tests.rs` (~65 lines) — the entire `#[cfg(test)] mod tests` block, included
  via `#[path] mod tests;` or as `#[cfg(test)] mod tests;` referencing the split
  functions through `super::*`.

## Re-export surface
`crypto/mod.rs` re-exports `derive_session_key`, `unwrap_content_key`,
`decrypt_frame`, `compute_request_sig` at `crate::crypto::*` so existing callers
(likely the CMAF session/segment-fetch code) are unaffected.

## Coupling / watch out
- `hex_decode` is a private helper used only by `derive_session_key` — keep it
  colocated in `session_key.rs`, do not promote to `mod.rs`.
- All functions take `seed: &str` / key material as plain params (no shared mutable
  state) — this file has no interior coupling beyond the type aliases, so the split
  is low-risk.
- `CmafError` comes from `crate::error` — re-import in each submodule.

## Verify after split
- `cargo test -p qbz-cmaf crypto` — all 7 existing unit tests must stay green.
- `cargo check -p qbz-cmaf` for any downstream crate depending on `qbz_cmaf::crypto::*`.
