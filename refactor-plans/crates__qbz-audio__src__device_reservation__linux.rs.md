# crates/qbz-audio/src/device_reservation/linux.rs (704 lines)

## Summary
Linux `DeviceReservation` guard: a D-Bus client for the
`org.freedesktop.ReserveDevice1` protocol that claims exclusive ownership of
an ALSA card (parsing `hw:`/`plughw:`/`CARD=` device strings, resolving
contention against a higher/lower-priority holder, releasing on `Drop`).

## Proposed split

- `linux/mod.rs` (~15 lines) — `mod` declarations + `pub use` of
  `DeviceReservation`, `ReservationError`, and the `pub(crate)` constants
  (`QBZ_PRIORITY`, `QBZ_APPLICATION_NAME`) so `device_reservation::linux::*`
  is unchanged for callers.
- `linux/reservation.rs` (~160 lines) — the `DeviceReservation` struct,
  `ReservationState` enum, `impl DeviceReservation` (`acquire`, `is_active`),
  `impl Drop for DeviceReservation`. This is the public type + its core
  lifecycle; keep `acquire`'s large doc comment (the "tight coupling rule")
  attached since it's a load-bearing safety warning about USB DAC hardware
  damage risk.
- `linux/error.rs` (~30 lines) — `ReservationError` enum + `Display` + `Error`
  impls.
- `linux/dbus_ops.rs` (~100 lines) — the low-level D-Bus call wrappers:
  `try_acquire_name`, `release_name`, `open_holder_proxy`, `read_priority`,
  `read_application_name`, `request_release`. Pure D-Bus plumbing, no
  business logic.
- `linux/contention.rs` (~90 lines) — `resolve_contention` (the
  priority-comparison / RequestRelease / retry algorithm) — the one function
  with real decision logic, worth isolating from the plumbing in
  `dbus_ops.rs`.
- `linux/device_name.rs` (~120 lines) — `parse_card_index`,
  `resolve_card_index_by_name`, `bus_name_for_card`, `object_path_for_card`
  (the ALSA device-string parsing + naming helpers, already largely
  self-contained with their own doc comments explaining the CARD= id-vs-name
  distinction).
- `linux/tests.rs` (~135 lines) — the entire `#[cfg(test)] mod tests` block.
  Still borderline over 130; if so, split into `tests/parse_card_index.rs`
  and `tests/naming.rs` (bus_name/object_path/degraded_guard tests).

## Re-export surface
`linux/mod.rs` re-exports `DeviceReservation` and `ReservationError` publicly
(matching current visibility) and keeps `parse_card_index`,
`bus_name_for_card`, `object_path_for_card`, `QBZ_PRIORITY` at
`pub(crate)` visibility reachable the same way — check the parent
`device_reservation/mod.rs` for exactly what it currently imports from
`linux` (likely `cfg(target_os = "linux")`-gated `pub use linux::*` or
similar) and preserve that.

## Tricky coupling / watch out
- `DeviceReservation` and `resolve_contention` in `contention.rs` both
  construct `ReservationState::Active { connection, bus_name, app_device_name
  }` — the private `ReservationState` enum must be visible to both files
  (`pub(super)` or keep it in `reservation.rs` and have `contention.rs`
  reference `super::reservation::ReservationState`).
- `resolve_contention`'s success path constructs a full `DeviceReservation`
  directly (bypassing `acquire`) — this coupling between `contention.rs` and
  `reservation.rs`'s private fields must survive the split (likely needs
  `pub(super)` on `DeviceReservation`'s single field or a `pub(super)`
  constructor).
- The `#[allow(dead_code)]` on `QBZ_APPLICATION_NAME` and on `Active`'s
  `app_device_name` field are deliberate (deferred Task 5 wiring per the doc
  comment) — don't "clean up" what looks like dead code during the split.
  The tests reference `ReservationState::Degraded` directly, so it must stay
  visible to `tests.rs`.

## What to verify after the real split
- `cargo test -p qbz-audio device_reservation::linux` — all tests
  (`parse_card_index_*`, `bus_name_format`, `object_path_format`,
  `degraded_guard_reports_inactive`) green.
- `cargo build -p qbz-audio --target x86_64-unknown-linux-gnu` (this file is
  Linux-only; verify it still compiles under the `cfg(target_os = "linux")`
  gate in the parent module).
- Grep for `device_reservation::linux::` / `DeviceReservation::acquire` call
  sites (expected in `qbz-audio`'s ALSA-direct stream code, per the doc's
  "Lifetime A" reference) to confirm the public path is unchanged.
- No live-hardware regression test is feasible in CI, but if a dev has a USB
  DAC available, a manual exclusive-mode toggle is the closest thing to a
  smoke test given the doc's hardware-damage warning.
