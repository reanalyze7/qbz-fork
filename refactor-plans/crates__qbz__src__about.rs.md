# crates/qbz/src/about.rs (204 lines)

## Summary
About-modal controller: seeds the static `AboutState` fields (version,
platform label, build date/commit, release URL, contributor list) once at
shell setup, wires the `open-url` callback, and asynchronously fetches +
paints GitHub avatars (author + contributors) onto their chips off the UI
thread.

## Proposed split
By responsibility (static text/version helpers vs contributor-data building
vs the async avatar-fetch pipeline):

- `about/mod.rs` (~55 lines) — lines 1-49 + the `install()` fn (99-127):
  module doc, `app_version()`, `build_date()`, `build_commit()`,
  `platform_label()` (the small static string helpers), and `install()`
  itself (the public entry point — seeds state, wires callback, dispatches
  avatar loads).
- `about/contributors.rs` (~50 lines) — lines 51-97: `AUTHOR_HANDLE`,
  `CONTRIBUTORS`, `CONTRIBUTORS_PER_ROW` consts + `build_contributor_groups()`
  — the static contributor-list data and its row-grouping logic.
- `about/avatars.rs` (~75 lines) — lines 129-204: `avatar_url()`,
  `spawn_avatar_loads()`, `fetch_avatar()` — the async GitHub-avatar
  fetch-and-paint pipeline (the file's only genuinely async/networked part).

## Re-export surface
`about/mod.rs` stays the `mod about;` target — the only external call is
`about::install(window, handle)` from shell setup, plus `about::app_version()`
which the diagnostics panel also calls (per the file's own doc comment: "The
diagnostics panel reads the same source"). Both must stay reachable at
`crate::about::install` / `crate::about::app_version` via `pub use
avatars::*;` (not needed, avatars has no pub items) and keeping `install`/
`app_version` defined directly in `mod.rs`.

## Coupling / watch out
- `AUTHOR_HANDLE` and `CONTRIBUTORS` (in `contributors.rs`) are used by BOTH
  `build_contributor_groups()` (same file) AND `spawn_avatar_loads()` (in
  `avatars.rs`, for the author avatar fetch + the `idx / CONTRIBUTORS_PER_ROW`
  / `idx % CONTRIBUTORS_PER_ROW` group/position addressing that must exactly
  match the grouping `build_contributor_groups()` produced) — these two
  files MUST stay in sync on `CONTRIBUTORS_PER_ROW`; make sure
  `avatars.rs` imports the consts from `contributors.rs` rather than
  hardcoding `5` again.
- `crate::artwork::pixels_to_image` is called from within the
  `upgrade_in_event_loop` closures in `spawn_avatar_loads` — an external-crate
  dependency, unaffected by this internal split, just keep the `use` in
  `avatars.rs`.
- The comment on `app_version()` ("the REAL release version, not the 0.1.0
  workspace pins for library crates... diagnostics panel reads the same
  source") is a cross-file invariant with `crates/qbz-app/src/diagnostics.rs`
  (see its own refactor plan in this batch, agent's `diagnostics/runtime.rs`)
  — do not let this doc comment get lost or the two file's owners will not
  realize the coupling.
- `AboutContributorGroup`/`AboutContributorRow`/`AboutState`/`AboutActions`/
  `AppWindow` (Slint-generated types, imported from `crate::` at the top)
  are used across `install()` and `contributors.rs`/`avatars.rs` alike —
  each new file needs its own subset of that `use` line.

## Verify after split
- `cargo check -p qbz` and `cargo build -p qbz`.
- Open the About modal in the running app, confirm version/build-date/commit/
  platform label render correctly, and that avatars (author + all
  contributor chips) still populate asynchronously without errors in logs.
