# crates/qbz-app/src/diagnostics.rs (356 lines)

## Summary
Frontend-agnostic (pure std + `/proc`/`/sys`/`/etc/os-release`) builders for
two diagnostics snapshots — `RuntimeDiagnostics` (saved settings + detected
graphics/env state) and `SystemInfo` (OS/distro/installed-library versions) —
ported verbatim from the Tauri diagnostics command so the headless `qbz`
Slint bin can produce byte-identical JSON.

## Proposed split
By domain — the file already has two clearly-marked banner sections
(`==== Runtime Diagnostics ====` / `==== System Info ====`), split along
that line and then further by struct-vs-detection-logic:

- `diagnostics/mod.rs` (~15 lines) — module doc, `pub use` re-exports of
  both public builder fns + both public structs from the two submodules.
- `diagnostics/runtime.rs` (~185 lines) — lines 18-191: `RuntimeDiagnostics`
  struct, `GraphicsRuntime` struct, `DiagnosticsInputs` struct,
  `runtime_diagnostics()`, `detect_graphics_runtime()`. This is already just
  under budget as one file; if the reviewer prefers splitting further, break
  the struct+input types (~90 lines) from the two build functions (~95
  lines), but it isn't strictly required at 185 lines... actually re-check:
  130-line rule means this MUST be split further:
  - `diagnostics/runtime/types.rs` (~95 lines) — the three struct defs
    (`RuntimeDiagnostics`, `GraphicsRuntime`, `DiagnosticsInputs`).
  - `diagnostics/runtime/build.rs` (~95 lines) — `runtime_diagnostics()` +
    `detect_graphics_runtime()`.
- `diagnostics/system.rs` (~165 lines) — lines 193-356: `SystemInfo` struct
  + `system_info()` + the four detection helpers (`read_os_release`,
  `detect_kernel_version`, `detect_install_method`,
  `detect_loaded_lib_version`). Still over 130 — split further:
  - `diagnostics/system/types.rs` (~20 lines) — the `SystemInfo` struct.
  - `diagnostics/system/detect.rs` (~145 lines) — the four helper fns; if
    still tight, split `detect_loaded_lib_version` (lines 282-323, ~40
    lines of `/proc/self/maps` parsing) into its own
    `diagnostics/system/lib_version.rs`.
  - `diagnostics/system/build.rs` (~30 lines) — `system_info()` itself,
    which calls into the detect helpers.

## Re-export surface
`diagnostics/mod.rs` becomes the `mod diagnostics;` target already used from
wherever the `qbz` bin's diagnostics command / settings screen calls
`qbz_app::diagnostics::runtime_diagnostics(...)` and
`qbz_app::diagnostics::system_info()`. Both public structs
(`RuntimeDiagnostics`, `SystemInfo`) and both public builder fns must stay
reachable at their current `qbz_app::diagnostics::X` paths via `pub use
runtime::*; pub use system::*;` (which themselves `pub use types::*; pub use
build::*;`).

## Coupling / watch out
- `#[serde(rename_all = "camelCase")]` on both structs is explicitly called
  out in the file's doc comment as load-bearing (an existing Svelte TS
  interface depends on the exact JSON keys) — do not let the split touch
  field names, derive order, or the rename attribute.
- `runtime_diagnostics()` calls `crate::graphics_autoconfig::detect_gpu_name`
  and `detect_graphics_runtime()` calls `crate::graphics_autoconfig::
  detect_environment()` — both are external-crate-relative (`crate::`)
  paths, unaffected by an internal `diagnostics/` split, but keep the `use`
  statements in whichever file actually calls them.
- `DiagnosticsInputs<'a>` borrows `&'a qbz_audio::settings::AudioSettings`,
  `&'a crate::settings::graphics::GraphicsSettings`, `&'a crate::settings::
  developer::DeveloperSettings` — if `types.rs` and `build.rs` are split,
  both need the same `use` imports for these external types.
- The single `#[cfg(test)]`-free nature of this file means there's no
  existing automated coverage — verification here is compile + manual.

## Verify after split
- `cargo check -p qbz-app` and `cargo build -p qbz-app`.
- Smoke-test the diagnostics/about screen in the running app (or the
  equivalent CLI/settings export) still produces the same JSON keys/values
  as before the split — diff a captured snapshot if one exists.
