# crates/qbz-qobuz/src/auth.rs (256 lines)

## Summary
Authentication and request-signing helpers for the Qobuz API: MD5 request
signatures (generic + per-endpoint helpers), the `parse_login_response`
parser that builds a `UserSession` from the raw JSON login response, and the
CMAF session/file-url signature functions.

## Proposed split
By responsibility — request signing vs login-response parsing vs CMAF
signing are three genuinely distinct concerns bundled in one file:

- `auth/mod.rs` (~15 lines) — module doc, `pub use` re-exports of
  `generate_signature`, `sign_get_file_url`, `sign_get_favorites`,
  `sign_search`, `sign_request`, `get_timestamp`, `parse_login_response`,
  `CMAF_SEED`, `sign_session_start`, `sign_file_url`.
- `auth/signing.rs` (~60 lines) — `generate_signature`, `sign_get_file_url`,
  `sign_get_favorites`, `sign_search`, `sign_request`, `get_timestamp`. The
  generic MD5-signature machinery for the classic (non-CMAF) API.
- `auth/login.rs` (~115 lines) — `parse_login_response` (including its
  nested `parse_subscription_valid_until` helper — promote that to a
  private top-level fn in this file rather than nested, for readability,
  though nesting also works if the reviewer prefers to preserve it exactly).
- `auth/cmaf.rs` (~20 lines) — `CMAF_SEED`, `sign_session_start`,
  `sign_file_url`. The CMAF-specific signing pair.
- `auth/tests.rs` (~55 lines) — the whole `#[cfg(test)] mod tests` block
  (test_generate_signature, test_sign_get_file_url, the `login_response`
  builder, parse_login_response_captures_country_and_language,
  parse_login_response_tolerates_missing_country_and_language), `use
  super::*;` pulling from both `signing` and `login`.

At 256 lines this is close to 2x budget — a simpler 2-way split
(signing+cmaf vs login+tests) would also work if the reviewer wants fewer
files; the 5-file version above keeps every module comfortably under 130
with room to grow.

## Re-export surface
`auth/mod.rs` is the `mod auth;` target already used as
`qbz_qobuz::auth::X` (or wherever this crate's `lib.rs` re-exports it —
check for `pub mod auth;` in `qbz-qobuz/src/lib.rs`). All current public
items stay reachable via `pub use signing::*; pub use login::*; pub use
cmaf::*;`.

## Coupling / watch out
- `parse_login_response` depends on `super::error::{ApiError, Result}` (a
  sibling module in this crate) and `qbz_models::UserSession` — these
  imports need to move into `auth/login.rs`, not stay at the old `auth.rs`
  top level.
- `sign_session_start`/`sign_file_url` (cmaf.rs) depend on the external
  `qbz_cmaf` crate's `compute_request_sig` and the `CMAF_SEED` const
  defined right above them — keep the const and its two consumers together
  in one file (as proposed) rather than separating them.
- `IneligibleUser` early-return in `parse_login_response` (when
  `has_subscription` is false) is a load-bearing gate — don't let it get
  lost or reordered relative to the `country_code`/`language_code`
  extraction above it during the split; the doc comment references a
  specific investigation doc (`qbz-nix-docs offline-mode/tauri-review-...`)
  as the source of truth for those field names — preserve that comment.
- No other file in this crate is in this batch, but `auth.rs` sits next to
  `error.rs` in the same crate (`use super::error::...`) — if another
  agent's slice also touches `qbz-qobuz/src/error.rs`, flag that both
  reference each other via `super::`.

## What to verify after the real split
- `cargo build -p qbz-qobuz`.
- `cargo test -p qbz-qobuz auth::` — all 4 existing tests green
  (test_generate_signature, test_sign_get_file_url,
  parse_login_response_captures_country_and_language,
  parse_login_response_tolerates_missing_country_and_language).
- Smoke-test importers: grep for `auth::sign_` / `auth::parse_login_response`
  / `auth::CMAF_SEED` call sites elsewhere in the crate (the request-signing
  call sites in the API client, the login flow) and confirm they resolve
  unchanged.
