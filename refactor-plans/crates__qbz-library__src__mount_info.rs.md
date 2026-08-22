# crates/qbz-library/src/mount_info.rs (269 lines)

## Summary
Linux network-mount detection for local library paths: reads
`/proc/mounts` (with a Flatpak/Snap `/run/host/proc/mounts` fallback),
matches a given path against the longest-matching mount point, and classifies
its filesystem as network-backed (NFS/CIFS/SMB/sshfs/rclone/WebDAV/etc.) or
local — exposing both a bool (`is_network_path`) and a label
(`network_fs_label`) — plus a `#[cfg(all(test, target_os = "linux"))]` module
(lines 210–269, ~60 lines).

## Proposed split
By pure/IO boundary — this file already separates cleanly into "read the
mount table" (IO) vs "classify a path against it" (pure):

- `mount_info/mod.rs` (~55 lines) — module doc, the public API:
  `is_network_path` and `network_fs_label` (both linux + non-linux stub
  variants, lines 41–92 and 128–131), re-exporting the internals.
- `mount_info/io.rs` (~20 lines) — `read_mounts` (lines 152–167), the only
  actual filesystem IO in this file (`std::fs::read_to_string`).
- `mount_info/parse.rs` (~25 lines) — `parse_mounts` (lines 169–190), pure
  string parsing of `/proc/mounts` lines into `(mount_point, fs_type)` pairs.
- `mount_info/classify.rs` (~110 lines) — the pure matching/classification
  logic: `NETWORK_FS_PREFIXES` (lines 20–39), `best_fs_type`,
  `path_within_mount`, `normalize_network_label`, `is_network_fs` (lines
  100–126, 133–150, 192–208) — all pure functions over already-parsed data.
- `mount_info/tests.rs` (~60 lines) — the existing test module (lines
  210–269), covering `is_network_fs`, `best_fs_type`, `parse_mounts`.

## Re-export surface
`mount_info/mod.rs` re-exports `is_network_path` and `network_fs_label` at
`crate::mount_info::*` — the module path other qbz-library code uses
(`use crate::mount_info::{is_network_path, network_fs_label};` or similar) is
unchanged since `mount_info` stays a module name, just backed by a directory.

## Coupling / watch out
- Every function in this file except the tiny public API wrappers is
  `#[cfg(target_os = "linux")]`-gated, with matching non-Linux stub functions
  (`is_network_path` returns `false`, `network_fs_label` returns `None`) — the
  split must preserve BOTH branches; `classify.rs`/`io.rs`/`parse.rs` are
  Linux-only files in their entirety (whole-file `#![cfg(target_os =
  "linux")]` at the top, or per-item cfg — whole-file is cleaner here since
  every item in those 3 files is Linux-only), while `mod.rs` keeps both the
  linux and non-linux variants of the two public functions.
- `best_fs_type` and `path_within_mount` are tightly coupled (one calls the
  other in a loop) — keep them together in `classify.rs`.
- `normalize_network_label` and `is_network_fs` both consume raw fs-type
  strings from the SAME parsed mount data but for different purposes (label
  vs bool) — no shared state, just two independent pure functions, fine to
  colocate in `classify.rs`.
- The test module is `#[cfg(all(test, target_os = "linux"))]` — preserve this
  exact cfg combination (not just `#[cfg(test)]`) since the tested functions
  don't exist at all on non-Linux.

## Verify after split
- `cargo test -p qbz-library mount_info` on Linux — all 4 existing tests
  (`nfs_variants_classify_network`, `local_fs_does_not_classify`,
  `best_fs_type_respects_path_boundaries`, `best_fs_type_longest_mount_wins`,
  `parse_mounts_reads_typical_entries`) must stay green.
- `cargo check -p qbz-library --target <a-non-linux-triple>` if cross-check
  tooling is available, to confirm the non-Linux stub path still compiles.
- `cargo check -p qbz-library` for downstream crates depending on
  `qbz_library::mount_info::*`.
