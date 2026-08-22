# crates/qbz-secrets/src/backend.rs (170 lines)

## 1. Summary
Selects and dispatches between two secret-storage backends at open time: an
OS-keyring-backed master key (preferred) or an HKDF-derived fallback key from
device identifiers, both used for AES-256-GCM wrap/unwrap via `cipher.rs`,
with a backend discriminator marker baked into every wrapped blob.

## 2. Proposed module split
| New file | Owns | ~lines |
|---|---|---|
| `backend/mod.rs` | Module decls + re-exports; module doc comment; the shared constants (`MASTER_KEY_LEN`, `KEYRING_ENTRY_NAME`, `HKDF_INFO`, `BACKEND_MARKER_KEYRING`, `BACKEND_MARKER_KDF`) | ~35 |
| `backend/kind.rs` | `BackendKind` enum + its doc comments | ~15 |
| `backend/backend.rs` | `Backend` struct + its `impl` (`new`, `kind`, `wrap`, `unwrap`) — the public dispatch surface | ~55 |
| `backend/keyring.rs` | `try_open_keyring` (OS keyring read/create) | ~40 |
| `backend/kdf.rs` | `derive_fallback_key` (HKDF-over-device-identifiers fallback) | ~35 |

This is only marginally over 130 lines, so a light split suffices — the goal
is mainly separating "which backend" (`kind.rs`), "the public wrap/unwrap
dispatcher" (`backend.rs`), and the two backend-specific key-acquisition
strategies (`keyring.rs`, `kdf.rs`) so each strategy can be read/tested in
isolation.

## 3. Re-export / public API surface
`backend/mod.rs` re-exports the current public surface:

```rust
mod backend;
mod kdf;
mod keyring;
mod kind;

pub use backend::Backend;
pub use kind::BackendKind;
```

(`try_open_keyring` and `derive_fallback_key` stay private/`pub(super)` —
they were never part of the public API, only called from `Backend::new`.)
Every caller doing `use qbz_secrets::backend::{Backend, BackendKind};` (the
crate's top-level secrets-manager, likely in `lib.rs`) keeps working
unchanged.

## 4. Tricky coupling / shared-state to watch out for
- `Backend::new` tries `try_open_keyring` FIRST and only falls back to
  `derive_fallback_key` on ANY error — this fallback ordering (and the
  `log::warn!` on fallback) is the core security/UX contract of the module;
  keep it as one function in `backend.rs` calling into the two split-out
  strategy functions, not re-implemented per-file.
- `Backend::wrap`/`unwrap` both re-derive the marker byte from `self.kind`
  via an identical `match` — this duplication already exists in the current
  file; preserve it as-is during the split (don't "fix" it into a shared
  helper as part of a pure reorg — that's a separate behavior-neutral
  cleanup best done deliberately, not accidentally during a file-split PR).
- `BACKEND_MARKER_KEYRING`/`BACKEND_MARKER_KDF` (u8 constants) are baked into
  every wrapped blob via `wrap_with_key`/`unwrap_with_key` (from
  `crate::cipher`) — these markers are a durable ON-DISK format, so they must
  keep their exact values (`0`/`1`) regardless of which file declares them;
  put them in `mod.rs` (shared by both `backend.rs` and any future
  migration code) rather than duplicating in `kind.rs`.
- `derive_fallback_key` depends on `crate::install_id::{load_or_create,
  machine_id}` — the IKM assembly order (`service_name` + NUL +
  `machine_id` + NUL + `install_uuid`) is part of the derivation's
  reproducibility contract; keep it byte-for-byte identical in `kdf.rs`.
- `try_open_keyring`'s three-way match (`Ok` existing / `Err(NoEntry)`
  generate-and-store / other `Err`) plus its base64 encode/decode round-trip
  must move together into `keyring.rs` — splitting the match arms across
  files would break the "create on first run" semantics.

## 5. What to verify after the real split
- `cargo build -p qbz-secrets` and `cargo test -p qbz-secrets` (check for
  backend-specific unit/integration tests exercising both the keyring and
  KDF-fallback paths, and the wrap/unwrap round-trip with the marker byte).
- Grep the workspace for `backend::` usages outside this crate (the offline
  cache / secrets-manager wiring in `qbz-app` or `qbz`) to confirm
  `Backend`/`BackendKind` import paths still resolve.
- Smoke-test: run the app on a machine both WITH and WITHOUT OS-keyring
  access available (e.g. a headless/CI-like environment or by denying
  keyring access) and confirm secrets still wrap/unwrap correctly via the
  KDF fallback, and that a blob wrapped under one backend correctly fails
  (rather than silently misdecrypting) when the OTHER backend attempts
  `unwrap` on it (the discriminator-marker guarantee).
