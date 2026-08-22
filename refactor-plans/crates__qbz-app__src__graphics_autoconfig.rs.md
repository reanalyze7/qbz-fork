# crates/qbz-app/src/graphics_autoconfig.rs (185 lines)

## Summary
Framework-agnostic Linux graphics-environment detection for the diagnostics
panel: display server (Wayland/X11), GPU vendor(s) (NVIDIA/AMD/Intel, via
`/proc` and `/sys` probing, hybrid-laptop-aware), desktop-environment name, and
virtual-machine detection (DMI/hypervisor probing).

## Proposed split
Pure detection logic, no true IO-vs-render split applicable — split by-domain
(the four independent detection concerns), which also naturally separates the
`Environment` struct + top-level `detect_environment()` orchestrator from its
per-concern probes:

- `graphics_autoconfig/mod.rs` (~40 lines) — module doc, `Environment` struct
  (10-20), `detect_environment()` orchestrator (22-40), re-exports.
- `graphics_autoconfig/display.rs` (~15 lines) — `detect_display_server` (42-51).
  Tiny; could alternatively fold into `mod.rs` if that keeps it under 130 —
  either is fine, but keeping it separate mirrors the by-domain principle used
  elsewhere in this batch.
- `graphics_autoconfig/gpu.rs` (~100 lines) — `is_nvidia_gpu`, `is_amd_gpu`,
  `is_intel_gpu` (53-72), `detect_gpu_name` (105-124), `nvidia_name`,
  `amd_name`, `intel_name` (126-155): the whole GPU-vendor detection +
  human-readable naming pipeline, the largest cohesive chunk.
- `graphics_autoconfig/vm.rs` (~30 lines) — `is_virtual_machine` (74-103): DMI
  product/vendor string matching + `/sys/hypervisor/type` check.
- `graphics_autoconfig/desktop.rs` (~15 lines) — `detect_desktop` (157-171):
  env-var fallback chain (XDG_CURRENT_DESKTOP -> XDG_SESSION_DESKTOP ->
  DESKTOP_SESSION -> "Unknown").
- `graphics_autoconfig/tests.rs` (~15 lines) — the single test
  `gpu_name_combines_hybrid_vendors` (173-185); small enough to instead keep
  inline in `gpu.rs` as `#[cfg(test)] mod tests` since it only exercises
  `detect_gpu_name`.

## Re-export surface
`graphics_autoconfig/mod.rs` re-exports `Environment` and `detect_environment`
— the only two items referenced from outside per the module doc ("Consumed by
the diagnostics panel"). `detect_gpu_name` is also `pub fn` in the original —
keep it `pub` and re-exported too in case the diagnostics panel or a test
elsewhere calls it directly (it's called in the one existing test with
explicit bool args, suggesting it might be unit-tested/used standalone).

## Coupling / watch out
- `detect_environment()` calls all six probe functions unconditionally, in a
  fixed order, and none of them share mutable state (all are pure `-> bool` /
  `-> String` reads of `/proc`, `/sys`, or env vars) — this is low-risk to
  split; there is no interior coupling beyond `detect_gpu_name` needing the
  three bool results as its precomputed inputs (kept in `mod.rs`'s
  orchestrator, unchanged).
- The GPU detection is a "detect, then separately fetch a human-readable name"
  two-pass design (`is_nvidia_gpu()` is called once to gate `nvidia_name()`) —
  don't accidentally merge these into a single fallible probe during the
  split; the two-pass shape lets `detect_gpu_name` combine hybrid vendors
  (e.g. NVIDIA+Intel laptops), which is the whole point of the function per
  its doc comment.
- All probes are best-effort / fail-silent (`.unwrap_or(false)`,
  `.unwrap_or_default()`) — preserve this fail-open discipline in every
  extracted file; none of these functions should start returning `Result` or
  panicking.

## Verify after split
- `cargo build -p qbz-app`.
- `cargo test -p qbz-app graphics_autoconfig` — the one hybrid-vendor test
  green.
- `cargo clippy -p qbz-app`.
- Manual smoke-test optional: run the diagnostics panel (via the `run` skill)
  on the actual dev machine and confirm display-server/GPU/desktop/VM fields
  still populate correctly — most of this logic is unreachable in a typical CI
  container (no `/proc/driver/nvidia`, no real DE), so the unit test is the
  primary safety net.
