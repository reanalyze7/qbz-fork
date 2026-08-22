# crates/qbz/src/auth.rs (423 lines)

## Summary
System-browser OAuth login for the Slint app: opens the Qobuz OAuth page,
captures the redirect code on a one-shot local HTTP listener, exchanges it
for a session, activates the per-user runtime (a long chain of per-feature
`init_for_user` calls), plus saved-session restore and logout.

## Proposed split
By responsibility: public login/restore/logout entry points, the shared
per-user activation sequence, the local HTTP listener, tests. The two
biggest duplication hot-spots (`login_via_system_browser` and
`restore_saved_session` both run nearly the same ~30-line per-user
activation block) should be unified into one shared helper during the
split, not just relocated verbatim.

- `auth/mod.rs` (~35 lines) — module doc, `pub mod` declarations, `pub use`
  re-exports of `SessionInfo`, `LoginPhase`, `login_via_system_browser`,
  `restore_saved_session`, `logout` so `crate::auth::X` paths are
  unchanged.
- `auth/session_activation.rs` (~90 lines) — a NEW shared
  `activate_user_session(runtime, user_id)` async fn extracted from the
  near-duplicate block in both `login_via_system_browser` and
  `restore_saved_session` (offline::activate, offline_cache load,
  offline_mode/fav_cache/reco_dismiss/reco/external_reco/artist-vector
  store/discover_prefs/artist_blacklist/pinned/local_favorites/
  search_service/session_persist init, subscription_mark_valid,
  set_offline_session(false)) — collapses ~60 duplicated lines into one
  call site per caller. This is the one place worth actually changing
  behavior-adjacent structure (not just moving code) since the two blocks
  are already comment-flagged as parity copies of each other.
- `auth/login.rs` (~110 lines) — `SessionInfo`, `LoginPhase`,
  `login_via_system_browser` (now short: init api, get app_id, bind
  listener, open browser, await code, exchange, call
  `session_activation::activate_user_session`, persist token).
- `auth/restore.rs` (~90 lines) — `restore_saved_session`, `is_auth_
  rejection` (now short: load token, `ensure_api_initialized`, exchange,
  call `session_activation::activate_user_session` on success, clear-on-
  rejection / keep-on-network-failure branches).
- `auth/logout.rs` (~25 lines) — `logout`.
- `auth/api_init.rs` (~25 lines) — `ensure_api_initialized` (the offline-
  session-flag-lift-around-cold-init helper, shared by login + restore).
- `auth/oauth_listener.rs` (~55 lines) — `OAUTH_TIMEOUT`, `capture_oauth_
  code`, `query_param`.
- `auth/tests.rs` (~20 lines) — the `#[cfg(test)] mod tests` block
  (`query_param` tests).

## Re-export surface
`auth/mod.rs` re-exports `SessionInfo`, `LoginPhase`,
`login_via_system_browser`, `restore_saved_session`, `logout` at
`crate::auth::*` — these are called from the Slint login screen's
Rust-side handlers (search for `auth::login_via_system_browser`,
`auth::restore_saved_session`, `auth::logout` call sites in
`crates/qbz/src/` before finalizing; likely in a `login`-related UI
wiring module).

## Coupling / watch out
- The extracted `activate_user_session` helper touches ~15 different
  per-feature modules' `init_for_user`/similar functions
  (`offline`, `offline_cache`, `offline_mode`, `fav_cache`, `reco_dismiss`,
  `reco`, `external_reco`, `qbz_reco::ArtistVectorStore`,
  `discover_prefs`, `artist_blacklist`, `pinned`, `local_favorites`,
  `search_service`, `session_persist`) — this is the widest coupling
  surface in the file; keep the exact CALL ORDER when extracting (several
  comments note ordering dependencies, e.g. "after offline::activate so
  the purge consumer can reach the cache", "Train after init (off-thread)
  so the seeds reflect this session's events"). Do not alphabetize or
  reorder these calls.
- `qbz_log::register_secret(token.clone())` must stay called exactly once
  per path (login and restore each register their own token) — don't
  accidentally fold this into the shared activation helper since the
  token differs by call site (login: fresh OAuth token; restore: loaded
  token) and registration timing relative to `ensure_api_initialized`
  matters (restore registers before, login registers after exchange).
- `is_auth_rejection` is a pure classifier used only inside
  `restore_saved_session`'s match arm — keep it next to `restore.rs`, not
  in a generic "errors" module, since its doc comment is specific to the
  D1 boot-token-clearing bug this file guards against.
- `logout`'s teardown call list must mirror (in reverse-ish) the
  activation call list's init list — if a new per-feature module gets an
  `init_for_user` added to `session_activation.rs`, its `teardown` almost
  certainly needs adding to `logout.rs` too; leave a cross-reference
  comment in both files.

## Verify after split
- `cargo test -p qbz auth::` — the `query_param` tests green.
- `cargo check -p qbz` (or full workspace) to confirm
  `crate::auth::{login_via_system_browser,restore_saved_session,logout}`
  call sites in the Slint login screen wiring still resolve.
- Manual smoke-test (requires a real Qobuz account): fresh login via
  system browser, confirm the browser opens, sign in, confirm the app
  activates the session (Discover/Library populate); quit and relaunch,
  confirm the saved session restores silently; log out, confirm the app
  returns to the login screen and a relaunch does NOT auto-restore.
