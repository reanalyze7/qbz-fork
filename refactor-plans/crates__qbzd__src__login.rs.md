# crates/qbzd/src/login.rs (880 lines)

## Summary
`qbzd login`/`qbzd logout` implementation: three OAuth entry points
(system-browser listener, paste-URL, direct-token), the daemon-reload
"nudge" ping/reload logic, plus a large batch of pure URL/callback-parsing
helpers and their unit tests (~270 of the 880 lines are `#[cfg(test)]`).

## Proposed split
By responsibility — public entry points vs pure parsing/URL-building logic
vs IO helpers vs error mapping — turning this into a `login/` directory
module:

- `login/mod.rs` (~20 lines) — module doc (lines 1-20) + `pub use`
  re-exports of every public item, so `crate::login::X` (used by `main.rs`
  and CLI commands) is unaffected.
- `login/error.rs` (~75 lines) — `LoginError` enum + its `Display`/`Error`
  impls (lines 41-80), plus `map_api_err`/`map_core_err` (lines 588-608).
- `login/entry.rs` (~200 lines) — the three public async entry points
  (`validate_token`, `login_browser`, `login_paste`,
  `login_with_token_arg`, lines 84-198) plus `logout`/`NudgeOutcome`/
  `nudge_reload_outcome`/`nudge_reload` (lines 200-245). This is the
  "public API" behavioral core.
- `login/parsing.rs` (~130 lines) — the pure, unit-tested helpers: lines
  249-394 (`resolve_callback_host`, `build_oauth_url`,
  `bracket_ipv6_for_url`, `parse_callback`, `code_from_paste`, `url_path`,
  `code_from_query`, `decode`, `gen_nonce`). All pure/no-IO — matches the
  file's own `// pure, unit-tested` section marker.
- `login/io.rs` (~160 lines) — lines 401-584: `build_login_runtime`,
  `read_app_id`, `exchange_code`, `finalize`, `nudge_host`,
  `bind_login_listener`, `read_stdin_line`, `capture_callback`,
  `http_request_2xx`, plus the `SUCCESS_HTML`/`WAITING_HTML` constants.
- `login/tests.rs` or per-file `#[cfg(test)] mod tests` split alongside
  each of the above (lines 610-880) — since the existing tests already
  group cleanly by which helper they exercise (OAuth URL building,
  `resolve_callback_host`, `bind_login_listener`, `parse_callback`,
  `code_from_paste`, `gen_nonce`, `nudge_reload`), move each test group
  into a `#[cfg(test)] mod tests` block at the bottom of its corresponding
  new file (`parsing.rs` tests, `io.rs` tests) rather than one big shared
  tests file — keeps tests next to what they test and each file under 130
  lines including its own tests.

## Re-export surface
`login/mod.rs` stays the public surface: `pub use error::{LoginError};
pub use entry::{validate_token, login_browser, login_paste,
login_with_token_arg, logout, NudgeOutcome, nudge_reload_outcome,
nudge_reload}; pub(crate) use io::...` as needed. The crate's root
(`lib.rs`/`main.rs`) line `mod login;` needs no change since `login/mod.rs`
resolves identically to the current `login.rs`.

## Coupling / watch out
- `LoginError::Display` (in `error.rs`) calls
  `crate::cli::copy::login_timeout` — unaffected by the split (absolute
  crate path), but note `cli/copy.rs` is ALSO one of this batch's files
  (see `crates__qbzd__src__cli__copy.rs.md`) — the two files are coupled
  via this call, so if `copy.rs`'s `login_timeout` signature changes during
  its own split, `login/error.rs` must be updated too.
- `entry.rs`'s `login_browser` calls into `parsing.rs` (`resolve_callback_
  host`, `gen_nonce`, `build_oauth_url`) and `io.rs` (`bind_login_listener`,
  `capture_callback`) — both cross-file calls are plain function calls once
  everything is `mod`-included under `login/`, no visibility changes needed
  since helpers are currently private (`fn`, not `pub fn`) — they'll need
  `pub(crate)` or `pub(super)` visibility to be callable from a sibling file
  in the new directory structure (private-by-default breaks across files
  even within the same module tree unless explicitly scoped).
- `finalize` (io.rs) is called from three of the four public entry points
  in `entry.rs` — keep it `pub(super)` so `entry.rs` can reach it.
- `nudge_host` (io.rs) is used by both `entry.rs`'s `logout`/`nudge_reload`
  paths AND is `pub(crate)` already (referenced from outside this module,
  check for external callers via `crate::login::nudge_host` before making
  it more private than it already is).
- The pure helpers in `parsing.rs` currently have NO qualification prefix
  (`fn parse_callback`, not `pub fn`) except a few explicitly `pub fn` ones
  (`resolve_callback_host`, `build_oauth_url`, `parse_callback`,
  `code_from_paste` are `pub fn`; `bracket_ipv6_for_url`, `url_path`,
  `code_from_query`, `decode`, `gen_nonce` are private) — the private ones
  are only called from within `parsing.rs` itself in the current file, so
  they can stay `fn` (private to the new `parsing.rs` file) without any
  visibility widening.

## Verify after split
- `cargo build -p qbzd`.
- `cargo test -p qbzd login::` — all ~35 unit tests in this file must still
  pass (OAuth URL building/bracketing, `resolve_callback_host` priority
  order, `bind_login_listener` family-aware binding, `parse_callback`/
  `code_from_paste` nonce validation, `gen_nonce` uniqueness/format,
  `nudge_reload`/`nudge_outcome` daemon-down detection, the verbatim
  `LoginError::Timeout` rendering).
- `cargo clippy -p qbzd`.
- Smoke-test importers: `grep -rn "login::" crates/qbzd/src` — confirm
  `main.rs`/CLI command dispatch still resolves `login::validate_token`,
  `login::login_browser`, `login::login_paste`,
  `login::login_with_token_arg`, `login::logout`, `login::nudge_reload`.
