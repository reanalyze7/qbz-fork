# crates/qbz-ui/ui/foundation/theme.slint (260 lines)

## Summary
The single semantic-color sink for the whole UI: the `ThemeColors` struct
(Slint mirror of `qbz_theme::ThemeColors`, ~65 fields), the `Theme` global
seeded with a default Dark palette literal, and ~70 `out property` aliases
(with System-theme / std-Palette fallback logic for a handful of tokens) that
the ~1900 `Theme.<token>` call sites across the UI read.

## Proposed split
This is a single `global` — Slint globals CAN be defined once and their
properties referenced from anywhere, but a global's body also can't be split
across files as separate `global Theme` blocks (that would create two
identically-named globals, a conflict). The practical split here is:
(1) move the struct definition out, and (2) split the seed literal from the
alias properties, using Slint's `import`/re-export of the SAME global is not
possible — so the split must instead separate concerns while keeping ONE
`global Theme { ... }` block. Given that constraint, the realistic split is by
FILE, each `.slint` file containing one non-overlapping piece, with the `Theme`
global's property LIST assembled via multiple files is not supported in Slint;
propose instead moving the seed data out as a separate constant-like global
that `Theme` reads from, which Slint DOES support (globals can reference other
globals' properties).

- `foundation/theme-colors.slint` (~70 lines) — the `ThemeColors` struct
  definition only (currently lines 20-86). Exported so both `theme.slint` and
  Rust's Slint-generated bindings reference the same shape.
- `foundation/theme-seed.slint` (~65 lines) — a new `global ThemeSeed { out
  property <ThemeColors> dark: { ... }; }` holding the literal Dark-palette
  default struct value (currently lines 92-148, the big `c:` initializer).
  `theme.slint`'s `Theme.c` then reads `in-out property <ThemeColors> c:
  ThemeSeed.dark;` instead of inlining the literal.
- `foundation/theme.slint` (~125 lines) — stays the `Theme` global itself:
  imports `ThemeColors` from `theme-colors.slint` and `ThemeSeed` from
  `theme-seed.slint`, keeps `is-system`/`is-dark`/`is-high-contrast` flags, and
  ALL the `out property <color> ...` alias lines (lines 182-259, ~78 lines) —
  this is the part every call site actually reads and must not move (moving
  aliases elsewhere would need `Theme.` to still resolve them, i.e. no benefit
  to relocating them out of the `Theme` global).

## Re-export surface
`foundation/theme.slint`'s `export global Theme` stays the ONE thing every
other `.slint` file imports (`import { Theme } from
"../foundation/theme.slint";` or via `semantic-colors.slint` re-export, if
that's the actual current import path used across the UI — confirm which file
name the ~1900 call sites import, since the task listed
`foundation/theme.slint` but several files read above import
`foundation/semantic-colors.slint` instead; if `semantic-colors.slint` is a
separate re-export shim over this file, THAT file is the true public surface
and must keep re-exporting `Theme` unchanged). `ThemeColors` also needs to stay
importable from wherever Rust's `slint!`/build-script binding expects it
(check `build.rs` / generated Rust struct name mapping before moving the
struct file — Rust's `qbz_theme::ThemeColors` field/order lockstep comment at
the top of this file is a hard constraint, verify the generated binding still
resolves `ThemeColors` after the file move).
- Confirm the actual import path via `grep -rn "theme.slint\"" crates/qbz-ui/ui | head` (or `semantic-colors.slint`) before starting the real split — the plan above assumes `theme.slint` is the file imported directly; if `semantic-colors.slint` is the real facade, keep `theme.slint`'s export name `Theme` unchanged and let `semantic-colors.slint` do `export { Theme } from "./theme.slint";` untouched.

## Coupling / watch out
- The doc comment at the top says `ThemeColors` "field names are kebab-case and
  MUST stay in lockstep with the Rust `set_c` population order/names" — moving
  the struct to `theme-colors.slint` does NOT change field names/order, but
  double-check the Rust side (`crates/qbz-theme` or wherever `set_c` lives)
  doesn't import/generate against a hardcoded file path expectation.
- `is-system`-conditional properties (`surface-main`, `text-primary`, etc.)
  read BOTH `c.<field>` (from the seed/pushed struct) AND `StdPalette.*` — this
  ternary logic must stay in `theme.slint` itself (it's the "alias" layer, not
  data), not accidentally moved into `theme-seed.slint`.
- The `alpha-08`/`alpha-8` duplicate-name legacy alias (explicitly commented as
  intentional backward-compat) must be preserved exactly — don't "clean up"
  during the split.
- `.slint` live-preview relies on the Dark-palette seed being present with NO
  Rust running (the doc comment says so explicitly) — after extracting
  `ThemeSeed`, verify `slint-viewer` on any page still renders sane default
  colors (i.e. `ThemeSeed.dark` is reachable at preview time without Rust
  pushing anything).

## Verify after split
- Slint compile check across the whole `qbz-ui` crate (this file is imported
  everywhere; a mistake here breaks the entire UI compile).
- `slint-viewer` on a representative page to confirm the seeded Dark colors
  still render without Rust running.
- Full app smoke-test: launch, switch through Light/Dark/System/High-Contrast
  themes, confirm no color regresses (System mode's std-Palette fallback in
  particular).
- `grep -rn "ThemeColors" crates/qbz-theme crates/qbz-ui` to confirm the
  Rust-side struct binding still matches field names/order after the file
  move.
