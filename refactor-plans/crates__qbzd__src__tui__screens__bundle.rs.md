# crates/qbzd/src/tui/screens/bundle.rs (517 lines)

## Summary
qbzd setup TUI's Import/Export screen (03 §3.6) — a pure renderer of the
`qbz_app::settings::bundle` engine's plan (classification logic lives
there, not here, per the file's own note on the D9 safety flaw). Handles
import path entry → plan → three-bucket review → optional device re-pick
→ apply, plus export destination + include-auth toggle.

## Proposed split
- `bundle_screen/mod.rs` (~15 lines) — re-exports `PendingImport`,
  `BundleState`.
- `bundle_screen/state.rs` (~230 lines) — `PendingImport` struct, `BField`,
  `Editor` enums, `BundleState` struct + its non-draw methods: `new`,
  `is_editing`, `editing_label`, `set_plan`, `apply_context`,
  `clear_pending`, `handle_key`, `activate`, `handle_editor_key`,
  `handle_review_key`, `handle_auth_confirm`, `open_device_picker`,
  `handle_picker_key`, `replan_with` (lines 31-349).
- `bundle_screen/draw.rs` (~180 lines) — `draw`, `push_field`,
  `push_action`, `draw_review`, `BucketKind` enum, `bucket()` free
  function (lines 350-517).

## Re-export surface
`bundle_screen/mod.rs` becomes the `mod bundle;` target (rename directory
or keep filename `bundle.rs` as a thin `pub use state::*; pub use
draw::*;` re-export if the module path `crate::tui::screens::bundle::{
PendingImport, BundleState}` must stay unchanged) — check how
`crate::tui::screens::bundle` is referenced from `tui/app.rs` before
deciding directory-vs-flat-file layout.

## Coupling / watch out
- `BundleState`'s `draw`/`draw_review` methods and `state.rs`'s `handle_*`
  methods both need access to the same private fields — keep both impl
  blocks as siblings inside one `bundle_screen` module so field visibility
  (module-private) still works, same pattern as the wizard.rs split.
- `replan_with` (state.rs) and the device picker flow depend on
  `super::audio::{group_devices, DeviceEntry}` — that import stays with
  whichever file calls it (likely `state.rs`, since `open_device_picker`/
  `handle_picker_key` are there).
- `PendingImport` captures a `LiveSystem` snapshot specifically so a
  re-pick can replan without re-touching hardware — this laziness
  invariant must survive the split (don't accidentally re-fetch `live` in
  `replan_with`).
- The three-bucket classification (`PlanLine`, `ImportPlan`) is owned by
  `qbz_app::settings::bundle` — this screen only renders it; do not
  duplicate classification logic into `draw.rs`.

## Verify after split
- `cargo build -p qbzd`.
- Manually exercise: import with a matching device (verbatim apply),
  import with an absent device (device-picker re-pick + replan), export
  with/without include-auth.
