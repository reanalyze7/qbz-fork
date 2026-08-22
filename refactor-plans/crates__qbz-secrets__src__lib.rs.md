# `crates/qbz-secrets/src/lib.rs` (190 lines)

## 1. Summary
Crate root for `qbz-secrets`: an extensive module-level doc comment (design
rationale, threat model, API example — ~72 lines) followed by module
declarations/re-exports, the `SecretBox` handle struct (wrap/unwrap/
backend_kind, backed by an OS-keyring-or-HKDF `Backend`), and its
round-trip/tamper-detection unit tests.

## 2. Proposed module layout

This file is only marginally over budget (190 vs. 130), almost entirely
because of the (valuable, keep-verbatim) crate doc comment. A minimal,
low-risk split:

- `lib.rs` (~95) — the full crate doc comment unchanged, `mod backend; mod
  cipher; mod envelope; mod error; mod install_id; mod secret_box;`, and
  `pub use backend::{Backend, BackendKind}; pub use envelope::WrappedSecret;
  pub use error::SecretError; pub use secret_box::SecretBox;`. **This is
  the re-export/public-API surface** — identical import paths
  (`qbz_secrets::SecretBox`, `qbz_secrets::SecretError`, etc.) for every
  external caller.
- `secret_box.rs` (~50) — the `SecretBox` struct + its `impl` (`open`,
  `from_backend`, `wrap`, `unwrap`, `backend_kind`).
- `secret_box.rs` also keeps its own `#[cfg(test)] mod tests { ... }` at the
  bottom (~55 lines: `test_vault()` helper + the 4 round-trip/tamper tests)
  — co-located with the code under test rather than a separate `tests.rs`,
  since combined `secret_box.rs` (struct + impl + tests) lands at ~105
  lines, comfortably under 130.

## 3. Re-export / public API surface
`lib.rs` stays the crate root and the only file external crates import
from (`use qbz_secrets::{SecretBox, SecretError, BackendKind, ...}`) — no
downstream code should need to change a single import.

## 4. Tricky coupling to watch
- `SecretBox::open` constructs `Backend::new(service_name, storage_dir)` —
  `Backend` is defined in the pre-existing `backend.rs` module (not part of
  this split), so `secret_box.rs` needs `use crate::backend::Backend;` (or
  `use crate::{Backend, ...}` if re-exported) — verify the exact import
  path `Backend` is referenced by today (it's currently in-scope via the
  same-file `mod backend;` + implicit crate-root resolution) and preserve
  it.
- `#[doc(hidden)] pub fn from_backend` exists specifically for test/
  integration use from OUTSIDE this crate (or from `backend.rs`'s own
  tests) — confirm no other crate or test harness relies on
  `qbz_secrets::SecretBox::from_backend` or on `SecretBox`'s field layout
  (it's a single private `backend: Arc<Backend>` field, should be
  unaffected by the file move).
- The crate doc's runnable-looking example (`use qbz_secrets::SecretBox;
  ... vault.wrap(...)`, marked `rust,ignore`) references the crate's public
  path, not an internal one — no change needed there, just don't let it
  drift out of sync with the real `SecretBox` API during the split.

## 5. What to verify after the real split
- `cargo test -p qbz-secrets` — all 4 existing tests (roundtrip_small_secret,
  roundtrip_16_byte_content_key, tampering_is_detected,
  two_wraps_of_same_plaintext_differ) stay green; these exercise the real
  keyring-or-HKDF backend so a broken import path will fail immediately at
  compile time, not silently.
- `cargo build --workspace` — confirm the offline-cache code in
  `qbz-library`/`qbz-app` (the stated primary consumer) still resolves
  `qbz_secrets::SecretBox` unchanged.
- `cargo doc -p qbz-secrets` to confirm the crate-level doc comment still
  renders attached to the crate root (doc comments must stay as the FIRST
  thing in `lib.rs`, before any `mod` statements, or they silently stop
  being crate-level docs).
