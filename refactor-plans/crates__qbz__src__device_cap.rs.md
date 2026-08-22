# crates/qbz/src/device_cap.rs (197 lines)

Local output-device quality cap: cached DAC rate ceiling -> Qobuz tier
mapping, refreshed on explicit triggers (startup/settings/device change).

## Proposed split

File is only marginally over budget (197 lines, ~24 of which are tests).
Prefer a light split over a deep one:

- `device_cap/mod.rs` (~120 lines) — `CapState`, `CAP` static,
  `tier_for_max_rate_hz`, `cap`, `summary`, `tier_display`,
  `rate_khz_label`, `refresh`, `default_output_node`. This is already the
  full public surface; keep it together since it's one cohesive
  "cache + refresh" unit.
- `device_cap/tests.rs` (~25 lines) — move the `#[cfg(test)] mod tests`
  block out to its own file, included via `#[path = "tests.rs"] mod tests;`
  at the bottom of `mod.rs`. This alone likely drops `mod.rs` under 130.

## Tricky coupling

- None significant — `CAP` is a single `RwLock<Option<CapState>>`, no
  cross-module sharing needed beyond what's already in this file.

## Verify after split

`cargo build -p qbz`, `cargo test -p qbz device_cap::`.
