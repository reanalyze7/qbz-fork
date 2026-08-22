# crates/qbz-app/src/user_data.rs (202 lines)

## 1. Summary

`UserDataPaths` — a central provider of per-user data/cache directory
paths (`~/.local/share/qbz/users/{uid}/`, `~/.cache/qbz/users/{uid}/`),
plus global (non-user-scoped) path helpers and a small "last active
user id" persistence mechanism (a flat file) used to restore sessions
across launches; includes a test suite.

## 2. Proposed module split

Given this file is only 202 lines (~1.55x the limit), a light 2-way
split plus tests is proportionate — no need for a deep multi-file
breakdown.

| New file | Owns | ~lines |
|---|---|---|
| `user_data/mod.rs` | `UserDataPaths` struct + `impl Default`; module doc; re-exports | ~40 |
| `user_data/scoped_paths.rs` | The per-user and global path methods: `new`, `set_user`, `clear_user`, `current_user_id`, `user_data_dir`, `user_cache_dir`, `data_dir_for`, `cache_dir_for`, `global_data_dir`, `global_cache_dir` (all as `impl UserDataPaths` in this file) | ~90 |
| `user_data/last_user.rs` | `save_last_user_id`, `load_last_user_id`, `clear_last_user_id`, `last_user_id_path` (also `impl UserDataPaths` methods, split out because they're a distinct concern: session-restore persistence vs. path resolution) | ~35 |
| `user_data/tests.rs` | The entire `#[cfg(test)] mod tests` block | ~45 |

## 3. Re-export / public API surface

`user_data/mod.rs`:

```rust
mod last_user;
mod scoped_paths;
#[cfg(test)]
mod tests;

pub struct UserDataPaths {
    user_id: std::sync::RwLock<Option<u64>>,
}

impl Default for UserDataPaths {
    fn default() -> Self {
        Self::new()
    }
}
```

As with `metadata.rs`, the actual methods live in `impl UserDataPaths`
blocks spread across `scoped_paths.rs` and `last_user.rs` — Rust allows
this within one crate. The struct itself and its one field stay in
`mod.rs` since `impl Default` needs `Self::new()` (defined in
`scoped_paths.rs`) — that's fine, inherent methods resolve across files.
Every caller doing `qbz_app::user_data::UserDataPaths::new()` /
`.user_data_dir()` / `UserDataPaths::load_last_user_id()` etc. keeps
working unchanged — no path changes.

## 4. Tricky coupling/shared state to watch out for

- `UserDataPaths::new()` (constructor, in `scoped_paths.rs`) is called
  by the `Default` impl in `mod.rs` — needs `Self::new()` which resolves
  fine as an inherent method, but confirm the derive/impl placement
  compiles cleanly (it will, but worth flagging since it's the one
  cross-file dependency in an otherwise embarrassingly-parallel split).
- `data_dir_for`/`cache_dir_for` (arbitrary-user-id variants, used by the
  "#553 guest-profile adoption" feature per their doc comments) must NOT
  be confused with `user_data_dir`/`user_cache_dir` (active-user
  variants) — keep both pairs' doc comments explaining the distinction
  intact, ideally adjacent in `scoped_paths.rs` since they're the same
  logical operation with/without an active-user requirement.
- `last_user_id_path()` (private today) is called only by
  `save_last_user_id`/`load_last_user_id`/`clear_last_user_id`, all
  moving together to `last_user.rs` — no cross-file split needed within
  that trio, keep them together.
- Every path method returns `Result<PathBuf, String>` with hand-rolled
  error strings (not a shared error enum) — this is pre-existing style,
  not something to "fix" during the split; preserve wording exactly so
  any UI that surfaces these strings to users doesn't change text.

## 5. What to verify after the real split

- `cargo build -p qbz-app` and `cargo test -p qbz-app user_data::` — all
  4 tests green (`starts_without_active_user`,
  `set_and_clear_user_updates_current_user`,
  `user_dirs_are_scoped_by_user_id`, `global_dirs_are_scoped_to_qbz`).
- Grep the workspace for `UserDataPaths::` usages (likely
  `qbz`/`qbz-app`'s login/logout flow, library.db/cache path wiring, and
  the guest-profile-adoption feature referenced in `data_dir_for`'s doc
  comment) to confirm all call sites still resolve.
- Smoke-test: log in as a user, confirm library/cache data lands under
  `.../users/{uid}/`; log out and back in, confirm "last active user"
  restore still works (the flat-file mechanism in `last_user.rs`) if
  remember-me is enabled.
