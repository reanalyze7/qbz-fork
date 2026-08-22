# crates/qbzd/src/cli/settings.rs (1290 lines)

## Summary
Implements every `qbzd settings`/`config` CLI subcommand: the canonical
dotted-key table + per-key value parse/render, `show`/`set`, `config
path`/`config show`, and the T12 settings `export`/`import` bundle flow
(device re-pick, secret validation, reload-nudge, three-bucket summary) —
plus a large in-module `#[cfg(test)]` block.

## Proposed split
By responsibility: key table/classification, value codecs, store IO,
show/set/config verbs, export/import verbs, import helpers, tests.

- `cli/settings/mod.rs` (~40 lines) — module doc/header comment, `pub mod`
  declarations, `pub use` re-exports of `ApplyClass`, `write_one` (currently
  `pub(crate)`), `show`, `set`, `config_path`, `config_show`, `export`,
  `import` so `qbzd`'s CLI dispatcher keeps calling
  `cli::settings::{show,set,...}` unchanged.
- `cli/settings/keys.rs` (~90 lines) — `ApplyClass`, `KEY_TABLE`,
  `classify`, `unknown_key_error`.
- `cli/settings/codec.rs` (~155 lines) — every `parse_*`/`render_*` value
  function (`parse_bool`/`render_bool` through `parse_autoplay`/
  `render_autoplay`).
- `cli/settings/store.rs` (~35 lines) — `open_audio`, `open_playback`,
  `read_all`.
- `cli/settings/write.rs` (~200 lines) — `SetError` + impls, `write_one`
  (the long per-key match), `nudge`, `nudge_outcome`, `local_token`.
- `cli/settings/verbs.rs` (~120 lines) — `show`, `set`, `config_path`,
  `config_show`, `unit_path`, `present_keys`.
- `cli/settings/import_export.rs` (~130 lines) — `export`, plus the top
  half of `import` (read/parse bundle, `--remap` parsing, `ProfilePaths`
  build, initial `plan`).
- `cli/settings/import_flow.rs` (~130 lines) — the rest of `import` (device
  re-pick prompt loop, secret validation, dry-run/apply/reload-nudge steps)
  — likely `import` itself needs splitting into a small `import()` entry in
  `import_export.rs` that calls a private continuation function here; keep
  the function signature (`pub async fn import(...)`) in one place only.
- `cli/settings/reload.rs` (~40 lines) — `reload_disposition`,
  `build_live_system`, `prompt_device`.
- `cli/settings/summary.rs` (~80 lines) — `print_summary_header`,
  `print_buckets`.
- `cli/settings/tests.rs` (~240 lines) — the entire `#[cfg(test)] mod
  tests` block (`scratch_roots`, `cleanup`, all ~18 tests).

## Re-export surface
`cli/settings/mod.rs` re-exports everything currently `pub`/`pub(crate)` at
`crate::cli::settings::*` — the daemon's CLI arg dispatcher (`main.rs` or
`cli/mod.rs`) calls `cli::settings::show`, `::set`, `::config_path`,
`::config_show`, `::export`, `::import`; `write_one` is `pub(crate)` and
also reused directly by the T13 setup TUI per the file's own doc comment
("the CLI's own validated writer... the TUI persists every screen through
this SAME validated writer") — grep for `settings::write_one` and
`cli::settings::write_one` across `crates/qbzd/src/tui/` before finalizing
which module re-exports it.

## Coupling / watch out
- `KEY_TABLE` order is NORMATIVE (`show`/`set` list/accept keys in this
  exact order) — `keys.rs` must keep it verbatim; `codec.rs`'s
  `read_all`/`write_one` match arms must stay in sync with it (there are
  two `unreachable!("...drifted apart...")` guards enforcing this — keep
  both when splitting).
- `write_one` is reused by the T13 setup TUI (see module doc) — do not
  change its signature or make it more private during the split.
- `import`'s steps are heavily sequential/stateful (bundle parse → remap →
  plan → device re-pick replan → secret validation → dry-run short-circuit
  → apply → reload-nudge → summary print) with early returns at almost
  every step — if split across `import_export.rs`/`import_flow.rs`, thread
  every intermediate value (`bundle`, `opts`, `live`, `plan`, `auth_note`,
  `validated_uid`) through explicit parameters/return tuples rather than
  trying to share mutable local state across files.
- `nudge`/`nudge_outcome`/`local_token` depend on `crate::login::{nudge_host,
  nudge_reload, nudge_reload_outcome}` and `crate::config::QbzdConfig` —
  cross-module (not just cross-file) coupling to keep intact.
- Tests use `scratch_roots`/`cleanup` helpers referencing `ProfileRoots`
  from `crate::paths` — keep these test-only helpers together with the
  tests, not split further.

## Verify after split
- `cargo test -p qbzd cli::settings::` — all tests green (they use
  per-test scratch temp dirs, safe in parallel).
- `cargo check -p qbzd` (or full workspace) — confirms `cli::settings::*`
  import paths used by the CLI dispatcher and (if applicable) the T13 TUI
  still resolve.
- Manual smoke-test: `qbzd settings show`, `qbzd settings show --json`,
  `qbzd settings set audio.backend alsa`, `qbzd settings export` then
  `qbzd settings import --dry-run` on the produced bundle, `qbzd config
  show`.
