# crates/qbzd/src/cli/status.rs (310 lines)

## Summary
The `qbzd status` and `qbzd ping` CLI verbs: fetch a JSON payload from the
daemon's `/api/status`/`/api/ping` HTTP endpoints, run the version-skew and
exit-code logic (§1.3/§1.6 of the daemon spec), and render a human-readable
composite status block (or raw JSON with `--json`).

## Proposed split
Clean pure (render/parse) vs. IO (the two async command entry points) split,
further divided by render sub-section since `render`/`render_audio`/
`render_playback`/etc. are all small pure string-formatting functions:

- `cli/status/mod.rs` (~75 lines) — module doc, `ping` and `status` async
  functions (the IO entry points, lines 15-73) — these are the only functions
  that touch the network (`ApiClient`) and stdout/stderr.
- `cli/status/exit_code.rs` (~20 lines) — `exit_from_state` (the §1.3 exit
  code table logic).
- `cli/status/render.rs` (~100 lines) — `render` (the composite block
  assembler), `render_auth`, `render_audio`, `render_playback`,
  `render_last_error` — all pure `&Value -> String` formatters, the biggest
  cohesive pure-logic chunk in the file.
- `cli/status/fmt.rs` (~25 lines) — `str_at`, `fmt_mmss`, `fmt_uptime` — tiny
  generic formatting helpers shared by `render.rs`.
- `cli/status/linger.rs` (~15 lines) — `linger_warning` (the `loginctl`
  subprocess check, §1.4) — isolated because it's the one function that
  shells out, distinct from the pure JSON-Value formatters around it.
- `cli/status/tests.rs` (~65 lines) — the `#[cfg(test)] mod tests` block
  (fixture `logged_in_payload` + the 5 tests), declared via `#[cfg(test)] mod
  tests;` in `mod.rs`.

## Re-export surface
`cli/status/mod.rs` keeps `pub async fn ping` and `pub async fn status` as
the module's only public API — the CLI dispatch table in `cli/mod.rs` (or
wherever the `qbzd ping`/`qbzd status` subcommands are wired) continues to
call `crate::cli::status::{ping, status}` unchanged; `render`,
`exit_from_state`, `str_at`, `fmt_mmss`, `fmt_uptime`, `linger_warning` all
stay module-private (`pub(super)` or unqualified `fn`, matching their current
non-`pub` visibility) so nothing outside this module can see the split at
all.

## Coupling / watch out
- `status()` (mod.rs) calls `render(&payload, client.host())`,
  `exit_from_state(&payload)`, and `linger_warning()` — three different new
  files — so `mod.rs` needs `use render::render; use exit_code::exit_from_state;
  use linger::linger_warning;` (or declare the submodules `mod render; mod
  exit_code; mod linger; mod fmt;` and call them qualified, e.g.
  `render::render(...)`). Either style works; pick one and use it
  consistently across all 6 new files.
- `render()` (render.rs) calls all four `render_*` sub-formatters AND
  `fmt_uptime`/`str_at` from `fmt.rs` — needs `use super::fmt::{str_at,
  fmt_uptime}` (and `fmt_mmss` is used by `render_playback`, also in
  render.rs, from the same `fmt.rs`).
- The version-skew check currently lives inline in `status()` (lines 45-57,
  reading `payload.get("api_version")`/`payload.get("version")` and calling
  `crate::API_VERSION` + `copy::api_version_skew`/`copy::version_skew`) — this
  is NOT extracted to its own file above; it's short enough (~13 lines) to
  stay in `mod.rs`'s `status()` body. If the implementer prefers, it could
  become a `version_skew.rs` file, but that's optional given it's already
  small and tightly coupled to the `status()` control flow (early return on
  mismatch).
- `crate::cli::client::ApiClient`, `crate::cli::copy`, `crate::paths::ProfileRoots`
  are only used in `mod.rs` (the two async entry points) — no need to
  re-import them into the pure-formatter files.
- Tests construct a full `logged_in_payload()` JSON fixture and call
  `exit_from_state`, `render`, `render_playback` directly (not through
  `ping`/`status`) — `tests.rs` needs `use super::exit_code::exit_from_state;
  use super::render::{render, render_playback};` (or `use super::*;` if
  everything is re-exported at `mod.rs` level via `pub(super) use`).

## Verify after split
- `cargo test -p qbzd cli::status::` — all 5 tests
  (`healthy_status_exits_zero`, `needs_auth_exits_four`,
  `configured_but_absent_device_exits_five`,
  `render_covers_the_composite_block`, `stopped_playback_renders_queue_only`)
  green.
- `cargo check -p qbzd` and confirm the `qbzd status`/`qbzd ping` CLI
  subcommand dispatch (wherever `cli::status::{ping,status}` is called from)
  still compiles unchanged.
- Manual smoke-test: run `qbzd status` and `qbzd status --json` against a
  live (or mock) daemon to confirm the human-readable block and JSON output
  are byte-identical to before the split.
