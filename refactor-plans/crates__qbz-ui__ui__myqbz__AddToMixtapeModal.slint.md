# crates/qbz-ui/ui/myqbz/AddToMixtapeModal.slint (553 lines)

## Summary
The global "Add to Mixtape/Collection" picker modal — own backdrop/card
chrome, header, a picker panel (search + collection list + create buttons)
and a mutually-exclusive create-new panel (name + kind toggle + footer),
mounted once at the AppShell root and driven by `MyQbzAddState`/`MyQbzAddActions`.

## Proposed split
Slint doesn't have a `mod.rs` re-export mechanism like Rust, but components
defined across multiple files in the same directory are all importable by
name — so the split is "by component into sibling files", with the exported
`AddToMixtapeModal` staying the single import path other `.slint` files use.

- `myqbz/add_to_mixtape/kind_radio.slint` (~35 lines) — `KindRadio` component
  (lines 27-60): the labelled radio dot used only by the create-panel.
- `myqbz/add_to_mixtape/pick_row.slint` (~100 lines) — `AddPickRow` component
  (lines 62-158): one collection row in the picker list (icon + name/meta +
  kind tag + "already added" hint).
- `myqbz/add_to_mixtape/footer_button.slint` (~45 lines) — `FooterIconButton`
  component (lines 160-201): the "+ Mixtapes / + Collections" footer chip.
- `myqbz/AddToMixtapeModal.slint` (~380 lines, stays as-is at this path) —
  the exported `AddToMixtapeModal` component (lines 203-553): backdrop,
  header, picker-panel body, create-panel body. Imports `KindRadio`,
  `AddPickRow`, `FooterIconButton` from the three new sibling files above.
  This file is still the largest piece even after extracting the three leaf
  components (the two body panels — picker ~140 lines, create-new ~120
  lines — are each single-use and tightly wired to `MyQbzAddState`, so a
  further split would mean threading state through extra component
  boundaries for no real cohesion win; recommend leaving the two body
  panels inline unless line count after the leaf-component split still
  exceeds 130 — in that case, extract `AddPickerPanel` and
  `AddCreatePanel` as two more sibling components taking the state as
  `in` properties and re-emitting the same `MyQbzAddActions` calls).

## Re-export surface
`AddToMixtapeModal.slint` (same filename/path) stays the only import other
`.slint` files use (`import { AddToMixtapeModal } from "myqbz/AddToMixtapeModal.slint";`
— AppShell mounts it by this path). The three extracted components are
private implementation details imported only by `AddToMixtapeModal.slint`
itself; no other file references `KindRadio`/`AddPickRow`/`FooterIconButton`
today (grep confirms these three names appear nowhere else in
`crates/qbz-ui/ui/`), so extracting them is a pure internal refactor.

## Coupling / watch out
- All three extracted components take plain `in property` state (no shared
  globals) — `KindRadio` takes `label`/`selected` + `clicked()` callback,
  `AddPickRow` takes `data: MyQbzAddRow`/`index`/`busy` + `clicked(string)`,
  `FooterIconButton` takes `label`/`icon` + `clicked()` — all fully
  self-contained, no coupling risk.
- `AddPickRow` imports `QbzIcon` and references `MyQbzAddRow` (from
  `../state.slint`, two directory levels up from a new
  `myqbz/add_to_mixtape/` subfolder) — adjust the relative import path
  (`../../state.slint` instead of `../state.slint`) when moving into the
  subfolder.
- The parent `AddToMixtapeModal` still owns all reads/writes of
  `MyQbzAddState`/`MyQbzAddActions` — none of the three extracted leaf
  components touch the global state directly, so there's no risk of
  divergent global access after the split.
- The module doc comment (lines 1-16) describes the Tauri spec parity (spec
  21 §A) and the Rust controller (`crate::myqbz_add`) — keep this doc on the
  main `AddToMixtapeModal.slint` file since it describes the whole feature,
  not just one sub-component.

## Verify after split
- Slint compile check (`cargo build -p qbz-ui` or whichever crate compiles
  the `.slint` tree) succeeds with no unresolved imports.
- Smoke-test: run the app, open the Add-to-Mixtape modal from an album/track
  context menu, verify the picker list renders, search filters, "+
  Mixtapes"/"+ Collections" opens the create panel, and Create & Add / Back
  both work — since Slint has no unit-test runner, this manual pass is the
  practical verification.
- Grep `crates/qbz-ui/ui/` for `AddToMixtapeModal` to confirm only the
  top-level file's import path is referenced elsewhere (AppShell mount
  point), so the internal split is invisible to importers.
