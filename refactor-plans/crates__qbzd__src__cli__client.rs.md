# `crates/qbzd/src/cli/client.rs` (296 lines)

The stateless HTTP client behind every networked CLI verb: target/token discovery,
`ApiClient` (get/post), transport error classification, and the frozen `CliError` exit-code
taxonomy.

## Proposed split

- `client.rs` (~60 lines) — re-export surface: `pub use` of `CliError`, `Target`,
  `resolve_host`, `ApiClient`. Keeps `crate::cli::client::ApiClient` etc. importable
  unchanged.
- `client/error.rs` (~90 lines) — `CliError` enum, `exit_code()`, `Display` impl,
  `error_from_envelope`. This is the frozen exit-code taxonomy (02 §1.3) — must not
  change behavior.
- `client/target.rs` (~40 lines) — `Target` struct, `resolve_host`, `resolve_token`,
  `normalize_hostport`. Pure discovery logic (§1.5), no I/O beyond env/config read.
- `client/api_client.rs` (~110 lines) — `ApiClient` struct + impl (`new`, `get`, `post`,
  `bearer`, `send`, `classify_transport`, `diagnose_skew`, `info_api_version`).
- Split `#[cfg(test)]` tests to follow their module (error tests → `error.rs` tests,
  host discovery tests → `target.rs` tests).

## Coupling to flag

- `error_from_envelope` calls `crate::cli::copy::*` for DSD-specific verbatim messages —
  keep that dependency visible/documented in `error.rs`.
- `ApiClient::new` takes `&ProfileRoots` and reads `QbzdConfig` — coupling between
  `target.rs` (token resolution) and `crate::config`/`crate::paths` stays as-is.
- This is the P0 transport used by every CLI verb — a re-export miss breaks the whole CLI,
  so keep `client.rs` a pure `pub use` façade.

## Verify after split

- `cargo test -p qbzd` — all client/error/target tests green.
- `cargo build -p qbzd` and smoke-test one CLI verb (`qbzd status`) against a running daemon.
