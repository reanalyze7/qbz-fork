# crates/qbz/src/single_instance.rs (167 lines)

## Summary
Linux-only single-instance guard via D-Bus well-known name +
`SingleInstanceIface` (`Present`/`OpenUrl`), with MPRIS `Raise` fallback for
older primaries. Ensures a second launch presents/navigates the existing
window instead of spawning a duplicate process.

## Proposed split
Only ~37 lines over — split the D-Bus interface/statics from the probe logic:

- `single_instance/mod.rs` (~75 lines) — constants (`BUS_NAME`,
  `OBJECT_PATH`, `IFACE_NAME`), statics (`CONN`, `MAIN_WEAK`,
  `PENDING_PRESENT`), `present_or_defer`, `bind_window`, `acquire_or_raise`,
  `pub use` of `iface` and `probe` submodules. Keep `#![cfg(target_os =
  "linux")]` at the top of `mod.rs` (applies to the whole module tree).
- `single_instance/iface.rs` (~25 lines) — `SingleInstanceIface` struct +
  its `#[zbus::interface(...)]` impl (`present`, `open_url`).
- `single_instance/probe.rs` (~70 lines) — `probe()` (name acquisition +
  fallback-to-existing-primary logic).

## Re-export surface
`single_instance/mod.rs` stays the `mod single_instance;` target (behind
`#[cfg(target_os = "linux")]` at the call site in `main.rs`). Only
`bind_window` and `acquire_or_raise` are called externally — both stay in
`mod.rs`, unaffected by the split. `probe()` is private, called only from
`acquire_or_raise` via `use probe::probe;` (or keep it `fn probe()` calling
`crate::single_instance::probe::probe()` — either works since it's private).

## Coupling / watch out
- `present_or_defer` (mod.rs) is called from BOTH `iface.rs`'s
  `SingleInstanceIface::present`/`open_url` methods AND indirectly relied on
  by nothing else — keep it `pub(super)` so `iface.rs` can call
  `super::present_or_defer()`.
- `probe()` registers the object server (`conn.object_server().at(...,
  SingleInstanceIface)`) BEFORE requesting the name — this ordering is
  explicitly commented as required (object must be callable the instant
  another launch sees the name taken); preserve exactly when split across
  files, don't reorder the two calls even though they're now in different
  modules conceptually.
- `zbus::blocking` API used deliberately (not async/tokio) — a comment
  explains the "tokio" feature is graph-wide forbidden; don't accidentally
  introduce async here during the split.
- `crate::deep_link::stash`/`take_pending`/`drain_pending` calls happen in
  both `iface.rs` (open_url) and `probe.rs` (probe) — no shared state beyond
  the `deep_link` module itself, no extra care needed beyond correct imports.

## Verify after split
- `cargo build -p qbz --target x86_64-unknown-linux-gnu` (Linux-only file;
  ensure the cfg gate still compiles on Linux CI).
- Manually test: launch the app, launch it again — second launch should
  raise the first window and exit; test with a `qobuz://` deep-link URL as
  the second launch's argv too.
