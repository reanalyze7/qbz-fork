# crates/qbz-ui/ui/discover/HomeSkeleton.slint (170 lines)

## Summary
Loading-skeleton placeholders for the Discover home page: a shared
`Shimmer` breathing-opacity block, a `SkeletonRow` (title + 5-card carousel
placeholder) used by the main `HomeSkeleton` component, plus two standalone
per-row skeletons (`SkeletonCarouselRow`, `SkeletonSlimRow`) used by the
Discover > Recommendations view while individual row builders resolve.

## Proposed split
By component family (shared primitive vs main skeleton vs standalone
per-row skeletons) — only slightly over budget, so a light two-way split:

- `discover/skeleton/Shimmer.slint` (~20 lines) — lines 16-25: the shared
  breathing `Shimmer` rectangle component.
- `discover/skeleton/HomeSkeleton.slint` (~55 lines) — lines 29-77:
  `SkeletonRow` + the exported `HomeSkeleton` component (its own 900ms
  `Timer` driving the shared `phase` for its two `SkeletonRow` children).
- `discover/skeleton/StandaloneRows.slint` (~90 lines) — lines 79-170:
  `SkeletonCarouselRow` and `SkeletonSlimRow`, each with its OWN independent
  900ms `Timer` (used by Discover > Recommendations in place of a row whose
  data hasn't resolved yet).
- `discover/HomeSkeleton.slint` (re-export shim, ~10 lines) — since the
  original file's exported name (`HomeSkeleton`) must keep working from its
  current path for the Discover home view's `import`, keep a thin
  `discover/HomeSkeleton.slint` that just does
  `import { HomeSkeleton } from "./skeleton/HomeSkeleton.slint"; export { HomeSkeleton };`
  (Slint supports re-export via `export { X } from "path";` shorthand,
  confirm exact syntax in this codebase's existing re-export shims before
  writing it) — OR, simpler and avoiding an extra indirection file entirely,
  just keep `HomeSkeleton` + `SkeletonRow` + `Shimmer` in-place in
  `discover/HomeSkeleton.slint` (it's only 77 lines, already under 130) and
  move ONLY `SkeletonCarouselRow`/`SkeletonSlimRow` out to
  `discover/skeleton/StandaloneRows.slint` since those are the two components
  actually pushing the file over the limit. This is the recommended, lower-
  churn option: two files total, not four.

## Re-export surface
Recommended minimal split: `discover/HomeSkeleton.slint` keeps exporting
`HomeSkeleton` at its current path (Discover home view's import is
unaffected). `discover/skeleton/StandaloneRows.slint` becomes the new home
for `SkeletonCarouselRow`/`SkeletonSlimRow` — the Discover > Recommendations
view (whichever file currently does
`import { SkeletonCarouselRow, SkeletonSlimRow } from "./HomeSkeleton.slint";`)
needs its import path updated to `"./skeleton/StandaloneRows.slint"`. Since
this DOES change an importer, grep for both component names across
`crates/qbz-ui/ui/discover/` before applying the real split, and update that
one import line as part of the same change (not left dangling).

## Coupling / watch out
- **Performance flag for the reported Discover freeze**: `Shimmer`,
  `SkeletonRow`, `HomeSkeleton`, `SkeletonCarouselRow`, and
  `SkeletonSlimRow` EACH declare an independent `Timer { interval: 900ms;
  running: true; ... }` (four separate `Timer` declarations total in this
  file: one in `HomeSkeleton`, one in `SkeletonCarouselRow`, one in
  `SkeletonSlimRow` — `SkeletonRow` itself takes `phase` as an `in property`
  from its parent rather than owning a timer). If Discover > Recommendations
  mounts several `SkeletonCarouselRow`/`SkeletonSlimRow` instances
  simultaneously (one per not-yet-resolved row), each gets its OWN 900ms
  timer AND its own independent `phase` (not synchronized with the others,
  since each has `property <bool> phase: false;` fully local) — this means
  N simultaneously-loading rows produce N independent timers all firing
  `animate opacity { duration: 900ms }` on every `Shimmer` child, all
  slightly desynchronized. Combined with `Carousel.slint`'s always-on 80ms
  windowing timers (see that file's plan) once the real carousels mount
  underneath, there is a real possibility of timer/animation pileup during
  the loading→loaded transition on the Discover page. This is exactly the
  kind of continuous-animation multiplication worth flagging to whoever
  investigates the reported freeze — the individual designs are each
  reasonable (reduce-motion is respected via `ShellState.reduce-motion`
  collapsing the animate duration to 0ms) but the AGGREGATE timer count when
  many skeleton rows are mounted at once has not obviously been considered.
- `ShellState.reduce-motion` gating (in `Shimmer`'s `animate opacity`
  duration) only affects the CSS-style transition length, NOT the `Timer`
  itself — the comment says "the 900ms Timer still flips the phase... no
  continuous opacity animation runs between flips (~1fps repaints instead of
  display-rate)" — i.e. even with reduce-motion on, N timers still fire every
  900ms. Keep this comment intact wherever `Shimmer` lands; it's the
  documented rationale for why the Timer isn't itself conditionally disabled.
- `ShellState` import (`../state.slint`) is used only by `Shimmer` — if
  `Shimmer` moves to its own file, only that file needs the import.

## Verify after split
- `slint-viewer` / project slint compile check.
- Grep-confirm no other `.slint` file imports `SkeletonCarouselRow` or
  `SkeletonSlimRow` from the old path before/after moving them.
- Full app build.
- Manually reproduce a cold Discover load with slow network (or artificially
  delay the loader) to see the skeleton multiple-timer behavior in practice,
  as a data point for the separate freeze investigation.
