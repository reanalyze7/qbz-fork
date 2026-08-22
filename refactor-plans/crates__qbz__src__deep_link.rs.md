# crates/qbz/src/deep_link.rs (236 lines)

## Summary
XDG/launcher deep-link handling: argv scanning at cold start, a pending-URL
slot, per-shell context binding, and dispatch through the existing Ctrl+L
link-resolver flow. Bridges cold-start argv AND the warm single-instance
D-Bus `OpenUrl` path to the same navigation code.

## Proposed split
By lifecycle stage — this file's own doc comment already describes two
entry paths (cold start / warm start) converging on one dispatch:

- `deep_link/mod.rs` (~45 lines) — `PENDING` static, `is_qobuz_link`,
  `select_link`, `capture_argv`, `stash`, `take_pending`, `pub use` of
  `shell_ctx` and `dispatch`.
- `deep_link/shell_ctx.rs` (~35 lines) — `ShellCtx` struct, `SHELL_CTX`
  static, `bind_shell_ctx`, `clear_shell_ctx`, `drain_pending`.
- `deep_link/dispatch.rs` (~40 lines) — the `dispatch` fn (resolve +
  navigate via `link_resolver`/`apply_resolved_link`).
- `deep_link/tests.rs` (~60 lines) — existing `#[cfg(test)] mod tests`.

## Re-export surface
`deep_link/mod.rs` stays the `mod deep_link;` target. Public fns called
from `main.rs`/`single_instance.rs` (`is_qobuz_link`, `capture_argv`,
`stash`, `take_pending`, `bind_shell_ctx`, `clear_shell_ctx`,
`drain_pending`) re-exported via `pub use shell_ctx::*;` — `dispatch` stays
private, called only internally from `drain_pending`.

## Coupling / watch out
- `single_instance.rs`'s `open_url` D-Bus method calls `crate::deep_link::
  stash` + `drain_pending` directly — keep both at the same
  `crate::deep_link::X` path (unaffected by this internal split).
- `PENDING` (mod.rs) is written by `capture_argv`/`stash` and read by
  `take_pending`/`drain_pending` — `drain_pending` lives in `shell_ctx.rs`
  per this plan, so it needs `use super::{take_pending, PENDING};` or just
  call the public `take_pending()` fn (cleaner — avoids reaching into the
  private static from another file at all).
- The doc comment's ordering guarantee ("`capture_argv()` ... BEFORE
  `single_instance::acquire_or_raise`") is a `main.rs`-level call-order
  contract, not something this split can enforce — just don't lose the
  comment explaining it.
- `dispatch`'s comment explicitly says it "mirrors the Ctrl+L
  `LinkResolverActions::on_submit` flow in `main.rs`" — if that flow ever
  changes, this fn needs a matching update; flag for whoever maintains
  both.

## Verify after split
- `cargo test -p qbz deep_link::` — all 7 existing tests green
  (link-matching table + the stateful `pending_drains_once_and_newest_wins`
  test, which the comment notes must stay a single sequential test due to
  the process-global `PENDING`).
- `cargo build -p qbz`.
- Manual test: launch with a `qobuzapp://` URL argv, confirm navigation
  after `enter_shell`; then test the warm D-Bus path (second launch with a
  URL while the first is already running).
