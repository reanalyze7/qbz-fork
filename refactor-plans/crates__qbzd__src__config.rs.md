# crates/qbzd/src/config.rs (172 lines)

## Summary
`qbzd.toml` config model — PROCESS concerns only (D14 single-source rule:
engine settings like audio/playback/qconnect content live in the stores,
never here): `QbzdConfig`/`ServerCfg`/`LogCfg`/`MprisCfg` structs with
serde defaults, a `KNOWN` key allowlist + `sweep()` unknown-key warning scan
(never a hard error, per J5 silent-revert guard), and `from_str`/`load`
loaders.

## Proposed split
Only 1.3x over budget, so a light 3-way split by responsibility — a
`config/` directory with a thin `mod.rs`.

- `config/mod.rs` (~75 lines) — the struct definitions (`QbzdConfig`,
  `ServerCfg`, `LogCfg`, `MprisCfg`) and their `Default` impls (with all the
  existing doc comments about D14, the LAN-first `bind: "0.0.0.0"` default
  FB6 rationale, and the opt-in `[server] token` semantics preserved
  verbatim — these comments encode product decisions, not incidental
  detail). Declares `mod known_keys; mod loader; #[cfg(test)] mod tests;`
  and re-exports nothing extra (the structs are defined here directly).
- `config/known_keys.rs` (~20 lines) — the `KNOWN: &[(&str, &str)]` const
  and the `sweep(v, table, warns)` recursive unknown-key scanner.
- `config/loader.rs` (~20 lines) — `impl QbzdConfig { pub fn from_str(...); pub fn load(...) }`,
  using `known_keys::sweep`.
- `config/tests.rs` (~55 lines) — the existing `#[cfg(test)] mod tests`
  block (`defaults_match_spec`, `unknown_keys_warn_never_error`,
  `server_token_defaults_none_and_parses_when_set`,
  `server_token_empty_string_parses_as_present_but_filtering_gates_it`),
  included via `#[cfg(test)] mod tests;` from `mod.rs`.

## Re-export surface
`config/mod.rs` is the public-API surface: `pub struct QbzdConfig`,
`pub struct ServerCfg`, `pub struct LogCfg`, `pub struct MprisCfg` are
defined there directly, so `crate::config::QbzdConfig` (and the nested
`ServerCfg`/`LogCfg`/`MprisCfg`) keep their exact current paths — only the
file becomes a directory (`config.rs` → `config/mod.rs`), transparent to
`mod config;` in `main.rs`/`lib.rs`.

## Coupling / watch out
- `QbzdConfig::from_str`/`load` (in `loader.rs`) call `sweep(&value, "", &mut warns)`
  (in `known_keys.rs`) before deserializing via `value.try_into()` — keep
  the exact ordering (sweep first, on the raw `toml::Value`, before the
  typed deserialize) since that's what lets unknown keys warn rather than
  silently vanish during `try_into`.
- `sweep`'s recursion (`toml::Value::Table(_) if table.is_empty() => sweep(inner, k, warns)`)
  only descends one level (assumes a flat `[section]` structure, not nested
  tables-of-tables) — this is an existing constraint, not something to "fix"
  during the split; just carry the exact match arms over.
- The `KNOWN` const is the single source of truth for every valid
  (table, key) pair across `ServerCfg`/`LogCfg`/`MprisCfg`/root-level fields
  — when `mod.rs`'s structs change in the future, `known_keys.rs`'s `KNOWN`
  list must be updated in lockstep (already true today, just now the two
  live in different files so it's easier to forget).
- Tests in `tests.rs` reference `QbzdConfig::from_str` and read struct fields
  (`c.server.bind`, `c.log.level`, `c.mpris.enabled`, `c.server.token`)
  directly — needs `use super::*;` to pull in both the structs (from
  `mod.rs`) and `from_str` (from `loader.rs`, re-exported via `mod.rs`'s
  glob if `pub use loader::*;` is added, or just `use super::loader::*` plus
  `use super::*` for the structs — pick one pattern and keep it consistent).

## Verify after split
- `cargo build -p qbzd`
- `cargo test -p qbzd config` — all four existing unit tests green,
  unchanged assertions (defaults, unknown-key warning, token opt-in
  presence/absence, empty/whitespace token parsing-but-not-erroring).
- Grep for `QbzdConfig`/`ServerCfg`/`LogCfg`/`MprisCfg` usage across `qbzd`
  (daemon startup, `qbzd config show --json` per the `Serialize` derive
  comment) to confirm no import path broke.
- Manual smoke test: `qbzd config show` (or equivalent CLI verb) against a
  config file with an unrecognized key, confirming it still warns (not
  errors) exactly as the `unknown_keys_warn_never_error` test expects.
