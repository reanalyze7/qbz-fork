# crates/qbz-media-controls/src/notify.rs (317 lines)

## Summary
Frontend-agnostic desktop "now playing" track-change notifications: formats
title/body/quality text, caches+resizes album art, and fires an XDG portal
notification (Linux) or `notify_rust` (macOS); a no-op stub elsewhere.

## Proposed split
By responsibility, matching the file's own `// --- section --- ` markers —
this is a natural pure(format)/IO(cache+network)/platform-dispatch split:

- `notify/mod.rs` (~40 lines) — module doc, `NotificationMeta` struct,
  re-exports, declares submodules.
- `notify/format.rs` (~50 lines) — pure text formatting: `format_quality`,
  `build_body` (no IO, easiest to unit test — currently untested; worth
  adding a quick test when splitting since the rule requires tests on
  touched code).
- `notify/artwork_cache.rs` (~100 lines) — `artwork_cache_dir`,
  `resolve_local_artwork`, `http_client`, `cache_artwork` (the IO layer:
  filesystem + blocking HTTP), all `#[cfg(any(linux, macos))]`.
- `notify/linux_icon.rs` (~50 lines) — `PORTAL_ICON_MAX_EDGE`/`_BYTES`
  consts + `prepare_icon_bytes` (crop/resize/encode), `#[cfg(target_os =
  "linux")]` only.
- `notify/show.rs` (~80 lines) — the public `show_track_notification` entry
  point, with its three `#[cfg(...)]` platform arms (linux/macos/other).

## Re-export surface
`notify/mod.rs` re-exports `NotificationMeta` and `show_track_notification`
(the only two items used outside this module — check callers in qbzd/
qbz-app) so `qbz_media_controls::notify::{NotificationMeta,
show_track_notification}` is unaffected.

## Coupling / watch out
- Heavy `#[cfg(target_os = ...)]` gating throughout — when splitting into
  files, keep each function's exact `#[cfg]` attribute; a file-level `#[cfg]`
  on the whole submodule is tempting but would silently drop the "linux OR
  macos" vs "linux only" distinction (`artwork_cache.rs` is linux+macos,
  `linux_icon.rs` is linux-only).
- `show_track_notification`'s linux arm calls `cache_artwork` then
  `prepare_icon_bytes` inside one `spawn_blocking` closure — both must stay
  reachable from `show.rs` (import both submodules).
- The whole notification path is fire-and-forget (errors logged, never
  propagated) — preserve that when moving code; don't accidentally add a
  `?` that changes error propagation.

## Verify after split
- `cargo check -p qbz-media-controls` on Linux (the CI/dev platform) — no
  unit tests exist today for this file; consider adding a couple for
  `format_quality`/`build_body` while splitting, per the project's "tests
  at each change" rule.
- Manual smoke-test: play a track, confirm a desktop notification appears
  with correct title/body/quality line and artwork icon.
