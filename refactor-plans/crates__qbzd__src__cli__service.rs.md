# `crates/qbzd/src/cli/service.rs` (335 lines)

`qbzd service [systemd|openrc|runit]` — pure-local generator that prints an init-system
service unit (systemd user/system, OpenRC, runit) with correct audio env (XDG_RUNTIME_DIR,
HOME) for the target user.

## Proposed split

- `service.rs` (~50 lines) — re-export surface + top-level `pub fn service()` dispatcher
  (arg handling, calls into the other modules, prints to stdout/stderr).
- `service/target.rs` (~60 lines) — `Target` struct, `resolve()`, `passwd()`, `id_group()`,
  `run()`, `detect_init()`. Pure discovery/shell-out logic.
- `service/templates.rs` (~120 lines) — `systemd_user`, `systemd_system`, `openrc`, `runit`
  template functions (the big format! strings).
- `service/hints.rs` (~40 lines) — `systemd_user_hint`, `systemd_system_hint`,
  `openrc_hint`, `runit_hint` (install instructions printed to stderr).
- Tests (`t()` fixture + template assertions) move alongside `templates.rs`; the
  `host_discovery`-style tests (none here, but `detect_init`/`passwd` tests if added) go
  with `target.rs`.

## Coupling to flag

- Nothing calls into this module except the CLI arg dispatcher (`qbzd service` subcommand)
  — low risk, no shared state. Straightforward mechanical split.

## Verify after split

- `cargo test -p qbzd` (template content assertions still green).
- Manually run `qbzd service systemd` / `openrc` / `runit` and diff output against current
  behavior (byte-for-byte, since these are user-facing generated files).
