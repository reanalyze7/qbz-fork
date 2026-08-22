# crates/qbzd/src/cli/scrobble.rs (189 lines)

## Summary
The `qbzd scrobble …` CLI verbs: connect Last.fm (web-auth flow) or
ListenBrainz (pasted token), check status, and enable/disable scrobbling —
writing to the canonical `ScrobblerSettingsStore` and best-effort nudging a
running daemon to reload.

## Proposed split

- `scrobble/mod.rs` (~15 lines) — module doc, `mod` declarations, `pub use`
  re-exports of `login_lastfm`, `login_listenbrainz`, `status`,
  `set_enabled`.
- `scrobble/connect.rs` (~80 lines) — `login_lastfm`, `login_listenbrainz`
  (the two web-auth/token-exchange flows, both async, both following the
  same "print/read -> exchange -> store -> nudge" shape).
- `scrobble/status.rs` (~65 lines) — `status`, `set_enabled`, `provider_line`
  (the read/toggle side, plus the shared status-string formatter).
- `scrobble/support.rs` (~35 lines) — `open_store`, `not_connected`,
  `nudge_reload` — the small shared internals currently under the
  `// ==== internals ====` banner.

## Re-export surface
`scrobble/mod.rs` re-exports `login_lastfm`, `login_listenbrainz`, `status`,
`set_enabled` at `crate::cli::scrobble::*` — the CLI arg-parsing/dispatch code
(wherever `qbzd scrobble <subcommand>` routes to these) needs no changes.

## Tricky coupling / watch out
- `open_store` (in `support.rs`) is called from every function in
  `connect.rs` and `status.rs` — straightforward cross-file call once
  re-exported, just don't duplicate the `ScrobblerSettingsStore::new_at`
  error-mapping logic.
- `nudge_reload` (in `support.rs`) re-resolves `ProfileRoots` independently of
  the `roots` parameter each caller already has (it calls
  `crate::paths::ProfileRoots::resolve(None, None)` fresh rather than reusing
  the passed-in `roots`) — this looks like existing behavior worth flagging
  rather than "fixing" during a pure file-split; preserve it as-is unless the
  user separately asks to fix it.
- `qbz_log::register_secret(...)` calls (for the Last.fm session key and the
  ListenBrainz token) happen inline in `connect.rs` right after the
  credential is obtained, before it's stored — keep this ordering (register
  the secret for log redaction BEFORE any subsequent operation that might log
  it) when moving code.
- `set_enabled` (in `status.rs`) has a subtlety: enabling a provider also
  flips the master `store.set_enabled(true)` toggle, but disabling one does
  NOT flip the master toggle off — preserve this asymmetry exactly.

## What to verify after the real split
- `cargo build -p qbzd` (no `#[cfg(test)]` block in this file today).
- Grep for `cli::scrobble::` in `crates/qbzd/src/cli/` (the command
  dispatcher) to confirm the four entry points still resolve.
- Manual smoke test via the `run` skill or a terminal: `qbzd scrobble status`
  (with and without connected providers), `qbzd scrobble login listenbrainz
  --token <token>` against a test token if available, and `qbzd scrobble
  enable/disable lastfm` to confirm the master-toggle asymmetry above still
  holds.
