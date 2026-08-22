# crates/qbzd/src/paths.rs (172 lines)

## Summary
Daemon profile-root resolution: `ProfileRoots` (config/data/cache paths,
fully separate from the desktop profile) with XDG-aware defaults and
override support, plus 0700-permission enforcement on the config dir.

## Proposed split
This file is only marginally over budget (172 lines, ~80 of which are
tests) — a light two-way split suffices, no directory needed:

- `paths.rs` (~90 lines) — KEEP as the main file: module doc (lines 1-5),
  `ProfileRoots` struct + `resolve()` (lines 8-53), and the three
  `default_*_dir()` helpers (lines 55-71). This is the small, cohesive
  public API.
- `paths/permissions.rs` (~20 lines) — the `ensure_config_dir` pair
  (`#[cfg(unix)]`/`#[cfg(not(unix))]`, lines 73-91), which is a distinct,
  platform-conditional concern (filesystem permission enforcement) from
  the pure path-resolution logic above it.
- Tests (lines 93-172) stay in `paths.rs`'s own `#[cfg(test)] mod tests`
  block, or move to a `paths/tests.rs` if the split above still leaves
  `paths.rs` over 130 with tests included — recount after moving
  `ensure_config_dir` out; likely `paths.rs` lands ~90 (code) + tests
  needing to move too. Given `resolve()` covers config_override/
  data_root_override/defaults all in one method, either: (a) keep tests
  inline in `paths.rs` and accept it around 170 lines total (still over
  budget), or (b) split tests into `paths/tests.rs` as `#[path]`-included
  `#[cfg(test)] mod tests;` — recommended, since the three existing tests
  are already self-contained scratch-dir-based tests with no dependency on
  `ensure_config_dir` directly (they check the resulting directory's mode
  via `std::fs::metadata`, not by calling the permission function
  directly), so they can live in either file; putting them in their own
  file is cleanest.

## Re-export surface
`paths.rs` stays the only import surface —
`crate::paths::ProfileRoots::resolve(...)` is unaffected either way,
whether `ensure_config_dir`/tests move to sibling files via `mod
permissions; mod tests;` declarations at the top of `paths.rs`, or stay
inline. No directory/`mod.rs` restructuring needed since `paths.rs` itself
can remain a single file with `#[path = "paths/tests.rs"] mod tests;`
style includes if going that route, or simply becomes a `paths/mod.rs` if
preferred for consistency with other crates' directory-style splits in
this batch.

## Coupling / watch out
- `ProfileRoots::resolve` calls `ensure_config_dir(&config)` inline (line
  49) — a straightforward cross-file call once `ensure_config_dir` is
  `pub(crate)`/`pub(super)` visible from `permissions.rs`; it's currently
  a bare private `fn`, so needs at minimum `pub(super)`.
- The three `default_*_dir()` helpers all read `dirs::{config,data,cache}
  _dir()` — small, independent, no shared state; safe to leave together in
  the main `paths.rs`.
- The `data_root_override` special case (cache becomes
  `<data_root>/cache`, NEVER a sibling of data_root) is explained by a
  comment worth preserving verbatim — it documents a specific anti-pattern
  (walking outside the container) that a future editor should not
  reintroduce.
- Test `defaults_resolve_under_xdg_roots_without_touching_real_home`
  mutates process env vars (`XDG_CONFIG_HOME` etc.) and restores them
  before returning — flagged in its own comment as relying on
  single-threaded test execution; if tests move to a separate file, this
  safety caveat comment must move with it verbatim (it is the reason this
  test cannot safely run under certain parallel test harnesses).

## Verify after split
- `cargo build -p qbzd`.
- `cargo test -p qbzd paths::` — all 3 existing tests
  (`config_override_uses_parent_dir_and_creates_it_0700`,
  `data_root_override_places_cache_under_it_not_beside_it`,
  `defaults_resolve_under_xdg_roots_without_touching_real_home`) must still
  pass, INCLUDING on a machine where they run in parallel with other tests
  that also touch env vars (watch for flakiness introduced by the split,
  not caused by it).
- `cargo clippy -p qbzd`.
- Smoke-test importers: `grep -rn "paths::ProfileRoots" crates/qbzd/src` —
  confirm `main.rs`, `login.rs` (`ProfileRoots` param), and config-loading
  code still compile.
