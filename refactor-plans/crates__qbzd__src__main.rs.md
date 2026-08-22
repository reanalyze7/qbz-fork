# crates/qbzd/src/main.rs (550 lines)

## Summary
`qbzd` binary entry point: the entire `clap` CLI grammar (top-level `Cli` +
every subcommand enum: `Cmd`, `QueueCmd`, `RecoCmd`, `FavCmd`, `PlaylistCmd`,
`SettingsCmd`, `ScrobbleCmd`, `ScrobbleLoginCmd`, `ConfigCmd`) plus the
`async fn main()` dispatcher that matches every `Cmd` variant to its handler
in the `cli::*` submodules, and a `login_roots()` helper.

## Proposed split
By responsibility — CLI grammar (pure data/derive, no logic) vs dispatch
(the big match) vs small helpers:

- `main.rs` (~20 lines) — `mod` declarations (already present, lines 3-14),
  `pub const API_VERSION`, and a slim `#[tokio::main] async fn main()` that
  just does `let cli = Cli::parse(); std::process::exit(dispatch(cli).await);`
- `cli_args.rs` (~230 lines, still over budget — split further) — the entire
  `#[derive(Parser)] struct Cli` + `#[derive(Subcommand)] enum Cmd` (lines
  18-146). This alone is ~130 lines; split into:
  - `cli_args/root.rs` (~35 lines) — `Cli` struct + `API_VERSION`-adjacent
    doc.
  - `cli_args/cmd.rs` (~130 lines) — the big `enum Cmd` (all top-level
    subcommands).
- `cli_args/sub.rs` (~100 lines) — the smaller nested subcommand enums:
  `QueueCmd`, `RecoCmd`, `FavCmd`, `PlaylistCmd` (lines 148-203).
- `cli_args/settings_sub.rs` (~40 lines) — `SettingsCmd` (lines 205-221).
- `cli_args/scrobble_sub.rs` (~40 lines) — `ScrobbleCmd`, `ScrobbleLoginCmd`,
  `ConfigCmd` (lines 223-246).
- `dispatch.rs` (~260 lines, still over — split further) — the big
  `match cli.cmd { ... }` body currently inline in `main()` (lines 251-534),
  as a `pub async fn dispatch(cli: Cli) -> i32`. Given its size, split by
  functional group instead of one flat file:
  - `dispatch/misc.rs` (~40 lines) — `Version`, `Service`, `Completions`.
  - `dispatch/auth.rs` (~60 lines) — `Run`, `Login`, `Logout` (the
    config/phase-1/phase-2 bootstrap logic currently inline, lines 268-332).
  - `dispatch/status.rs` (~20 lines) — `Status`, `Ping`, `Now`, `Watch`.
  - `dispatch/browse.rs` (~60 lines) — `Search`, `Album`, `Artist`,
    `Similar`, `Suggest`, `Discover`, `Reco`.
  - `dispatch/library.rs` (~90 lines) — `Fav`, `Playlist` (both have nested
    match arms, this is the biggest chunk after auth).
  - `dispatch/transport.rs` (~70 lines) — `Shuffle`, `Repeat`, `Art`,
    `Resolve`, `Play`, `Pause`, `Toggle`, `Stop`, `Next`, `Prev`, `Seek`,
    `Volume`, `Mute`, `Queue`.
  - `dispatch/config.rs` (~60 lines) — `Settings`, `Scrobble`, `Config`,
    `Setup`.
- `roots.rs` (~15 lines) — `login_roots()` helper (lines 542-549), used by
  several dispatch groups.

## Re-export surface
`main.rs` is the binary entry point — it is not a library, so there is no
"importer" concern in the usual sense, but `dispatch::dispatch` must be
`pub(crate)` (or `pub`) and imported into `main.rs`; and every `cli_args::*`
type (`Cli`, `Cmd`, `QueueCmd`, etc.) must be re-exported from a single
`cli_args/mod.rs` with `pub use root::*; pub use cmd::*; pub use sub::*; ...`
so `main.rs` can keep writing `Cli::parse()` and matching on `Cmd::Variant`
without `cli_args::cmd::Cmd::Variant` qualification everywhere.

## Coupling / watch out
- Every dispatch arm builds `let roots = paths::ProfileRoots::resolve(None,
  None);` (or calls `login_roots()`) then calls into the existing
  `crate::cli::*` submodules (`cli::status`, `cli::transport`, `cli::search`,
  `cli::browse`, `cli::discover`, `cli::reco`, `cli::fav`, `cli::playlist`,
  `cli::mode`, `cli::art`, `cli::resolve`, `cli::play`, `cli::queue`,
  `cli::settings`, `cli::scrobble`, `cli::service`, `cli::copy`) — these are
  UNCHANGED by this split (they're separate files already), just make sure
  each new `dispatch/*.rs` has the right `use crate::cli::...` and `use
  crate::{paths, config, login, tui}` lines for what it touches.
- `cli.host` and `cli.quiet` (global clap args on `Cli`) are read inside
  several dispatch arms (e.g. `cli.host` passed everywhere) — if `dispatch`
  becomes a free function taking `cli: Cli` by value, this is fine, but if
  split by functional group each `dispatch::<group>::handle(...)` fn needs
  `cli.host` (and maybe `cli.quiet`) passed in explicitly since `cli` itself
  won't be split.
- `Cmd::Run`'s phase-1/phase-2 config bootstrap (reading `qbzd.toml` before
  knowing `data_root`) is delicate two-step logic — keep it as one
  contiguous block in `dispatch/auth.rs`, do not split further.
- The `Cmd::Setup` arm calls `tui::run(roots).await` — unaffected here,
  but note `tui/strings.rs` is ALSO being split by another agent in this
  same effort; no direct file overlap, just the same `tui` module tree.

## Verify after split
- `cargo check -p qbzd` and `cargo build -p qbzd`.
- `cargo test -p qbzd` if any dispatch-adjacent tests exist (none observed
  in this file itself).
- Smoke-test the CLI directly: `qbzd --help`, `qbzd version --json`,
  `qbzd status`, and one nested-subcommand path like `qbzd playlist list`
  or `qbzd queue add 123` against a running daemon, to confirm clap's
  derived parsing and the dispatch match still route identically after the
  module split (clap derives are easy to silently break by moving a struct
  without its field doc-comments, which double as `--help` text).
