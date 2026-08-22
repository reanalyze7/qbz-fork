# crates/qbzd/src/tui/screens/account.rs (196 lines)

## Summary
qbzd setup TUI's Account screen (03 §3.1): status row (never fabricates a
name offline) + login/logout/paste-token actions. All auth logic is
delegated to the T5 engine (`login.rs`) — this file is state + key
handling + rendering only. Small, single natural split point between key
handling and drawing.

## Proposed split
- `account/mod.rs` (~15 lines) — re-exports `AuthSnapshot`, `AccountState`.
- `account/state.rs` (~100 lines) — `AuthSnapshot` struct, `AccountState`
  struct, `new`, `set_auth`, `is_editing`, `editing_label`, `actions`,
  `handle_key` (lines 20-136).
- `account/draw.rs` (~60 lines) — the `draw` method (lines 140-195), as a
  second `impl AccountState` block.

Given the modest overage (66 lines), a 2-file split is sufficient — no
need for finer-grained sectioning.

## Re-export surface
`account/mod.rs` becomes the `mod account;` target; `pub use state::*;`
keeps `AuthSnapshot`/`AccountState` reachable at
`crate::tui::screens::account::{AuthSnapshot, AccountState}` unchanged.

## Coupling / watch out
- `AccountState`'s fields (`auth`, `focus`, `token_input`,
  `confirm_logout`) are private — `draw.rs`'s `impl AccountState` block
  needs them visible, so keep both files as siblings under the same
  `account` module (module-private fields are visible within the module
  tree, same pattern as the wizard/bundle splits in this batch).
- `draw()` calls `self.actions()` and `self.is_editing()`, both defined in
  `state.rs` — no special handling needed, just normal cross-file method
  calls within the same `impl`'d type.
- The Account screen intentionally has no `is_dirty` (comment at line
  52-53: "Account is all immediate actions... the App's `active_is_dirty`
  short-circuits it") — preserve that comment, it explains an absence
  that would otherwise look like an oversight.

## Verify after split
- `cargo build -p qbzd`.
- Manually exercise: status display (logged out / cred-file-present /
  logged in with email+plan), browser login handoff, paste-token flow,
  logout confirm (y/n/Esc).
