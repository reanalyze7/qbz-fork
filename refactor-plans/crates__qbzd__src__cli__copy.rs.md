# crates/qbzd/src/cli/copy.rs (256 lines)

## Summary
Normative, spec-verbatim CLI copy: pure string-formatting functions for the
auth verbs (login timeout/success, logout, SSH-detection note), the
general error-voice family (daemon-down, needs-auth, linger-off, DSD
volume/seek errors, port conflicts, version skew), and the settings-bundle
copy (export/import warnings and errors) — plus one pinned-wording unit
test.

## Proposed split
By spec section / topic — this file is already just flat functions with no
shared state, so the split is a clean by-domain partition into a `copy/`
directory:

- `copy/mod.rs` (~15 lines) — module doc (lines 1-6), `pub use`
  re-exports of every function from the sub-files below.
- `copy/auth.rs` (~90 lines) — lines 9-56: `login_timeout`,
  `login_ssh_detected`, `login_browser_open_failed`, `login_success`,
  `logout_success`. All auth/login-verb copy (02 §2.2).
- `copy/daemon_errors.rs` (~100 lines) — lines 58-160: `daemon_down`,
  `daemon_up_needs_auth`, `linger_off`, `volume_fixed_dsd`,
  `seek_unsupported_dsd`, `port_in_use`, `foreign_qbzd`,
  `lan_posture_note`, `version_skew`, `api_version_skew`. The general
  error-voice family (02 §1.4/§1.6/§6.3).
- `copy/bundle.rs` (~95 lines) — lines 162-231: `basename` (private
  helper), `bundle_secret_warning`, `bundle_export_success`,
  `bundle_no_desktop_profile`, `bundle_token_decrypt_failed`,
  `bundle_version_too_new`, `bundle_token_rejected`. Settings-bundle copy
  (04-settings-portability.md).
- The one test (`lan_posture_note_renders_the_verbatim_copy`, lines
  232-256) moves into a `#[cfg(test)] mod tests` at the bottom of
  `copy/daemon_errors.rs` (it tests a `daemon_errors.rs` function).

## Re-export surface
`copy/mod.rs` stays the public surface: `pub use auth::*; pub use
daemon_errors::*; pub use bundle::*;`. The crate's `lib.rs`/`cli/mod.rs`
line `pub mod copy;` (under `cli/`) needs no change — `cli/copy/mod.rs`
resolves identically to the current `cli/copy.rs`. Every call site
(`crate::cli::copy::login_timeout(...)`, etc., referenced from
`login.rs`, `client.rs`, and others) is unaffected.

## Coupling / watch out
- Every function here is pure (`String`/`&'static str` in, formatted string
  out) — genuinely zero shared state or cross-function coupling, making
  this one of the lowest-risk splits in the batch. The only "coupling" is
  external: callers reaching in via `crate::cli::copy::<name>`.
- `basename` (private helper in the `bundle` section) is used by
  `bundle_secret_warning` and `bundle_export_success` — keep it private to
  `copy/bundle.rs` (both callers live there after the split, no
  cross-file visibility needed).
- The doc comments above each function cite exact spec section numbers
  (02 §1.4, §1.6, §2.2, §6.3; 04 §3, §4.1, §5.3, §5.6) and say the wording
  is "verbatim ... modulo interpolated values" — preserve these doc
  comments exactly when moving functions; they're the reason nobody should
  casually reword the returned strings.
- `login.rs` (also in this batch) calls `crate::cli::copy::login_timeout`
  from `LoginError::Display` — confirm that call site still resolves
  after `copy.rs` becomes `copy/mod.rs` (it will, since the path
  `crate::cli::copy::login_timeout` is unaffected by internal
  file-splitting as long as `copy/mod.rs` re-exports it).

## Verify after split
- `cargo build -p qbzd`.
- `cargo test -p qbzd cli::copy::` — the one pinned-wording test
  (`lan_posture_note_renders_the_verbatim_copy`) must still pass verbatim.
- `cargo clippy -p qbzd`.
- Smoke-test importers: `grep -rn "cli::copy::" crates/qbzd/src` — confirm
  every call site (`login.rs`, error-envelope rendering in `cli/client.rs`
  or similar, `settings` bundle commands) still compiles against the same
  function names.
